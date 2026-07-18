//! Reference solutions for module 1.

fn format_profile(name: &str, age: u8) -> String {
    format!("{name} is {age} years old")
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

fn character_count(text: &str) -> usize {
    text.chars().count()
}

fn rectangle_area(width: u32, height: u32) -> u64 {
    u64::from(width) * u64::from(height)
}

fn capped_progress(current: u8, completed: u8) -> u8 {
    current.saturating_add(completed)
}

fn main() {
    assert_eq!(format_profile("Ada", 36), "Ada is 36 years old");
    assert!((fahrenheit_to_celsius(212.0) - 100.0).abs() < f64::EPSILON);
    assert_eq!(character_count("Rust 🦀"), 6);
    assert_eq!(rectangle_area(4, 3), 12);
    assert_eq!(
        rectangle_area(u32::MAX, u32::MAX),
        u64::from(u32::MAX).pow(2)
    );
    assert_eq!(capped_progress(250, 10), u8::MAX);
    println!("Module 1 solutions passed.");
}
