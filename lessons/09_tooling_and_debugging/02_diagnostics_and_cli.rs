//! Lesson 9.2: parse at the boundary and keep domain logic independent.
//!
//! Command-line arguments are untrusted input. Parse them once into a validated
//! struct (`GreetingOptions`), then let the rest of the program work with that
//! type. Separating parsing from behavior makes both easy to test and gives the
//! process one place to report errors and choose an exit code.

#[derive(Debug, PartialEq)]
struct GreetingOptions {
    name: String,
    shout: bool,
}

fn parse_options(arguments: &[String]) -> Result<GreetingOptions, String> {
    let mut name = None;
    let mut shout = false;

    // Classify each argument once. Unknown flags and extra positionals are
    // rejected here rather than deeper in the program.
    for argument in arguments {
        match argument.as_str() {
            "--shout" => shout = true,
            value if value.starts_with('-') => {
                return Err(format!("unknown option: {value}"));
            }
            value if name.is_none() => name = Some(value.to_owned()),
            value => return Err(format!("unexpected argument: {value}")),
        }
    }

    Ok(GreetingOptions {
        name: name.unwrap_or_else(|| String::from("Rustacean")),
        shout,
    })
}

// A pure function of validated options: no parsing, no I/O, easy to test.
fn build_greeting(options: &GreetingOptions) -> String {
    let message = format!("Hello, {}!", options.name);
    if options.shout {
        message.to_uppercase()
    } else {
        message
    }
}

fn run(arguments: &[String]) -> Result<String, String> {
    let options = parse_options(arguments)?;
    Ok(build_greeting(&options))
}

fn main() {
    // `args()` includes the executable path at index 0, so skip it.
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("usage: lesson-09-diagnostics-cli [NAME] [--shout]");
            // A non-zero exit code reports failure to the shell and other tools.
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_name_and_flag_in_either_order() {
        let first = vec![String::from("Ada"), String::from("--shout")];
        let second = vec![String::from("--shout"), String::from("Ada")];
        assert_eq!(parse_options(&first), parse_options(&second));
    }

    #[test]
    fn rejects_unknown_flags() {
        let arguments = vec![String::from("--loud")];
        assert!(parse_options(&arguments).is_err());
    }
}
