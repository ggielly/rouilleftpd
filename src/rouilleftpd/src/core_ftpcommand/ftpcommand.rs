#[derive(Eq, Hash, PartialEq, Debug)]
pub enum FtpCommand {
    USER,
    PASS,
    QUIT,
    PWD,
    LIST,
    CWD,
    NOOP,
    MKD,
    RMD,
    DELE,
    RNFR,
    RNTO,
    RETR,
    STOR,
    PORT,
    PASV,
    SITE,
    FEAT,
    ALLO,
    SYST,
    TYPE,
    CDUP, //
}

impl FtpCommand {
    pub fn from_str(cmd: &str) -> Option<FtpCommand> {
        match cmd.to_ascii_uppercase().as_str() {
            "USER" => Some(FtpCommand::USER),
            "PASS" => Some(FtpCommand::PASS),
            "QUIT" => Some(FtpCommand::QUIT),
            "PWD" => Some(FtpCommand::PWD),
            "LIST" => Some(FtpCommand::LIST),
            "CWD" => Some(FtpCommand::CWD),
            "NOOP" => Some(FtpCommand::NOOP),
            "MKD" => Some(FtpCommand::MKD),
            "RMD" => Some(FtpCommand::RMD),
            "DELE" => Some(FtpCommand::DELE),
            "RNFR" => Some(FtpCommand::RNFR),
            "RNTO" => Some(FtpCommand::RNTO),
            "RETR" => Some(FtpCommand::RETR),
            "STOR" => Some(FtpCommand::STOR),
            "PORT" => Some(FtpCommand::PORT),
            "PASV" => Some(FtpCommand::PASV),
            "SITE" => Some(FtpCommand::SITE),
            "FEAT" => Some(FtpCommand::FEAT),
            "ALLO" => Some(FtpCommand::ALLO),
            "SYST" => Some(FtpCommand::SYST),
            "TYPE" => Some(FtpCommand::TYPE),
            "CDUP" => Some(FtpCommand::CDUP),
            // Add more commands here !
            _ => None,
        }
    }
}
