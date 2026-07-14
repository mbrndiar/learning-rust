//! Lesson 2.1: `if`, `match`, guards, and destructuring.

fn grade(score: u8) -> &'static str {
    if score >= 90 {
        "excellent"
    } else if score >= 60 {
        "pass"
    } else {
        "retry"
    }
}

fn describe_point(point: (i32, i32)) -> &'static str {
    match point {
        (0, 0) => "origin",
        (0, _) => "on the y-axis",
        (_, 0) => "on the x-axis",
        (x, y) if x == y => "on the x=y diagonal",
        _ => "ordinary point",
    }
}

fn day_kind(day: u8) -> &'static str {
    match day {
        1..=5 => "weekday",
        6 | 7 => "weekend",
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
