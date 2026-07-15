//! Lesson 8.2: deterministic dependencies and behavior-focused tests.
//!
//! Depending on a trait (`Clock`) instead of the real wall clock lets a test
//! inject a fixed value and assert behavior deterministically. A table of
//! (input, expected) cases keeps one test covering many boundaries, and
//! asserting on observable output keeps tests robust to internal refactoring.

// Abstracting the time source behind a trait lets tests substitute a fake clock.
trait Clock {
    fn current_hour(&self) -> u8;
}

// A deterministic clock for tests: it always reports the same hour.
struct FixedClock {
    hour: u8,
}

impl Clock for FixedClock {
    fn current_hour(&self) -> u8 {
        self.hour
    }
}

// Taking `&impl Clock` decouples this logic from the real system clock.
fn greeting(clock: &impl Clock, name: &str) -> String {
    let period = match clock.current_hour() {
        0..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    };
    format!("{period}, {name}!")
}

fn main() {
    let clock = FixedClock { hour: 9 };
    println!("{}", greeting(&clock, "Ada"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_period_from_injected_time() {
        // A table of (hour, expected) cases exercises every boundary in one test.
        let cases = [
            (0, "Good morning, Ada!"),
            (11, "Good morning, Ada!"),
            (12, "Good afternoon, Ada!"),
            (17, "Good afternoon, Ada!"),
            (18, "Good evening, Ada!"),
            (23, "Good evening, Ada!"),
        ];

        for (hour, expected) in cases {
            let clock = FixedClock { hour };
            // The trailing argument adds context to the panic message on failure.
            assert_eq!(greeting(&clock, "Ada"), expected, "hour={hour}");
        }
    }

    #[test]
    fn preserves_the_supplied_name() {
        let clock = FixedClock { hour: 8 };
        assert!(greeting(&clock, "Grace").ends_with("Grace!"));
    }
}
