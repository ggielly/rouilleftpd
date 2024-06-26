// src/constants.rs

pub const USERNAME_REGEX: &str = r"^[a-zA-Z0-9]{1,32}$";
pub const IP_HOSTNAME_MAX_LENGTH: usize = 128;

// Constants specific to the `site addip` command
pub const MIN_ADDIP_ARGS: usize = 2;
pub const MAX_ADDIP_IPS: usize = 10;

// Constants specific to the `site delip` command
pub const MIN_DELIP_ARGS: usize = 2;
pub const MAX_DELIP_IPS: usize = 10;


/*
  Flagname       	Flag	Description
    ------------------------------------------------------------------------
    SITEOP		1	Siteoperator, has access to most commands.
    GADMIN		2	Groupadmin of at least one of the user's public
                    groups,	private groups have no such thing.
                    Use 'site chgadmin' to give or take this flag.
    GLOCK		3	GroupLock : user cannot change group.
    EXEMPT		4	By default allows to log in when site is full,
                    to do 'site idle 0' (same as IDLER flag) and
                    exempts from the sim_xfers limit in config file.
    COLOR		5	Enables having colors in listings and other
                    serverreplies. Can also be changed via the
                    'site color' command.
    DELETED		6	User is deleted.
                    Use 'site deluser' to give or 'site readd' to
                    take this flag.
    USEREDIT	7	"Co-Siteop"
    ANONYMOUS	8	Anonymous user (per-session like login).
                    No DELE, RMD or RNTO/RNFR; '!' on login is
                    ignored and the userfile doesn't get updated
                    with stats (serves as template - use external
                    scripts if transfer stats must be recorded).
    NUKE		A	Allows to use 'site nuke/reqlog'.
    UNNUKE		B	Allows to use 'site unnuke/reqlog'.
    UNDUPE		C	Allows to use 'site undupe/predupe'.
    KICK		D	Allows to use 'site kick'.
    KILL		E	Allows to use 'site kill/swho'.
    TAKE		F	Allows to use 'site take'.
    GIVE		G	Allows to use 'site give'.
    USERS		H	Allows to use 'site user/users/ginfo'.
    IDLER		I	Allows to idle forever.
    CUSTOM1		J	Custom flag 1
    CUSTOM2		K	Custom flag 2
    CUSTOM3		L	Custom flag 3
    CUSTOM4		M	Custom flag 4
    CUSTOM5		N	Custom flag 5

    Custom flags can be used in the config file to give some users access to
    certain things without having to use private groups. These flags will
    only show up in 'site flags' if they're turned on. Custom flags up to
    'Z' can be used.

    Note:   flag 1 is not GOD mode, you must have the correct flags for the
            actions you wish to perform.
    Note:   a user with flag 1 DOES NOT WANT flag 2.
    Note:   flags A-H can have their access changed through the -flag
            permissions in the config file.
*/
pub const SITEOP: u8 = 1;
pub const GADMIN: u8 = 2;
pub const GLOCK: u8 = 3;
pub const EXEMPT: u8 = 4;
pub const COLOR: u8 = 5;
pub const DELETED: u8 = 6;
pub const USEREDIT: u8 = 7;
pub const ANONYMOUS: u8 = 8;
pub const NUKE: u8 = 10;
pub const UNNUKE: u8 = 11;
pub const UNDUPE: u8 = 12;
pub const KICK: u8 = 13;
pub const KILL: u8 = 14;
pub const TAKE: u8 = 15;
pub const GIVE: u8 = 16;
pub const USERS: u8 = 17;
pub const IDLER: u8 = 18;
pub const CUSTOM1: u8 = 19;
pub const CUSTOM2: u8 = 20;
pub const CUSTOM3: u8 = 21;
pub const CUSTOM4: u8 = 22;
pub const CUSTOM5: u8 = 23;
// End of flags

pub const STATLINE_PATH: &str = "ftp-data/text/statline.txt";
