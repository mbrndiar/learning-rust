//! Lesson 8.1: unit tests, result assertions, and test-only modules.
//!
//! Unit tests live beside the code in a `#[cfg(test)]` module that is compiled
//! only for `cargo test`, so it adds nothing to the shipped binary. `use
//! super::*;` pulls the parent module into scope. Each `#[test]` fails when an
//! assertion does not hold; comparing whole `Result` values checks both arms.

#[derive(Debug, PartialEq)]
enum DiscountError {
    NegativePrice,
    PercentageOutOfRange,
}

fn discounted_price(price_cents: i64, percentage: u8) -> Result<i64, DiscountError> {
    if price_cents < 0 {
        return Err(DiscountError::NegativePrice);
    }
    if percentage > 100 {
        return Err(DiscountError::PercentageOutOfRange);
    }
    Ok(price_cents * i64::from(100 - percentage) / 100)
}

fn main() {
    println!("20% off 2500 cents = {:?}", discounted_price(2_500, 20));
}

// `#[cfg(test)]` compiles this module only during `cargo test`.
#[cfg(test)]
mod tests {
    use super::*; // bring the parent module's items into scope

    #[test]
    fn applies_percentage_discount() {
        // Comparing whole `Result` values checks the variant and its payload.
        assert_eq!(discounted_price(2_500, 20), Ok(2_000));
    }

    #[test]
    fn handles_boundaries() {
        assert_eq!(discounted_price(999, 0), Ok(999));
        assert_eq!(discounted_price(999, 100), Ok(0));
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert_eq!(discounted_price(-1, 10), Err(DiscountError::NegativePrice));
        assert_eq!(
            discounted_price(100, 101),
            Err(DiscountError::PercentageOutOfRange)
        );
    }
}
