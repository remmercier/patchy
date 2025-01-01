use std::env;

use colored::Colorize;
use indexmap::IndexSet;
use once_cell::sync::Lazy;

use crate::{commands::help::format_description, types::CommandArgs};

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
        arg.get(flag.short.len()..).map(|value| value.into())
    } else if arg.starts_with(flag.long) {
        arg.get(flag.long.len()..).map(|value| value.into())
    } else {
        None
    }
}

pub fn contains_flag(args: &IndexSet<String>, flag: &Flag) -> bool {
    args.contains(flag.short) || args.contains(flag.long)
}

/// Checks whether an input argument is a valid flag
pub fn is_valid_flag(arg: &str, available_flags: &[&Flag]) -> bool {
    // TODO: flags that don't end in "=" should be compared fully, not just the beginning
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

pub static IS_VERBOSE: Lazy<bool> = Lazy::new(|| {
    let args: CommandArgs = env::args().collect();
    args.contains("--verbose")
});
