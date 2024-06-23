// src/constants.rs

pub const USERNAME_REGEX: &str = r"^[a-zA-Z0-9]{1,32}$";
pub const IP_HOSTNAME_MAX_LENGTH: usize = 128;

// Constants specific to the `site addip` command
pub const MIN_ADDIP_ARGS: usize = 2;
pub const MAX_ADDIP_IPS: usize = 10;