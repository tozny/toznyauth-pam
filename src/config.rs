use getopts;
use getopts::Options;
use std;
use std::clone::Clone;
use std::fmt;
use std::old_io::{Command, File, Reader};
use std::old_path::{GenericPath};
use std::old_io::fs::{PathExtensions};
use std::old_path::posix::{Path};
use toml;
use tozny_auth::{Login, UserApi};
use tozny_auth::protocol::{KeyId, Newtype};
use url;


#[derive(PartialEq, Debug)]
pub struct Config {
    realm_key_id:     KeyId,
    api_url:          url::Url,
    authorized_users: toml::Array,
    pub home_dir:     Path,
    pub presence:     bool,
    pub prompt:       bool,
    pub qr:           bool,
    pub mobile_url:   bool,
}

impl Config {
    pub fn build(unix_user: &str, args: &[String]) -> Result<Config, ConfigError> {
        program_opts().parse(args).map_err(ConfigError::GetoptsError)
        .and_then(|opts| {
            get_home(unix_user).ok_or(ConfigError::NoHomeDir)
            .and_then(|home| {
                let auth = get_auth_path(&home);
                if auth.is_file() { Ok((home, auth)) }
                    else { Err(ConfigError::MissingAuthFile(auth)) }
            })
            .and_then(|(home, auth)| {
                read_config(&auth)
                .map(|config_file| {
                    Config {
                        realm_key_id:     config_file.realm_key_id,
                        api_url:          config_file.api_url,
                        authorized_users: config_file.authorized_users,
                        home_dir:         home,
                        prompt:           opts.opt_present("prompt"),
                        presence:         !opts.opt_present("no-presence"),
                        qr:               !opts.opt_present("no-qr"),
                        mobile_url:       !opts.opt_present("no-mobile"),
                    }
                })
            })
        })
    }

    pub fn get_user_api(&self) -> UserApi {
        UserApi::new(self.realm_key_id.clone(), self.api_url.clone())
    }

    pub fn is_authorized(&self, login: &Login) -> bool {
        self.authorized_users.iter().filter_map(|u| u.as_str()).any(|u| {
            u == login.user_id.as_slice()
        })
    }
}

#[derive(PartialEq, Debug)]
struct ConfigFile {
    realm_key_id:     KeyId,
    api_url:          url::Url,
    authorized_users: toml::Array,
}

impl ConfigFile {
    fn from_table(table: &toml::Table) -> Result<ConfigFile, ConfigError> {
        let url_raw: &str = get(table, "api_url").and_then(as_str)
            .unwrap_or("https://api.tozny.com");
        url::Url::parse(url_raw).map_err(ConfigError::InvalidUrl)
        .and_then(|url| {
            get(table, "realm_key_id")
            .and_then(as_str)
            .and_then(move |key| {
                get(table, "authorized_users")
                .and_then(as_slice)
                .map(|users| {
                    ConfigFile {
                        realm_key_id:     KeyId::from_slice(key),
                        api_url:          url,
                        authorized_users: users.to_vec(),
                    }
                })
            })
        })
    }
}

fn program_opts() -> Options {
    let mut opts = Options::new();
    opts.optflag("p", "prompt", "prompts user to press Enter (might be required with OpenSSH)");
    opts.optflag("Q", "no-qr", "suppresses display of QR code");
    opts.optflag("P", "no-presence", "disables push notifications");
    opts.optflag("M", "no-mobile", "disables display of mobile URL");
    opts
}

fn read_config(path: &Path) -> Result<ConfigFile, ConfigError> {
    File::open(path)
    .read_to_string()
    .map_err(ConfigError::ErrorReading)
    .and_then(|input| {
        toml::Parser::new(&input).parse()
            .ok_or(ConfigError::ParseError)
    })
    .and_then(|table| {
        ConfigFile::from_table(&table)
    })
}

#[derive(Debug)]
pub enum ConfigError {
    ErrorReading(std::old_io::IoError),
    GetoptsError(getopts::Fail),
    InvalidUrl(url::ParseError),
    MissingField(String),
    MissingAuthFile(Path),
    NoHomeDir,
    ParseError,
    TypeError(&'static str, &'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &ConfigError::ErrorReading(_) => {
                f.write_str("Error reading configuration file.")
            }
            &ConfigError::GetoptsError(ref e) => {
                f.write_fmt(format_args!("{}", e))
            }
            &ConfigError::InvalidUrl(ref e) => {
                f.write_fmt(format_args!("Invalid api_url: {}", e))
            }
            &ConfigError::MissingField(ref key) => {
                f.write_fmt(format_args!("Missing key in configuration: {}", key))
            }
            &ConfigError::MissingAuthFile(ref path) => {
                f.write_fmt(format_args!("No such file, {:?}", path))
            }
            &ConfigError::NoHomeDir => {
                f.write_str("Expected to find authorized file in home directory, but user has no home directory.")
            }
            &ConfigError::ParseError => {
                // TODO: Read errors from parser object.
                f.write_str("Error parsing configuration file.")
            }
            &ConfigError::TypeError(ref expected, ref actual) => {
                f.write_fmt(format_args!(
                        "Type error in configuration: expected {}, but got {}",
                        expected, actual))
            }
        }
    }
}

fn get_home(user: &str) -> Option<Path> {
    Command::new("getent").arg("passwd").arg(user).output().ok()
    .and_then(|out| {
        if out.status.success() {
            Some(out.output)
        }
        else {
            None
        }
    })
    .and_then(|bytes| {
        String::from_utf8(bytes).ok()
    })
    .and_then(|passwd| {
        let home_dir = passwd.split(":").skip(5).next().unwrap();
        let path = Path::new(home_dir);
        if path.exists() { Some(path) } else { None }
    })
}

fn get_auth_path(home: &Path) -> Path {
    let mut auth = home.clone();
    auth.push(".config");
    auth.push("tozny");
    auth.push("authorized.toml");
    auth
}

fn as_str(v: &toml::Value) -> Result<&str, ConfigError> {
    v.as_str().ok_or_else(|| {
        ConfigError::TypeError("String", v.type_str())
    })
}

fn as_slice(v: &toml::Value) -> Result<&[toml::Value], ConfigError> {
    v.as_slice().ok_or_else(|| {
        ConfigError::TypeError("Array", v.type_str())
    })
}

fn get<'a>(table: &'a toml::Table, key: &str) -> Result<&'a toml::Value, ConfigError> {
    table.get(key).ok_or_else(|| {
        ConfigError::MissingField(key.to_string())
    })
}
