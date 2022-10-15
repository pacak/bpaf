#[cfg(feature = "bright-color")]
macro_rules! w_section {
    ($buf:ident, $pat:literal) => {
        write!(
            $buf,
            "{}",
            ::owo_colors::OwoColorize::if_supports_color(
                &$pat,
                ::owo_colors::Stream::Stdout,
                |s| ::owo_colors::OwoColorize::style(s, ::owo_colors::Style::new().yellow().bold())
            )
        )
    };
}

#[cfg(not(feature = "bright-color"))]
macro_rules! w_section {
    ($buf:ident, $pat:literal) => {
        write!(
            $buf,
            "{}",
            ::owo_colors::OwoColorize::if_supports_color(
                &$pat,
                ::owo_colors::Stream::Stdout,
                |s| ::owo_colors::OwoColorize::style(
                    s,
                    ::owo_colors::Style::new().bold().underline()
                )
            )
        )
    };
}

#[cfg(not(feature = "bright-color"))]
macro_rules! w_err {
    ($item:expr) => {
        ::owo_colors::OwoColorize::if_supports_color(
            &$item,
            ::owo_colors::Stream::Stdout,
            ::owo_colors::OwoColorize::bold,
        )
    };
}

#[cfg(feature = "bright-color")]
macro_rules! w_err {
    ($item:expr) => {
        ::owo_colors::OwoColorize::if_supports_color(&$item, ::owo_colors::Stream::Stdout, |s| {
            ::owo_colors::OwoColorize::style(s, ::owo_colors::Style::new().red().bold())
        })
    };
}

#[cfg(feature = "bright-color")]
macro_rules! w_flag {
    ($item:expr) => {
        ::owo_colors::OwoColorize::if_supports_color(&$item, ::owo_colors::Stream::Stdout, |s| {
            ::owo_colors::OwoColorize::style(s, ::owo_colors::Style::new().green().bold())
        })
    };
}

#[cfg(not(feature = "bright-color"))]
macro_rules! w_flag {
    ($item:expr) => {
        ::owo_colors::OwoColorize::if_supports_color(
            &$item,
            ::owo_colors::Stream::Stdout,
            ::owo_colors::OwoColorize::bold,
        )
    };
}

#[doc(hidden)]
pub use owo_colors::set_override;
