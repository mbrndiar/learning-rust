//! Lesson 1.2: bindings, constants, shadowing, and core types.

const SECONDS_PER_MINUTE: u32 = 60;

fn main() {
    let course = "Rust foundations";
    let mut completed_lessons: u8 = 1;
    completed_lessons += 1;

    // Shadowing creates a new binding. The new binding may have another type.
    let spaces = "   ";
    let spaces = spaces.len();

    let signed: i32 = -42;
    let unsigned = 42_u64;
    let ratio: f64 = 3.0 / 2.0;
    let ready = true;
    let crab = '🦀';

    let point: (i32, i32) = (3, 4);
    let [first, second, third] = [10, 20, 30];

    println!("{course}: {completed_lessons} lessons completed");
    println!("{spaces} spaces were measured");
    println!("numbers: {signed}, {unsigned}, {ratio}");
    println!("ready={ready}, mascot={crab}");
    println!("point=({}, {})", point.0, point.1);
    println!("array values: {first}, {second}, {third}");
    println!("two minutes = {} seconds", 2 * SECONDS_PER_MINUTE);
}
