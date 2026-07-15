//! Lesson 2.2: loops, ranges, early exit, and labels.

fn describe_number(number: u32) -> String {
    let mut candidate = 2;
    loop {
        if candidate >= number {
            break format!("{number} is prime");
        }
        if number % candidate == 0 {
            break format!("{number} is divisible by {candidate}");
        }
        candidate += 1;
    }
}

fn main() {
    let values = [10, 20, 30];
    for value in values {
        println!("value = {value}");
    }

    let mut countdown = 3;
    while countdown > 0 {
        println!("{countdown}...");
        countdown -= 1;
    }

    let mut attempts = 0;
    let accepted = loop {
        attempts += 1;
        if attempts == 3 {
            break "accepted";
        }
    };
    println!("{accepted} after {attempts} attempts");

    'rows: for row in 1..=3 {
        for column in 1..=3 {
            if row * column == 6 {
                println!("first product of 6: {row} × {column}");
                break 'rows;
            }
        }
    }

    for number in 2..=10 {
        println!("{}", describe_number(number));
    }
}
