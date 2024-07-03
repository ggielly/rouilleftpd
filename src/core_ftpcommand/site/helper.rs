use crate::constants::USERNAME_REGEX;
use log::error;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use url::Url;

use crate::constants::*;
use crate::tokio::fs;
use crate::Config;
use std::collections::HashMap;

// Load the src/cookies.rs

pub async fn respond_with_error(
    writer: &Arc<Mutex<TcpStream>>,
    msg: &[u8],
) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(msg).await
}

/// Sends a success response message to the FTP client.
///
/// This function locks the provided TCP stream writer and writes a success message to the client.
/// It is typically used to acknowledge the successful completion of an FTP command.
///
/// # Arguments
///
/// * `writer` - A reference to an `Arc<Mutex<TcpStream>>` that represents the TCP stream writer.
/// * `msg` - A byte slice containing the success message to be sent to the client.
///
/// # Returns
///
/// This function returns a `Result` that is:
/// * `Ok(())` if the message was successfully written to the client.
/// * `Err(std::io::Error)` if there was an error writing the message.
///
/// # Errors
///
/// This function will return an error if it fails to acquire a lock on the TCP stream writer or if it fails to write the message to the stream.
pub async fn respond_with_success(
    writer: &Arc<Mutex<TcpStream>>,
    msg: &[u8],
) -> Result<(), std::io::Error> {
    let mut writer = writer.lock().await;
    writer.write_all(msg).await
}

/// Checks if a string is a valid IPv4 address or hostname.
///
/// # Arguments
///
/// * `ip` - The string to validate.
/// * `max_length` - The maximum allowed length of the string.
///
/// # Returns
///
/// Returns `true` if the string is a valid IPv4 address or hostname, `false` otherwise.
pub fn is_valid_ip_or_hostname(ip: &str) -> bool {
    const IP_HOSTNAME_MAX_LENGTH: usize = 128;

    if ip.len() > IP_HOSTNAME_MAX_LENGTH {
        return false;
    }

    // Check if it's a valid IPv4 address
    if ip.parse::<std::net::Ipv4Addr>().is_ok() {
        return true;
    }

    // Check if it's a valid FQDN
    Url::parse(&format!("http://{}", ip)).is_ok()
}

pub fn is_valid_ident_ip(ident_ip: &str) -> bool {
    let parts: Vec<&str> = ident_ip.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let ident = parts[0];
    let ip_or_hostname = parts[1];

    if ident.is_empty() || !is_valid_ip_or_hostname(ip_or_hostname) {
        return false;
    }
    true
}

/// Validates the username according to the defined rules.
///
/// # Arguments
///
/// * `username` - The username to validate.
///
/// # Returns
///
/// Returns `true` if the username is valid, `false` otherwise.
pub fn is_valid_username(username: &str) -> bool {
    let re = regex::Regex::new(USERNAME_REGEX).unwrap();
    if username == "rouilleftpd" {
        return false;
    }
    re.is_match(username)
}

/// Performs a basic validation of the password.
///
/// # Arguments
///
/// * `password` - The password to validate.
///
/// # Returns
///
/// Returns `true` if the password is not empty, `false` otherwise.
pub fn is_valid_password(password: &str) -> bool {
    !password.is_empty() // You should implement more robust password checks
}

/// Maps flag names to their respective values.
pub fn get_flag_value(flag_name: &str) -> Option<u8> {
    match flag_name.to_uppercase().as_str() {
        "SITEOP" => Some(SITEOP),
        "GADMIN" => Some(GADMIN),
        "GLOCK" => Some(GLOCK),
        "EXEMPT" => Some(EXEMPT),
        "COLOR" => Some(COLOR),
        "DELETED" => Some(DELETED),
        "USEREDIT" => Some(USEREDIT),
        "ANONYMOUS" => Some(ANONYMOUS),
        "NUKE" => Some(NUKE),
        "UNNUKE" => Some(UNNUKE),
        "UNDUPE" => Some(UNDUPE),
        "KICK" => Some(KICK),
        "KILL" => Some(KILL),
        "TAKE" => Some(TAKE),
        "GIVE" => Some(GIVE),
        "USERS" => Some(USERS),
        "IDLER" => Some(IDLER),
        "CUSTOM1" => Some(CUSTOM1),
        "CUSTOM2" => Some(CUSTOM2),
        "CUSTOM3" => Some(CUSTOM3),
        "CUSTOM4" => Some(CUSTOM4),
        "CUSTOM5" => Some(CUSTOM5),
        _ => None,
    }
}

