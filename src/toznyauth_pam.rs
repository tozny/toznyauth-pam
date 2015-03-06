#![feature(core)]
#![feature(old_io)]
#![feature(libc)]
#![feature(old_path)]
#![feature(std_misc)]
#![allow(dead_code)]

extern crate core;
extern crate getopts;
extern crate libc;
extern crate mdo;
extern crate pam;
extern crate qrcode;
extern crate toml;
extern crate tozny_auth;
extern crate url;

use libc::{c_char, c_int};
use mdo::result::{bind};
use pam::{constants, module};
use pam::conv::{PamConv};
use pam::constants::*;
use std::{ffi, fmt, num, str};
use std::old_io::timer::sleep;
use std::old_io::{Writer};
use std::time::Duration;
use tozny_auth::{login, protocol, question, user};
use tozny_auth::protocol::{Newtype};

use config::{Config, ConfigError};

mod config;
#[macro_use] mod my_mdo;
mod presence;
mod qr_term;

#[no_mangle]
pub extern fn pam_sm_authenticate(pamh: &module::PamHandleT, flags: PamFlag,
                                  argc: c_int, argv: *const *const c_char
                                  ) -> PamResultCode {
    let args = unsafe { translate_args(argc, argv) };
    let decision = mdo! {
        user   =<< module::get_user(pamh, None).map_err(AuthError::PamResult);
        config =<< Config::build(user.as_slice(), args.as_slice())
            .map_err(AuthError::ConfigError);
        conv   =<< unsafe { module::get_item::<PamConv>(pamh) }.map_err(AuthError::PamResult);
        login  =<< authenticate(&config, user.as_slice(), &conv);
        ign show_info(conv, flags, &format!("Authenticated as {}", login.user_display));
        ret Ok(constants::PAM_SUCCESS)
    };

    decision.unwrap_or_else(|e| {
        if flags & constants::PAM_SILENT == 0 {
            show_err(pamh, &e)
        }
        error_code(&e)
    })
}

#[allow(unused_variables)]
#[no_mangle]
pub extern fn pam_sm_setcred(pamh: *mut module::PamHandleT, flags: PamFlag,
                             argc: c_int, argv: *const *const c_char
                             ) -> PamResultCode {
    constants::PAM_SUCCESS
}

#[derive(Debug)]
enum AuthError {
    ConfigError(ConfigError),
    NotAuthorized,
    PamResult(PamResultCode),
    TimedOut,
    QuestionError(question::QuestionError),
}

fn authenticate(config: &config::Config,
                user: &str,
                conv: &PamConv,
                ) -> Result<login::Login, AuthError> {
    let user_api = config.get_user_api();
    let q        = AuthError::QuestionError;

    user_api.login_challenge().map_err(q)
    .and_then(|challenge| {
        let did_push = if config.presence {
            let d = push_notification(&user_api, config, &challenge.session_id);
            let _ = presence::save_presence(user, &config.home_dir, &challenge.presence);
            d
        } else { false };
        if did_push { show_push(config, conv) } else { show_qr(config, conv, &challenge) }
        .and_then(|_| {
            poll_session_status(&user_api, &challenge, 100)  // TODO: configurable TTL
        })
    })
    .and_then(|question| {
        question::unpack::<login::Login>(&question.signed_data).map_err(q)
    })
    .and_then(|login| {
        if config.is_authorized(&login) { Ok(login) } else { Err(AuthError::NotAuthorized) }
    })
}

fn push_notification(user_api: &user::UserApi,
                     config: &config::Config,
                     session_id: &protocol::SessionId,
                     ) -> bool {
    match presence::get_presence(&config.home_dir) {
        Some(presence) => {
            user_api.push(session_id, &presence)
            .map(|_| true)
            .unwrap_or(false)
        },
        None => false,
    }
}

fn show_push(config: &config::Config, conv: &PamConv) -> Result<(), AuthError> {
    interact(config, conv, |writer| {
        let _ = writer.write_str("Check your phone for a push notification from Tozny.");
    })
}

