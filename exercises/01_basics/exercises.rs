//! Exercises for module 1. Replace each `todo!()` and run the example tests.

pub fn format_profile(_name: &str, _age: u8) -> String {
    todo!("format the name and age")
}

pub fn fahrenheit_to_celsius(_fahrenheit: f64) -> f64 {
    todo!("apply (F - 32) * 5 / 9")
}

pub fn character_count(_text: &str) -> usize {
    todo!("count Unicode scalar values")
}

pub fn rectangle_area(_width: u32, _height: u32) -> u32 {
    todo!("return the product")
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
    }
}
