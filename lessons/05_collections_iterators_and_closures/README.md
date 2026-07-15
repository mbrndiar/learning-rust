# 🧺♻️ Module 5: Collections, Iterators, and Closures

Standard collections own groups of values. Iterators borrow or consume those
collections through lazy, composable transformations.

## 🎯 Learning objectives

After this module, you should be able to choose a sequence, map, or set, update
collections through their APIs, distinguish `iter`, `iter_mut`, and `into_iter`,
compose iterator adapters, and use closures without obscuring ownership.

## 🗂️ Choosing a collection

| Type | Main property | Typical use |
| --- | --- | --- |
| `Vec<T>` | ordered, growable sequence | items processed by position/order |
| `VecDeque<T>` | efficient work at both ends | queues |
| `HashMap<K, V>` | average constant-time lookup | key-to-value association |
| `BTreeMap<K, V>` | keys kept sorted | deterministic ordered lookup |
| `HashSet<T>` | unique values and membership | tags, deduplication |
| `BTreeSet<T>` | unique sorted values | ordered unique output |

Choose by semantics first. Measure before replacing clear code with a structure
chosen only for theoretical complexity.

## ♻️ Iteration and ownership

For a collection named `values`:

- `values.iter()` yields shared references;
- `values.iter_mut()` yields exclusive mutable references;
- `values.into_iter()` consumes the collection and yields owned values.

Adapters such as `map`, `filter`, and `take` are lazy. A consumer such as
`collect`, `sum`, `find`, or a `for` loop drives the pipeline.

Closures can borrow, mutably borrow, or capture values by moving them. `move`
changes capture mode; it does not automatically make captured data `Copy` or
thread-safe.

## 📘 Lessons

- `01_collections.rs` — vectors, strings, maps, sets, entry APIs
- `02_iterators_and_closures.rs` — lazy adapters, ownership modes, closure
  capture, collecting results

## 🚀 Running

```bash
cargo run --example lesson-05-collections
cargo run --example lesson-05-iterators-closures
```

Then practice with
[`exercises/05_collections_iterators_and_closures/`](../../exercises/05_collections_iterators_and_closures/README.md).

## 🚧 Common mistakes

- Indexing a vector without handling the possibility of an invalid index.
- Keeping a reference into a vector while pushing may reallocate it.
- Assuming `HashMap` or `HashSet` iteration order is stable.
- Calling `map` without consuming the resulting iterator.
- Cloning items only to satisfy an ownership mismatch that a borrowed iterator
  would avoid.
- Writing an iterator chain whose control flow is less clear than a loop.

## 🧠 Review questions

1. When would `BTreeMap` be preferable to `HashMap`?
2. How do `get` and indexing differ for a missing key or element?
3. What item type does each of the three iterator entry methods yield?
4. Why does a chain of adapters do no work until consumed?
5. What does a `move` closure capture differently?
