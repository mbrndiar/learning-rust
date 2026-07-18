//! Lesson 1.2: bindings, constants, shadowing, core types, and numeric bounds.
//!
//! Bindings are immutable by default; `mut` opts into mutation. `const` names a
//! compile-time value and always needs a type annotation. Shadowing reuses a
//! name with a fresh binding (and possibly a new type), which differs from
//! mutating one binding in place. Rust infers most types; annotate for clarity.
//! Fixed-width integers require an explicit overflow policy at real boundaries.

// `const` values are computed at compile time and must be annotated with a type.
// The convention is to name them in SCREAMING_SNAKE_CASE.
const SECONDS_PER_MINUTE: u32 = 60;

fn main() {
    let course = "Rust foundations";
    // `mut` is required to change a binding after creation; without it the `+=`
    // below would not compile. The type stays `u8` across the mutation.
    let mut completed_lessons: u8 = 1;
    completed_lessons += 1;

    // Shadowing introduces a brand-new binding that reuses the name. Unlike
    // mutation, the replacement may have a different type (`&str` -> `usize`).
    let spaces = "   ";
    let spaces = spaces.len();

    let signed: i32 = -42;
    let unsigned = 42_u64; // the `_u64` suffix pins the literal's concrete type
    let ratio: f64 = 3.0 / 2.0;
    let ready = true;
    let crab = '🦀'; // a `char` is one Unicode scalar value, not a single byte

    let point: (i32, i32) = (3, 4); // a tuple groups a fixed set of types
    let [first, second, third] = [10, 20, 30]; // destructure a fixed-size array

    println!("{course}: {completed_lessons} lessons completed");
    println!("{spaces} spaces were measured");
    println!("numbers: {signed}, {unsigned}, {ratio}");
    println!("ready={ready}, mascot={crab}");
    println!("point=({}, {})", point.0, point.1);
    println!("array values: {first}, {second}, {third}");
    println!("two minutes = {} seconds", 2 * SECONDS_PER_MINUTE);

    // State overflow behavior explicitly instead of relying on build-profile
    // checks. Module 4 explains the `Option` returned by `checked_add` in depth.
    println!("checked overflow={:?}", u8::MAX.checked_add(1));
    println!("saturating overflow={}", u8::MAX.saturating_add(1));
    println!("wrapping overflow={}", u8::MAX.wrapping_add(1));
    let (wrapped, overflowed) = u8::MAX.overflowing_add(1);
    println!("overflowing result={wrapped}, overflowed={overflowed}");

    // Binary floating point does not represent every decimal exactly. It also
    // includes non-finite values that many external formats deliberately reject.
    let decimal_sum = 0.1_f64 + 0.2;
    println!(
        "0.1 + 0.2 is close to 0.3: {}",
        (decimal_sum - 0.3).abs() < f64::EPSILON
    );
    println!("NaN is finite: {}", f64::NAN.is_finite());
}
