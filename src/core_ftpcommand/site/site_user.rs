use crate::core_ftpcommand::site::helper::{
    respond_with_error, respond_with_success,
};
use crate::{session::Session, Config};

use crate::constants::DELETED;
use std::{
    fs::{self},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{net::TcpStream, sync::Mutex};

const MIN_SITE_USER_ARGS: usize = 1;

pub async fn handle_site_user_command(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    _session: Arc<Mutex<Session>>,
    arg: String,
) -> Result<(), std::io::Error> {
    let args: Vec<&str> = arg.split_whitespace().collect();
    if args.len() < MIN_SITE_USER_ARGS {
        list_all_users(writer, config).await
    } else {
        let username = args[0].to_string();
        show_user_info(writer, config, _session, &username).await
    }
}

async fn list_all_users(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
) -> Result<(), std::io::Error> {
    let users_dir = PathBuf::from(&config.server.chroot_dir).join("ftp-data/users");

    let mut users = Vec::new();

    if let Ok(entries) = fs::read_dir(users_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.ends_with(".user") {
                        let user_name = file_name.trim_end_matches(".user").to_string();
                        if !is_user_deleted(&entry.path()) {
                            users.push(user_name);
                        }
                    }
                }
            }
        }
    }

    let response = format!("200 Non-deleted users: {}\r\n", users.join(", "));
    respond_with_success(&writer, response.as_bytes()).await
}

async fn show_user_info(
    writer: Arc<Mutex<TcpStream>>,
    config: Arc<Config>,
    _session: Arc<Mutex<Session>>,
    username: &str,
) -> Result<(), std::io::Error> {
    let user_file_path = PathBuf::from(&config.server.chroot_dir)
        .join("ftp-data/users")
        .join(format!("{}.user", username));

    if !user_file_path.exists() {
        respond_with_error(&writer, b"550 User not found.\r\n").await?;
        return Ok(());
    }

    let mut user_data = String::new();
    {
        let mut file = fs::File::open(&user_file_path)?;
        file.read_to_string(&mut user_data)?;
    }

    // Parse user file
    let user_info = parse_user_data(&user_data);

    // Load and replace statline template
    let user_info_template = load_user_info_template(config.clone()).await?;
    let cleaned_info = format_user_info(&user_info_template, &user_info);

    respond_with_success(&writer, cleaned_info.as_bytes()).await
}

fn parse_user_data(user_data: &str) -> UserInfo {
    let mut user_info = UserInfo::default();

    for line in user_data.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let key = parts[0];
        let value = parts[1..].join(" ");

        match key {
            "USER" => user_info.username = value,
            "ADDED" => user_info.created = value,
            "ADDEDBY" => user_info.added_by = value,
            "EXPIRES" => user_info.expires = value,
            "LAST" => user_info.last_seen = value,
            "FLAGS" => user_info.flags = value,
            "CREDITS" => user_info.credits = value,
            "LOGINS" => {
                let logins: Vec<&str> = value.split_whitespace().collect();
                if logins.len() >= 2 {
                    user_info.total_logins = logins[0].parse().unwrap_or_default();
                    user_info.current_logins = logins[1].parse().unwrap_or_default();
                }
            }
            "MAXLOGINS" => user_info.max_logins = value,
            "MAXIP" => user_info.from_same_ip = value,
            "MAXUP" => user_info.max_sim_uploads = value,
            "MAXDN" => user_info.max_sim_downloads = value,
            "MAXUPSP" => user_info.max_upload_speed = value,
            "MAXDNSP" => user_info.max_download_speed = value,
            "NUKE" => user_info.times_nuked = value,
            "BYTESNUKED" => user_info.bytes_nuked = value,
            "WKLYALLOT" => user_info.weekly_allotment = value,
            "MSGS" => user_info.messages_waiting = value,
            "TIMELIMIT" => user_info.time_limit = value,
            "TIMEFRAME" => user_info.timeframe = value,
            "TAGLINE" => user_info.tagline = value,
            "GROUPS" => user_info.groups = value,
            "PRIVGROUPS" => user_info.priv_groups = value,
            _ => {}
        }
    }

    user_info
}

#[derive(Default)]
struct UserInfo {
    username: String,
    created: String,
    added_by: String,
    expires: String,
    last_seen: String,
    flags: String,
    credits: String,
    total_logins: usize,
    current_logins: usize,
    max_logins: String,
    from_same_ip: String,
    max_sim_uploads: String,
    max_sim_downloads: String,
    max_upload_speed: String,
    max_download_speed: String,
    times_nuked: String,
    bytes_nuked: String,
    weekly_allotment: String,
    messages_waiting: String,
    time_limit: String,
    timeframe: String,
    tagline: String,
    groups: String,
    priv_groups: String,
}

fn format_user_info(template: &str, user_info: &UserInfo) -> String {
    template
        .replace("%[%-20s]Iu", &user_info.username)
        .replace("%[%-20s]IC", &user_info.created)
        .replace("%[%-20s]I-", &user_info.added_by)
        .replace("%[%-20s]I!", &user_info.expires)
        .replace("%[%-19s]Ie", &user_info.last_seen)
        .replace("%[%-24s]I+", &user_info.last_seen)
        .replace("%[%-22s]IZ", &user_info.flags)
        .replace("%[%-10s]I@", &user_info.flags)
        .replace("%[%-61s]I=", &user_info.credits)
        .replace("%[%-60s]IX", &user_info.credits)
        .replace("%[%-9d]IL", &user_info.total_logins.to_string())
        .replace("%[%-9d]Im", &user_info.current_logins.to_string())
        .replace("%[%-9s]IM", &user_info.max_logins)
        .replace("%[%-9s]I#", &user_info.from_same_ip)
        .replace("%[%-9s]I^", &user_info.max_sim_uploads)
        .replace("%[%-9s]I*", &user_info.max_sim_downloads)
        .replace("%[%7.1f]I&%[%s]V", &user_info.max_upload_speed)
        .replace("%[%7.1f]Iz%[%s]V", &user_info.max_download_speed)
        .replace("%[%-5d]IO", &user_info.times_nuked)
        .replace("%[%6.0f]Ix%[%s]Y", &user_info.bytes_nuked)
        .replace("%[%5.0f]IW%[%s]Y", &user_info.weekly_allotment)
        .replace("%[%-10s]Id", &user_info.messages_waiting)
        .replace("%[%4d]Iw", &user_info.time_limit)
        .replace("%[%s]IE", &user_info.timeframe)
        .replace("%[%-55s]It", &user_info.tagline)
        .replace("%[%-62s]Iy", &user_info.groups)
        .replace("%[%-57s]IY", &user_info.priv_groups)
}

fn is_user_deleted(user_file_path: &Path) -> bool {
    let mut user_data = String::new();
    if let Ok(mut file) = fs::File::open(user_file_path) {
        if let Ok(_) = file.read_to_string(&mut user_data) {
            for line in user_data.lines() {
                if line.starts_with("FLAGS") {
                    return line.contains(DELETED.to_string().as_str());
                }
            }
        }
    }
    false
}

// Helper function to load user info template
async fn load_user_info_template(config: Arc<Config>) -> Result<String, std::io::Error> {
    let template_path = PathBuf::from(&config.server.chroot_dir).join("ftp-data/text/user.txt");
    let mut template_data = String::new();
    let mut file = fs::File::open(template_path)?;
    file.read_to_string(&mut template_data)?;
    Ok(template_data)
}
