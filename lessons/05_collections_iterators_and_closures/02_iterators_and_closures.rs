//! Lesson 5.2: iterator ownership modes, lazy adapters, and closures.

fn normalized_scores(scores: &[i32]) -> Vec<i32> {
    scores
        .iter()
        .copied()
        .filter(|score| *score >= 0)
        .map(|score| score.clamp(0, 100))
        .collect()
}

fn apply_twice<T>(mut value: T, mut operation: impl FnMut(T) -> T) -> T {
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
