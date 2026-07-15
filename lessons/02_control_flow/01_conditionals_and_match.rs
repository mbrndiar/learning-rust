//! Lesson 2.1: `if`, `match`, guards, and destructuring.
//!
//! `match` compares a value against patterns and must be exhaustive: every
//! possible case has to be handled, so the compiler rejects a forgotten one.
//! Patterns can destructure tuples, match ranges, combine alternatives with `|`,
//! and add `if` guards. Like `if`, `match` is an expression that yields a value.

fn grade(score: u8) -> &'static str {
    // An `if`/`else if` chain is an expression; the chosen branch is returned.
    if score >= 90 {
        "excellent"
    } else if score >= 60 {
        "pass"
    } else {
        "retry"
    }
}

fn describe_point(point: (i32, i32)) -> &'static str {
    // Arms are tried top to bottom and the first match wins. `_` matches any
    // value, and a guard (`if x == y`) adds a boolean condition to an arm.
    match point {
        (0, 0) => "origin",
        (0, _) => "on the y-axis",
        (_, 0) => "on the x-axis",
        (x, y) if x == y => "on the x=y diagonal",
        _ => "ordinary point", // the catch-all keeps the match exhaustive
    }
}

fn day_kind(day: u8) -> &'static str {
    match day {
        1..=5 => "weekday", // an inclusive range pattern
        6 | 7 => "weekend", // `|` matches either alternative
        _ => "invalid day",
    }
}

fn main() {
    let score = 84;
    let result = grade(score);
    println!("score {score}: {result}");

    for point in [(0, 0), (0, 3), (4, 0), (2, 2), (2, 5)] {
        println!("{point:?}: {}", describe_point(point));
    }

    for day in [1, 5, 6, 8] {
        println!("day {day}: {}", day_kind(day));
    }
}