/// Maps flag values to their respective names.
pub fn get_flag_name(flag_value: u8) -> Option<&'static str> {
    match flag_value {
        SITEOP => Some("SITEOP"),
        GADMIN => Some("GADMIN"),
        GLOCK => Some("GLOCK"),
        EXEMPT => Some("EXEMPT"),
        COLOR => Some("COLOR"),
        DELETED => Some("DELETED"),
        USEREDIT => Some("USEREDIT"),
        ANONYMOUS => Some("ANONYMOUS"),
        NUKE => Some("NUKE"),
        UNNUKE => Some("UNNUKE"),
        UNDUPE => Some("UNDUPE"),
        KICK => Some("KICK"),
        KILL => Some("KILL"),
        TAKE => Some("TAKE"),
        GIVE => Some("GIVE"),
        USERS => Some("USERS"),
        IDLER => Some("IDLER"),
        CUSTOM1 => Some("CUSTOM1"),
        CUSTOM2 => Some("CUSTOM2"),
        CUSTOM3 => Some("CUSTOM3"),
        CUSTOM4 => Some("CUSTOM4"),
        CUSTOM5 => Some("CUSTOM5"),
        _ => None,
    }
}

pub async fn load_statline(config: Arc<Config>) -> Result<String, std::io::Error> {
    let statline_path = format!("{}/ftp-data/text/statline.txt", config.server.chroot_dir);

    match fs::read_to_string(statline_path).await {
        Ok(content) => Ok(content),
        Err(e) => {
            error!("Failed to read statline file: {}", e);
            Err(e)
        }
    }
}

pub fn cleanup_cookie_statline(statline: &str, replacements: &HashMap<&str, String>) -> String {
    let mut statline_replaced = statline.to_string();

    // Iterate through the replacements and replace placeholders with actual values
    for (placeholder, value) in replacements {
        statline_replaced = statline_replaced.replace(placeholder, value);
    }

    // Remove formatting markers
    statline_replaced = statline_replaced.replace("!e", "");
    statline_replaced = statline_replaced.replace("!g", "");
    statline_replaced = statline_replaced.replace("!G", "");
    statline_replaced = statline_replaced.replace("!C", "");
    statline_replaced = statline_replaced.replace("!0", "");
    statline_replaced = statline_replaced.replace("!I", "");
    statline_replaced = statline_replaced.replace("!Z", "");
    statline_replaced = statline_replaced.replace("!@", "");

    statline_replaced
}

pub async fn cleanup_cookie_site_user<'a>(
    user_info: &'a HashMap<&str, String>,
    template: &'a str,
    username: &'a str,
) -> String {
    let mut placeholders = HashMap::new();

    placeholders.insert("%[%-20s]Iu", username.to_string()); // Username
    placeholders.insert("%[%-20s]IC", "".to_string()); // Created (dummy value)
    placeholders.insert("%[%-20s]I-", "".to_string()); // Added by (dummy value)
    placeholders.insert("%[%-20s]I!", "".to_string()); // Expires (dummy value)
    placeholders.insert(
        "%[%-19s]Ie",
        user_info.get("TIME").unwrap_or(&"".to_string()).to_string(),
    ); // Time On Today
    placeholders.insert("%[%-24s]I+", "".to_string()); // Last seen (dummy value)
    placeholders.insert(
        "%[%-22s]IZ",
        user_info
            .get("FLAGS")
            .unwrap_or(&"".to_string())
            .to_string(),
    ); // Flags
    placeholders.insert("%[%-10s]I@", "".to_string()); // Idle time (dummy value)
    placeholders.insert(
        "%[%-61s]I=",
        user_info
            .get("RATIO")
            .unwrap_or(&"".to_string())
            .to_string(),
    ); // Ratios
    placeholders.insert(
        "%[%-60s]IX",
        user_info
            .get("CREDITS")
            .unwrap_or(&"".to_string())
            .to_string(),
    ); // Credits
    placeholders.insert(
        "%[%-9d]IL",
        user_info
            .get("LOGINS")
            .unwrap_or(&"".to_string())
            .to_string(),
    ); // Total Logins
    placeholders.insert("%[%-9d]Im", "0".to_string()); // Current Logins (dummy value)
    placeholders.insert("%[%-9s]IM", "".to_string()); // Max Logins (dummy value)
    placeholders.insert("%[%-9s]I#", "".to_string()); // From same IP (dummy value)

    cleanup_cookie_statline(template, &placeholders)
}
