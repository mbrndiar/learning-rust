//! Reference solution for module 8.

#[derive(Debug, PartialEq)]
enum MathError {
    DivisionByZero,
    Overflow,
}

fn slugify(text: &str) -> String {
    text.split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}

fn divide(numerator: i32, denominator: i32) -> Result<i32, MathError> {
    if denominator == 0 {
        Err(MathError::DivisionByZero)
    } else {
        numerator
            .checked_div(denominator)
            .ok_or(MathError::Overflow)
    }
}

fn main() {
    assert_eq!(slugify("  Hello   Rust  "), "hello-rust");
    assert_eq!(slugify(""), "");
    assert_eq!(divide(12, 3), Ok(4));
    assert_eq!(divide(12, 0), Err(MathError::DivisionByZero));
    assert_eq!(divide(i32::MIN, -1), Err(MathError::Overflow));
    println!("Module 8 solutions passed.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_normalized_slug() {
        assert_eq!(slugify("  Hello   Rust  "), "hello-rust");
    }

    #[test]
    fn empty_input_produces_an_empty_slug() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn divides_whole_numbers() {
        assert_eq!(divide(12, 3), Ok(4));
    }

    #[test]
    fn division_by_zero_is_typed_failure() {
        assert_eq!(divide(12, 0), Err(MathError::DivisionByZero));
    }

    #[test]
    fn integer_division_overflow_is_typed_failure() {
        assert_eq!(divide(i32::MIN, -1), Err(MathError::Overflow));
    }
}
