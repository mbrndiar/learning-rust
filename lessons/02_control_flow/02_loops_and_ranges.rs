//! Lesson 2.2: loops, ranges, early exit, and labels.

fn first_divisor(number: u32) -> Option<u32> {
    (2..number).find(|candidate| number % candidate == 0)
}

fn main() {
    let values = [10, 20, 30];
    for (index, value) in values.iter().enumerate() {
        println!("values[{index}] = {value}");
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
        match first_divisor(number) {
            Some(divisor) => println!("{number} is divisible by {divisor}"),
            None => println!("{number} is prime"),
        }
    }
}
