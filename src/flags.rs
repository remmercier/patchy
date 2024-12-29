use colored::Colorize;
use indexmap::IndexSet;

use crate::commands::help::format_description;

pub struct Flag<'a> {
    pub short: &'a str,
    pub long: &'a str,
    pub description: &'a str,
}

/// Extracts value out of a `flag` which can have an assignment
///
/// # Examples
///
/// ```rust
/// use patchy::flags::{extract_value_from_flag, Flag};
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

pub fn contains_flag(args: &IndexSet<String>, flag: &Flag) -> bool {
    args.contains(flag.short) || args.contains(flag.long)
}

pub fn is_valid_flag(arg: &str, available_flags: &[&Flag]) -> bool {
    available_flags
        .iter()
        .flat_map(|flag| [flag.short, flag.long])
        .any(|flag| arg.starts_with(flag))
}

/// Formats a flag into a colored format with a description, printable to the terminal
pub fn format_flag(flag: &Flag) -> String {
    format!(
        "{}{}{}\n    {}",
        flag.short.magenta(),
        ", ".black(),
        flag.long.magenta(),
        format_description(flag.description)
    )
}
