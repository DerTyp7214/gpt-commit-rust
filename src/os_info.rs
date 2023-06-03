use std::{env, time::SystemTime};

pub fn get_os_info() -> String {
    let os_platform = env::consts::OS.to_owned();
    let os_version = os_version::detect().unwrap().to_string();
    let os_arch = env::consts::ARCH.to_owned();
    let current_dir = env::current_dir().unwrap().to_str().unwrap().to_owned();
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    format!(
        "OS Platform: {}\nOS Version: {}\nOS Arch: {}\nCurrent Directory: {}\nTime: {}",
        os_platform, os_version, os_arch, current_dir, time
    )
}
