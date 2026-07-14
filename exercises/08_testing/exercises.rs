//! Exercises for module 8. Replace `todo!()` in the tests, not the implementation.

#[derive(Debug, PartialEq)]
pub enum MathError {
    DivisionByZero,
}

pub fn slugify(text: &str) -> String {
    text.split_whitespace()
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}

pub fn divide(numerator: i32, denominator: i32) -> Result<i32, MathError> {
    if denominator == 0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(numerator / denominator)
    }
}

fn main() {
    println!("Replace the todo!() calls in tests, then run `cargo test --example ex-08-testing`.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_normalized_slug() {
        todo!("assert spaces collapse and letters become lowercase")
    }

    #[test]
    fn empty_input_produces_an_empty_slug() {
        todo!("assert the boundary case")
    }

    #[test]
    fn divides_whole_numbers() {
        todo!("assert a successful Result")
    }

    #[test]
    fn division_by_zero_is_typed_failure() {
        todo!("assert the exact MathError variant")
    }
}