fn show_qr(config: &config::Config, conv: &PamConv, challenge: &user::LoginChallenge
           ) -> Result<(), AuthError> {
    interact(config, conv, |writer| {
        if config.qr {
            show_inline_qr(challenge, writer)
        }
        else {
            show_qr_url(challenge, writer)
        }
        if config.mobile_url {
            let _ = writer.write_fmt(format_args!(
                "\n\nIf you are on your mobile device, use this URL to invoke the Tozny app:\n{}",
                challenge.mobile_url.to_string()));
        }
    })
}

fn show_inline_qr(challenge: &user::LoginChallenge, writer: &mut Vec<u8>) {
    let qr = build_qr(challenge);
    let _ = writer.write_str("\n");
    qr_term::output_unicode(&qr, "        ", writer);
    let _ = writer.write_fmt(format_args!(
        "\nScan the code above with the Tozny app. \
        Or if the code does not display correctly, open this URL:\n{}",
        challenge.qr_url.to_string()));
}

fn show_qr_url(challenge: &user::LoginChallenge, writer: &mut Vec<u8>) {
    let _ = writer.write_fmt(format_args!(
        "Open this URL, and scan the QR code with the Tozny app:\n{}",
        challenge.qr_url.to_string()));
}

fn interact<F>(config: &config::Config, conv: &PamConv, f: F) -> Result<(), AuthError>
        where F: Fn(&mut Vec<u8>) {
    let mut writer = Vec::new();
    f(&mut writer);
    if config.prompt {
        let _ = writer.write_str("\n\nPress Enter at any time:");
    }
    let mtype = if config.prompt { PAM_PROMPT_ECHO_OFF } else { PAM_TEXT_INFO };
    conv.send(mtype, str::from_utf8(writer.as_slice()).unwrap())
    .map_err(AuthError::PamResult)
    .and(Ok(()))
}

fn poll_session_status(api: &user::UserApi, challenge: &user::LoginChallenge, ttl: usize
                       ) -> Result<question::Question, AuthError> {
    api.check_session_status(&challenge.session_id)
        .map_err(AuthError::QuestionError)
    .and_then(|res| {
        match res {
            Some(question) => Ok(question),
            None => {
                if ttl > 0 {
                    sleep(Duration::seconds(1));
                    poll_session_status(api, challenge, ttl - 1)
                }
                else {
                    Err(AuthError::TimedOut)
                }
            }
        }
    })
}

fn error_code(err: &AuthError) -> PamResultCode {
    match err {
        &AuthError::ConfigError(_)         => PAM_AUTHINFO_UNAVAIL,
        &AuthError::NotAuthorized          => PAM_PERM_DENIED,
        &AuthError::PamResult(code)        => code,
        &AuthError::TimedOut               => PAM_AUTHINFO_UNAVAIL,
        &AuthError::QuestionError(ref err) => match err {
            &question::QuestionError::InvalidSignature => PAM_AUTH_ERR,
            _                                          => PAM_SERVICE_ERR,
        },
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &AuthError::ConfigError(ref err) => err.fmt(f),
            &AuthError::NotAuthorized        => {
                f.write_str("You are not authorized to access this account.")
            }
            &AuthError::PamResult(_)         => Ok(()),
            &AuthError::TimedOut             => {
                f.write_str("Timed out waiting for user to authenticate.")
            }
            &AuthError::QuestionError(ref err) => err.fmt(f),
        }
    }
}

fn show_info<E>(conv: &PamConv, flags: PamFlag, info: &str) -> Result<(), E> {
    if flags & constants::PAM_SILENT == 0 {
        let _ = conv.send(PAM_TEXT_INFO, info);
    }
    Ok(())
}

fn show_err(pamh: &module::PamHandleT, err: &AuthError) {
    for conv in unsafe { module::get_item::<PamConv>(pamh) }.iter() {
        let _ = conv.send(PAM_ERROR_MSG, &format!("{}", err));
    }
}

fn build_qr(challenge: &user::LoginChallenge) -> qrcode::QrCode {
    qrcode::QrCode::new(challenge.mobile_url.to_string().as_bytes()).unwrap()
}

unsafe fn translate_args(argc: c_int, argv: *const *const c_char) -> Vec<String> {
    let v = Vec::<*const c_char>::from_raw_buf(argv, num::cast(argc).unwrap());
    v.into_iter().filter_map(|arg| {
        let bytes = ffi::CStr::from_ptr(arg).to_bytes();
        String::from_utf8(bytes.to_vec()).ok()
    })
    .collect()
}
