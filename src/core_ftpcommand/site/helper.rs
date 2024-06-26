use crate::constants::USERNAME_REGEX;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use url::Url;
use log::error;

use crate::constants::*;
use crate::Config;
use crate::session::Session;
use crate::tokio::fs;

// Load the src/cookies.rs
use crate::cookies::COOKIE_DEFINITIONS;

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

pub async fn replace_cookies(statline: String, session: Arc<Mutex<Session>>) -> String {
    let mut statline_replaced = statline;
    let session = session.lock().await;
    
    let ul_stat = (0.0, "MB".to_string());
    let dl_stat = (0.0, "MB".to_string());
    let speed_stat = (7181.07, "K/s".to_string());
    let free_space_stat = 973625.0;
    let section = "DEFAULT".to_string();
    let credits_stat = (14.6, "MB".to_string());
    let ratio_stat = "Unlimited".to_string();
    
    let replacements = vec![
        ("%[%.1f]IG%[%s]Y", format!("{:.1}{}", ul_stat.0, ul_stat.1)),
        ("%[%.1f]IJ%[%s]Y", format!("{:.1}{}", dl_stat.0, dl_stat.1)),
        ("%[%.2f]A%[%s]V", format!("{:.2}{}", speed_stat.0, speed_stat.1)),
        ("%[%.0f]FMB", format!("{:.0}MB", free_space_stat)),
        ("%[%s]b", section),
        ("%[%.1f]Ic%[%s]Y", format!("{:.1}{}", credits_stat.0, credits_stat.1)),
        ("%[%s]Ir", ratio_stat),
    ];
    
    for (cookie, value) in replacements {
        statline_replaced = statline_replaced.replace(cookie, &value);
    }

    // Remove formatting markers
    statline_replaced = statline_replaced.replace("!e", "");
    statline_replaced = statline_replaced.replace("!g", "");
    statline_replaced = statline_replaced.replace("[!G", "[");
    statline_replaced = statline_replaced.replace("]!0", "]");

    statline_replaced
}
