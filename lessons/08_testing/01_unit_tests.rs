//! Lesson 8.1: unit tests, result assertions, and test-only modules.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_percentage_discount() {
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
