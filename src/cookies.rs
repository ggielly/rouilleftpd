use tokio::fs;
use std::collections::HashMap;

// These cookies use a printf-style type indicator, see 'man 3 printf'.
// Types to be used are: %d = decimal, %f = float and %s = string.
// WARNING: Be sure the above types are used correctly with the right cookie.
// Example: FREESPACE     %[%.1f]FMB = 755.5MB
//          EMAIL         %[%-13.13s]E = root@127.0.0.
//          COUNTER       %[%02d]c = 01
//          TRANSFER RATE %[%.2f]AK/s = 150.23K/s

/// General cookie definitions
pub const GENERAL_COOKIES: &[(&str, &str)] = &[
    ("%[%f]A", "Last realtime average transfer rate in KiB/s"),
    ("%[%f]F", "Free space in CWD in MiB"),
    ("%[%f]X", "Free space in CWD in size set by the display_size_unit setting"),
    ("%[%s]E", "Email address, empty if no email setting exists"),
    ("%[%d]M", "Max # of users allowed online"),
    ("%[%s]T", "Current time and date"),
    ("%[%s]D", "Date and time current file was last modified, n/a if no file"),
    ("%[%s]R", "Remote host"),
    ("%[%s]H", "Hostname"),
    ("%[%s]S", "Sitename long format, SITE if no sitename_long setting exists"),
    ("%[%s]s", "Sitename short format, SITE if no sitename_short setting exists"),
    ("%[%d]c", "Counter"),
    ("%[%s]d", "Current working directory, dirname only"),
    ("%[%s]b", "Name of current section (based on pwd)"),
    ("%[%s]a", "Name of requested section (based on user input or related stats)"),
    ("%[%s]P", "Type of TLS connection: ctrl&data|ctrl|none"),
    ("%[%d]B*", "# of users currently logged in (can use any symbol instead of *)"),
    ("%[%d]Bo", "# of users currently logged in excluding the hidden users"),
    ("%[%s]V", "Speed unit to display"),
    ("%[%s]Y", "Size unit to display"),
];

/// User cookie definitions
pub const USER_COOKIES: &[(&str, &str)] = &[
    ("%[%s]Iu", "Username"),
    ("%[%s]I$", "User's comment"),
    ("%[%s]I!", "User's expiring date as YYYY-MM-DD or Never if 0"),
    ("%[%s]I-", "The user that added this user"),
    ("%[%s]Iy", "Groups: long string separated by commas"),
    ("%[%s]Ia", "Added date as YYYY-MM-DD"),
    ("%[%s]Ib", "Banned date as YYYY-MM-DD"),
    ("%[%s]Ix", "Number of logins"),
    ("%[%s]Iv", "Number of logouts"),
    ("%[%s]Iw", "Time since last login as a long string"),
    ("%[%s]It", "User's tagline"),
    ("%[%s]Ip", "User's IP address"),
    ("%[%s]Ig", "User's primary group"),
    ("%[%s]Igx", "User's extended groups"),
];

/// Oneliners cookie definitions
pub const ONELINERS_COOKIES: &[(&str, &str)] = &[
    ("%[%s]ou", "User who added the oneliner"),
    ("%[%s]og", "User's primary group"),
    ("%[%s]ot", "User's tagline"),
    ("%[%s]oD", "Day oneliner was added"),
    ("%[%s]oM", "Month oneliner was added"),
    ("%[%s]oY", "Year oneliner was added"),
    ("%[%s]om", "Message"),
];

/// Lastonline cookie definitions
pub const LASTONLINE_COOKIES: &[(&str, &str)] = &[
    ("%[%s]Lu", "Username"),
    ("%[%s]Lg", "Primary groupname"),
    ("%[%s]Lt", "User's tagline"),
    ("%[%s]Li", "Time of login"),
    ("%[%s]Lo", "Time of logout"),
    ("%[%s]LU", "Amount of bytes uploaded"),
    ("%[%s]LD", "Amount of bytes downloaded"),
    ("%[%s]Ls", "Actions the user did"),
];

/// New directories cookie definitions
pub const NEW_DIRECTORIES_COOKIES: &[(&str, &str)] = &[
    ("%[%s]ND", "Newest directory, absolute path"),
    ("%[%s]Nd", "Newest directory, dirname only"),
    ("%[%s]Na", "Age"),
    ("%[%s]Nm", "Creator"),
    ("%[%s]Ns", "Files + Size"),
];

/// Nuke/Unnuke cookie definitions
pub const NUKE_UNNUKE_COOKIES: &[(&str, &str)] = &[
    ("%[%s]KD", "Nuked/Unnuked directory, absolute path"),
    ("%[%s]Kd", "Nuked/Unnuked directory, dirname only"),
    ("%[%s]Ka", "Age"),
    ("%[%s]Kr", "Reason/Comment"),
    ("%[%s]Kn", "Nukee/Old Nukee"),
    ("%[%s]KN", "Nuker/Unnuker"),
    ("%[%s]Ks", "Multiplier + Size"),
];

