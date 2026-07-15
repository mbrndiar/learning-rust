//! Exercises for module 9: parsing CLI arguments at the boundary.
//!
//! Implement each `todo!()` body, then run the example tests. Do not change any
//! signature.

/// Parsed command-line options: an input path and a verbosity flag.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub input: String,
    pub verbose: bool,
}

/// Parse one optional positional path plus an optional `--verbose` flag.
///
/// The path defaults to `"."` when omitted. Returns `Err` for an unknown flag or
/// a second positional argument.
pub fn parse_options(_arguments: &[&str]) -> Result<Options, String> {
    todo!("parse one optional path plus --verbose")
}

/// Render a summary from already-parsed options.
///
/// In verbose mode returns `"{input} contains {n} items"`; otherwise just the
/// count. This is pure formatting, kept separate from parsing.
pub fn build_summary(options: &Options, item_count: usize) -> String {
    if options.verbose {
        format!("{} contains {item_count} items", options.input)
    } else {
        item_count.to_string()
    }
}

fn main() {
    println!("Run `cargo test --example ex-09-tooling` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_and_explicit_values() {
        assert_eq!(
            parse_options(&[]),
            Ok(Options {
                input: String::from("."),
                verbose: false,
            })
        );
        assert_eq!(
            parse_options(&["src", "--verbose"]),
            Ok(Options {
                input: String::from("src"),
                verbose: true,
            })
        );
    }

    #[test]
    fn rejects_unknown_or_extra_arguments() {
        assert!(parse_options(&["--loud"]).is_err());
        assert!(parse_options(&["src", "tests"]).is_err());
    }

    #[test]
    fn keeps_formatting_separate_from_parsing() {
        let options = Options {
            input: String::from("src"),
            verbose: true,
        };
        assert_eq!(build_summary(&options, 4), "src contains 4 items");
    }
}
