//! Lesson 5.2: iterator ownership modes, lazy adapters, and closures.

fn normalized_scores(scores: &[i32]) -> Vec<i32> {
    scores
        .iter() // yields &i32 because the function borrowed the slice
        .copied() // turns each &i32 into i32; integers implement Copy
        .filter(|score| *score >= 0)
        .map(|score| score.clamp(0, 100))
        .collect() // consumes the lazy pipeline and builds Vec<i32>
}

fn apply_twice(mut value: String, mut operation: impl FnMut(String) -> String) -> String {
    value = operation(value);
    operation(value)
}

fn main() {
    let scores = vec![82, -1, 105, 91, 40];
    let normalized = normalized_scores(&scores);
    println!("original={scores:?}");
    println!("normalized={normalized:?}");

    let passing_total: i32 = normalized.iter().filter(|score| **score >= 60).sum();
    println!("sum of passing scores={passing_total}");

    let mut offset = 0;
    let adjusted: Vec<_> = normalized
        .iter()
        .map(|score| {
            offset += 1;
            score + offset
        })
        .collect();
    println!("stateful closure result={adjusted:?}");

    let suffix = String::from("!");
    // `move` transfers `suffix` into the closure so the closure can outlive this
    // local binding. It does not make the captured String copyable.
    let add_suffix = move |mut text: String| {
        text.push_str(&suffix);
        text
    };
    println!("{}", apply_twice(String::from("Rust"), add_suffix));

    let owned_words = vec![String::from("ownership"), String::from("borrowing")];
    for word in owned_words {
        println!("consumed word={word}");
    }
}