/// Multipurpose stats cookie definitions
pub const MULTIPURPOSE_STATS_COOKIES: &[(&str, &str)] = &[
    ("%[%s]Gb", "Amount of bytes"),
    ("%[%s]GN", "Times nuked"),
    ("%[%s]Gs", "Speed in speed unit"),
    ("%[%s]Gn", "Bytes nuked"),
    ("%[%s]Gt", "User's tagline"),
    ("%[%s]Gu", "Username"),
    ("%[%s]Gg", "Primary group"),
    ("%[%s]Gf", "Number of files"),
    ("%[%s]Gp", "Percentage of total bytes"),
    ("%[%s]Gd", "Group description"),
    ("%[%s]Gm", "Number of group members"),
];

/// All cookie definitions combined
pub const COOKIE_DEFINITIONS: &[(&str, &str)] = &[
    // General Cookies
    ("%[%f]A", "Last realtime average transfer rate in KiB/s"),
    ("%[%f]F", "Free space in CWD in MiB"),
    ("%[%f]X", "Free space in CWD in size set by the display_size_unit setting"),
    ("%[%s]E", "Email address, empty if no email setting exists"),
    ("%[%d]M", "Max # of users allowed online"),
    ("%[%s]T", "Current time and date"),
    ("%[%s]D", "Date and time current file was last modified, n/a if no file"),
    ("%[%s]R", "Remote host"),
    ("%[%s]H", "Hostname"),
    ("%[%s]S", "Sitename long format, SITE if no sitename_long setting exists"),
    ("%[%s]s", "Sitename short format, SITE if no sitename_short setting exists"),
    ("%[%d]c", "Counter"),
    ("%[%s]d", "Current working directory, dirname only"),
    ("%[%s]b", "Name of current section (based on pwd)"),
    ("%[%s]a", "Name of requested section (based on user input or related stats)"),
    ("%[%s]P", "Type of TLS connection: ctrl&data|ctrl|none"),
    ("%[%d]B*", "# of users currently logged in (can use any symbol instead of *)"),
    ("%[%d]Bo", "# of users currently logged in excluding the hidden users"),
    ("%[%s]V", "Speed unit to display"),
    ("%[%s]Y", "Size unit to display"),
    // User Cookies
    ("%[%s]Iu", "Username"),
    ("%[%s]I$", "User's comment"),
    ("%[%s]I!", "User's expiring date as YYYY-MM-DD or Never if 0"),
    ("%[%s]I-", "The user that added this user"),
    ("%[%s]Iy", "Groups: long string separated by commas"),
    ("%[%s]Ia", "Added date as YYYY-MM-DD"),
    ("%[%s]Ib", "Banned date as YYYY-MM-DD"),
    ("%[%s]Ix", "Number of logins"),
    ("%[%s]Iv", "Number of logouts"),
    ("%[%s]Iw", "Time since last login as a long string"),
    ("%[%s]It", "User's tagline"),
    ("%[%s]Ip", "User's IP address"),
    ("%[%s]Ig", "User's primary group"),
    ("%[%s]Igx", "User's extended groups"),
    // Oneliners Cookies
    ("%[%s]ou", "User who added the oneliner"),
    ("%[%s]og", "User's primary group"),
    ("%[%s]ot", "User's tagline"),
    ("%[%s]oD", "Day oneliner was added"),
    ("%[%s]oM", "Month oneliner was added"),
    ("%[%s]oY", "Year oneliner was added"),
    ("%[%s]om", "Message"),
    // Lastonline Cookies
    ("%[%s]Lu", "Username"),
    ("%[%s]Lg", "Primary groupname"),
    ("%[%s]Lt", "User's tagline"),
    ("%[%s]Li", "Time of login"),
    ("%[%s]Lo", "Time of logout"),
    ("%[%s]LU", "Amount of bytes uploaded"),
    ("%[%s]LD", "Amount of bytes downloaded"),
    ("%[%s]Ls", "Actions the user did"),
    // New Directories Cookies
    ("%[%s]ND", "Newest directory, absolute path"),
    ("%[%s]Nd", "Newest directory, dirname only"),
    ("%[%s]Na", "Age"),
    ("%[%s]Nm", "Creator"),
    ("%[%s]Ns", "Files + Size"),
    // Nuke/Unnuke Cookies
    ("%[%s]KD", "Nuked/Unnuked directory, absolute path"),
    ("%[%s]Kd", "Nuked/Unnuked directory, dirname only"),
    ("%[%s]Ka", "Age"),
    ("%[%s]Kr", "Reason/Comment"),
    ("%[%s]Kn", "Nukee/Old Nukee"),
    ("%[%s]KN", "Nuker/Unnuker"),
    ("%[%s]Ks", "Multiplier + Size"),
    // Multipurpose stats Cookies
    ("%[%s]Gb", "Amount of bytes"),
    ("%[%s]GN", "Times nuked"),
    ("%[%s]Gs", "Speed in speed unit"),
    ("%[%s]Gn", "Bytes nuked"),
    ("%[%s]Gt", "User's tagline"),
    ("%[%s]Gu", "Username"),
    ("%[%s]Gg", "Primary group"),
    ("%[%s]Gf", "Number of files"),
    ("%[%s]Gp", "Percentage of total bytes"),
    ("%[%s]Gd", "Group description"),
    ("%[%s]Gm", "Number of group members"),
];
