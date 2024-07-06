use crate::Config;

use std::path::PathBuf;






/// Constructs the directory path within the user's current directory and the server's chroot directory.
pub fn construct_path(config: &Config, current_dir: &str, sanitized_arg: &str) -> PathBuf {
    // Join the current directory with the sanitized argument to form the directory path.
    let new_dir = PathBuf::from(current_dir).join(sanitized_arg);
    // Convert the directory path to a string, trimming leading slashes.
    let new_dir_str = new_dir
        .to_str()
        .unwrap()
        .trim_start_matches('/')
        .to_string();
    // Construct the full path within the chroot directory.
    let dir_path = PathBuf::from(&config.server.chroot_dir)
        .join(config.server.min_homedir.trim_start_matches('/'))
        .join(new_dir_str);

    dir_path
}
