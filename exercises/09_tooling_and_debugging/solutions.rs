//! Reference solutions for module 9.

#[derive(Debug, PartialEq, Eq)]
struct Options {
    input: String,
    verbose: bool,
}

fn parse_options(arguments: &[&str]) -> Result<Options, String> {
    let mut input = None;
    let mut verbose = false;

    for argument in arguments {
        match *argument {
            "--verbose" => verbose = true,
            value if value.starts_with('-') => return Err(format!("unknown option: {value}")),
            value if input.is_none() => input = Some(value.to_owned()),
            value => return Err(format!("unexpected argument: {value}")),
        }
    }

    Ok(Options {
        input: input.unwrap_or_else(|| String::from(".")),
        verbose,
    })
}

fn build_summary(options: &Options, item_count: usize) -> String {
    if options.verbose {
        format!("{} contains {item_count} items", options.input)
    } else {
        item_count.to_string()
    }
}

fn main() {
    let options = parse_options(&["src", "--verbose"]).expect("valid options");
    assert_eq!(build_summary(&options, 4), "src contains 4 items");
    assert!(parse_options(&["--loud"]).is_err());
    println!("Module 9 solutions passed.");
}
