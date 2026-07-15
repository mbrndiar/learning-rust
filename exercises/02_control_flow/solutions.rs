//! Reference solutions for module 2.

fn number_kind(value: i32) -> &'static str {
    // Open-ended range patterns partition all of `i32`, so the match is
    // exhaustive without a wildcard arm.
    match value {
        ..=-1 => "negative",
        0 => "zero",
        1.. => "positive",
    }
}

fn fizz_buzz(value: u32) -> String {
    match (value % 3 == 0, value % 5 == 0) {
        (true, true) => String::from("FizzBuzz"),
        (true, false) => String::from("Fizz"),
        (false, true) => String::from("Buzz"),
        (false, false) => value.to_string(),
    }
}

fn sum_until_limit(values: &[u32], limit: u32) -> u32 {
    let mut total: u32 = 0;
    for value in values {
        // `checked_add` returns `None` on overflow, so we stop cleanly instead
        // of panicking on a wraparound.
        let Some(next) = total.checked_add(*value) else {
            break;
        };
        if next > limit {
            break;
        }
        total = next;
    }
    total
}

fn first_multiple(values: &[u32], divisor: u32) -> Option<u32> {
    if divisor == 0 {
        return None;
    }
    values.iter().copied().find(|value| value % divisor == 0)
}

fn main() {
    assert_eq!(number_kind(-2), "negative");
    assert_eq!(fizz_buzz(15), "FizzBuzz");
    assert_eq!(sum_until_limit(&[3, 4, 5], 8), 7);
    assert_eq!(first_multiple(&[5, 8, 12], 4), Some(8));
    println!("Module 2 solutions passed.");
}
