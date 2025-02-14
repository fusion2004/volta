//! The view layer of Volta, with utilities for styling command-line output.
use archive::Origin;
use cfg_if::cfg_if;
use console::{style, StyledObject};
use failure::Fail;
use indicatif::{ProgressBar, ProgressStyle};
use term_size;

const MAX_WIDTH: usize = 100;
const MAX_PROGRESS_WIDTH: usize = 40;

/// Generate the styled prefix for a success message
pub(crate) fn success_prefix() -> StyledObject<&'static str> {
    style("success:").green().bold()
}

/// Format the underlying cause of an error
pub(crate) fn format_error_cause(inner: &dyn Fail) -> String {
    format!(
        "{}{} {}",
        style("Error cause").underlined().bold(),
        style(":").bold(),
        inner
    )
}

/// Determines the string to display based on the Origin of the operation.
fn action_str(origin: Origin) -> &'static str {
    match origin {
        Origin::Local => "Unpacking",
        Origin::Remote => "Fetching",
    }
}

pub fn tool_version<N, V>(name: N, version: V) -> String
where
    N: std::fmt::Display + Sized,
    V: std::fmt::Display + Sized,
{
    format!("{:}@{:}", name, version)
}

/// Get the width of the terminal, limited to a maximum of MAX_WIDTH
pub fn text_width() -> Option<usize> {
    term_size::dimensions().map(|(w, _)| w.min(MAX_WIDTH))
}

/// Constructs a command-line progress bar based on the specified Origin enum
/// (e.g., `Origin::Remote`), details string (e.g., `"v1.23.4"`), and logical
/// length (i.e., the number of logical progress steps in the process being
/// visualized by the progress bar).
pub fn progress_bar(origin: Origin, details: &str, len: u64) -> ProgressBar {
    let action = action_str(origin);
    let action_width = action.len() + 2; // plus 2 spaces to look nice
    let msg_width = action_width + 1 + details.len();

    //   Fetching node@9.11.2  [=============>                          ]  34%
    // |--------| |---------|   |--------------------------------------|  |-|
    //    action    details                      bar                 percentage
    let bar_width = match text_width() {
        Some(width) => MAX_PROGRESS_WIDTH.min(width - 2 - msg_width - 2 - 2 - 1 - 3 - 1),
        None => MAX_PROGRESS_WIDTH,
    };

    let bar = ProgressBar::new(len);

    bar.set_message(&format!(
        "{: >width$} {}",
        style(action).green().bold(),
        details,
        width = action_width,
    ));
    bar.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{msg}}  [{{bar:{}.cyan/blue}}] {{percent:>3}}%",
                bar_width
            ))
            .progress_chars("=> "),
    );

    bar
}

cfg_if! {
    if #[cfg(windows)] {
        /// Constructs a command-line progress spinner with the specified "message"
        /// string. The spinner is ticked by default every 100ms.
        pub fn progress_spinner(message: &str) -> ProgressBar {
            let spinner = ProgressBar::new_spinner();
            // Windows CMD prompt doesn't support Unicode characters, so use a simplified spinner
            let style = ProgressStyle::default_spinner().tick_chars(r#"-\|/-"#);

            spinner.set_message(message);
            spinner.set_style(style);
            spinner.enable_steady_tick(100);

            spinner
        }
    } else {
        /// Constructs a command-line progress spinner with the specified "message"
        /// string. The spinner is ticked by default every 50ms.
        pub fn progress_spinner(message: &str) -> ProgressBar {
            // ⠋ Fetching public registry: https://nodejs.org/dist/index.json
            let spinner = ProgressBar::new_spinner();

            spinner.set_message(message);
            spinner.set_style(ProgressStyle::default_spinner());
            spinner.enable_steady_tick(50);

            spinner
        }
    }
}
