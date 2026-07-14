//! Exercises for module 2.

pub fn number_kind(_value: i32) -> &'static str {
    todo!("return negative, zero, or positive")
}

pub fn fizz_buzz(_value: u32) -> String {
    todo!("return FizzBuzz, Fizz, Buzz, or the number")
}

pub fn sum_until_limit(_values: &[u32], _limit: u32) -> u32 {
    todo!("stop before the total would exceed limit")
}

pub fn first_multiple(_values: &[u32], _divisor: u32) -> Option<u32> {
    todo!("find the first divisible value; divisor zero returns None")
}

fn main() {
    println!("Run `cargo test --example ex-02-control-flow` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_numbers() {
        assert_eq!(number_kind(-2), "negative");
        assert_eq!(number_kind(0), "zero");
        assert_eq!(number_kind(2), "positive");
    }

    #[test]
    fn applies_fizz_buzz_rules_in_order() {
        assert_eq!(fizz_buzz(15), "FizzBuzz");
        assert_eq!(fizz_buzz(9), "Fizz");
        assert_eq!(fizz_buzz(10), "Buzz");
        assert_eq!(fizz_buzz(7), "7");
    }

    #[test]
    fn stops_before_limit_is_exceeded() {
        assert_eq!(sum_until_limit(&[3, 4, 5], 8), 7);
        assert_eq!(sum_until_limit(&[], 10), 0);
    }

    #[test]
    fn finds_a_multiple_safely() {
        assert_eq!(first_multiple(&[5, 8, 12], 4), Some(8));
        assert_eq!(first_multiple(&[1, 3], 2), None);
        assert_eq!(first_multiple(&[0], 0), None);
    }
}
