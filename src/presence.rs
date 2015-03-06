use std::iter::{IteratorExt};
use std::{old_io, str};
use std::old_io::{fs, Command, FileAccess, FileMode, FileType, Reader};
use std::old_path::{GenericPath};
use std::old_path::posix::{Path};
use tozny_auth::protocol::{Newtype, Presence};

pub fn get_presence(home: &Path) -> Option<Presence> {
    get_presence_file(home)
    .and_then(|path| {
        fs::File::open(&path).ok()
    })
    .and_then(|mut file| {
        file.read_to_string().ok()
    })
    .map(Presence::new)
}

pub fn save_presence(user: &str, home: &Path, presence: &Presence) {
    get_presence_file(home)
    .and_then(|path| {
        fs::File::open_mode(&path, FileMode::Truncate, FileAccess::Write).ok()
        .and_then(|mut file| {
            file.write_str(presence.as_slice()).ok()
        });

        // If this module is used for `sudo` authentication, presence file will
        // be in user's home directory, but will be owned by root.
        get_id("-u", user)
        .and_then(|uid| {
            get_id("-g", user).map(|gid| {
                let _ = fs::chown(&path, uid, gid);
            })
        })
    });
}


pub fn get_presence_file(home: &Path) -> Option<Path> {
    let mut file = home.clone();
    file.push(".cache");
    match fs::stat(&file) {
        Ok(stat) => {
            if stat.kind == FileType::Directory {
                Some(())
            }
            else {
                None
            }
        },
        Err(_) => {
            fs::mkdir(&file, old_io::USER_RWX).ok().map(|_| ())
        }
    }
    .map(|_| {
        file.push("toznyauth_pam_presence");
        file
    })
}

fn get_id(flag: &str, user: &str) -> Option<isize> {
    Command::new("id").arg(flag).arg(user).output().ok()
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
    .and_then(|s| {
        str::FromStr::from_str(s.as_slice()).ok()
    })
}
