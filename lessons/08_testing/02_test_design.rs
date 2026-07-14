//! Lesson 8.2: deterministic dependencies and behavior-focused tests.

trait Clock {
    fn current_hour(&self) -> u8;
}

struct FixedClock {
    hour: u8,
}

impl Clock for FixedClock {
    fn current_hour(&self) -> u8 {
        self.hour
    }
}

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
            assert_eq!(greeting(&clock, "Ada"), expected, "hour={hour}");
        }
    }

    #[test]
    fn preserves_the_supplied_name() {
        let clock = FixedClock { hour: 8 };
        assert!(greeting(&clock, "Grace").ends_with("Grace!"));
    }
}
