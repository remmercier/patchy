use commands::pr_fetch::Flag;

pub mod backup;
pub mod commands;
pub mod git_commands;
pub mod types;
pub mod utils;

pub static CONFIG_ROOT: &str = ".patchy";
pub static CONFIG_FILE: &str = "config.toml";
pub static APP_NAME: &str = env!("CARGO_PKG_NAME");
pub static INDENT: &str = "  ";

/// Extracts value out of a `flag` which can have an assignment
///
/// # Examples
///
/// ```rust
/// use patchy::extract_value_from_flag;
/// use patchy::commands::pr_fetch::Flag;
///
/// let my_flag = Flag {
///     short: "-r=",
///     long: "--remote-name=",
/// };
///
/// let long_version = extract_value_from_flag("--remote-name=abc", &my_flag);
/// let short_version = extract_value_from_flag("-r=abcdefg", &my_flag);
/// let invalid = extract_value_from_flag("-m=abcdefg", &my_flag);
///
/// assert_eq!(long_version, Some("abc".into()));
/// assert_eq!(short_version, Some("abcdefg".into()));
/// assert_eq!(invalid, None);
/// ```
pub fn extract_value_from_flag(arg: &str, flag: &Flag) -> Option<String> {
    if arg.starts_with(flag.short) {
        arg.get(flag.short.len()..).map(|a| a.into())
    } else if arg.starts_with(flag.long) {
        arg.get(flag.long.len()..).map(|a| a.into())
    } else {
        None
    }
}
