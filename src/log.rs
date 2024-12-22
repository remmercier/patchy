#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        format!("{}{}", "Error: ".bright_red().bold(), format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        format!("{}{}", "✓ ".bright_green().bold(), format!($($arg)*))
    }};
}
