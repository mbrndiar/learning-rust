//! Exercises for module 1: bindings, types, text, and functions.
//!
//! Implement each function by replacing its `todo!()` body, then run the example
//! tests. Do not change any signature; the tests and reference solution rely on
//! them.

/// Format a person's name and age as `"{name} is {age} years old"`.
///
/// Borrows `name` read-only and returns a newly allocated `String`.
pub fn format_profile(_name: &str, _age: u8) -> String {
    todo!("format the name and age")
}

/// Convert a Fahrenheit temperature to Celsius using `(F - 32) * 5 / 9`.
///
/// Works in floating point, so callers should compare results with a tolerance
/// rather than `==`.
pub fn fahrenheit_to_celsius(_fahrenheit: f64) -> f64 {
    todo!("apply (F - 32) * 5 / 9")
}

/// Count the Unicode scalar values in `text`, not its bytes.
///
/// For non-ASCII input this differs from `text.len()` (the byte count).
pub fn character_count(_text: &str) -> usize {
    todo!("count Unicode scalar values")
}

/// Return the rectangle area after losslessly widening both inputs to `u64`.
pub fn rectangle_area(_width: u32, _height: u32) -> u64 {
    todo!("widen both inputs before multiplying")
}

/// Add completed lessons, stopping at `u8::MAX` instead of overflowing.
pub fn capped_progress(_current: u8, _completed: u8) -> u8 {
    todo!("add with saturating arithmetic")
}

fn main() {
    println!("Run `cargo test --example ex-01-basics` after replacing every todo!().");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_profile() {
        assert_eq!(format_profile("Ada", 36), "Ada is 36 years old");
    }

    #[test]
    fn converts_temperature() {
        assert!((fahrenheit_to_celsius(32.0) - 0.0).abs() < f64::EPSILON);
        assert!((fahrenheit_to_celsius(212.0) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn counts_characters_not_bytes() {
        assert_eq!(character_count("Rust 🦀"), 6);
    }

    #[test]
    fn calculates_area() {
        assert_eq!(rectangle_area(4, 3), 12);
        assert_eq!(
            rectangle_area(u32::MAX, u32::MAX),
            u64::from(u32::MAX).pow(2)
        );
    }

    #[test]
    fn caps_progress_at_the_numeric_boundary() {
        assert_eq!(capped_progress(250, 10), u8::MAX);
    }
}
