//! Lesson 2.2: loops, ranges, early exit, and labels.
//!
//! Rust has three loops: `for` walks a range or collection, `while` repeats
//! while a condition holds, and `loop` runs until an explicit `break`. `loop` is
//! an expression: `break value` makes the whole loop evaluate to `value`. A loop
//! label such as `'rows` lets an inner loop break out of an enclosing one.

fn describe_number(number: u32) -> String {
    let mut candidate = 2;
    // `break <value>` makes the loop evaluate to that value, so this function
    // returns whatever string the loop breaks with.
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
    // `for` moves through each element in turn; integers are `Copy`, so the
    // array stays usable afterwards.
    for value in values {
        println!("value = {value}");
    }

    let mut countdown = 3;
    while countdown > 0 {
        println!("{countdown}...");
        countdown -= 1;
    }

    let mut attempts = 0;
    // Using `loop` as an expression: `break "accepted"` becomes `accepted`.
    let accepted = loop {
        attempts += 1;
        if attempts == 3 {
            break "accepted";
        }
    };
    println!("{accepted} after {attempts} attempts");

    // The `'rows` label lets `break 'rows` exit the outer loop from inside the
    // inner one, stopping both at once.
    'rows: for row in 1..=3 {
        for column in 1..=3 {
            if row * column == 6 {
                println!("first product of 6: {row} × {column}");
                break 'rows;
            }
        }
    }

    for number in 2..=10 {
        // `2..=10` is an inclusive range, so 10 is visited.
        println!("{}", describe_number(number));
    }
}
