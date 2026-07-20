---
title: "Slices"
area: "Types & Data Modeling"
embedded_support: full
groups: ["Types & Data Modeling", "Working with Collections", "Collections"]
related_syntax: ["[ ]"]
see_also: ["Arrays vs Vec"]
---

## Explanation

A slice, `[T]`, is a view into a contiguous run of elements without
owning them — almost always seen behind a reference, `&[T]` or
`&mut [T]`, since `[T]` itself is unsized (the compiler doesn't know its
length at compile time, so it can't be a plain stack value).

A slice reference is a "fat pointer": under the hood it's a pointer to
the first element plus a length, which is why slicing an array or `Vec`
(`&v[1..3]`) doesn't copy any elements — it just produces a new
pointer+length pair describing a sub-range of the original data. This
makes slices the natural common interface for "a sequence of `T`" that
works identically whether the backing storage is a fixed-size array, a
`Vec`, or a sub-range of either — a function taking `&[T]` accepts all
three without needing to know or care which one it was handed.

Bounds are checked at runtime on indexing and slicing (`arr[5]`,
`arr[1..3]`) — an out-of-range access panics rather than reading
adjacent memory, which is part of what makes slices memory-safe despite
being a thin, low-level view rather than an owning collection.

## Basic usage example

```
let v = vec![10, 20, 30, 40];
let s: &[i32] = &v[1..3]; // <- a view into part of v; no elements are copied
println!("{:?}", s);       // [20, 30]
```

**Restriction:** indexing or slicing out of range panics at runtime
(`&v[1..10]` here would panic) rather than being caught at compile time
— use `.get(range)`, which returns `Option`, when the bounds aren't
already known to be valid.

## Best practices & deeper information

### Scenario: Working with collections

A function that only needs to *read* a sequence should take `&[T]`
rather than committing to `&Vec<T>` or `&[T; N]` specifically — the same
function then works no matter how the caller happens to be storing the
data.

```
fn average(readings: &[f64]) -> f64 { // <- &[f64] accepts a Vec, an array, or a sub-slice of either
    readings.iter().sum::<f64>() / readings.len() as f64
}

let today: Vec<f64> = vec![21.5, 22.0, 21.8];
let fixed: [f64; 3] = [19.0, 19.5, 20.1];

average(&today);  // Vec coerces to &[f64] via Deref
average(&fixed);  // array coerces to &[f64] via an unsized coercion -- same function, no duplication
```

**Why this way:** the API Guidelines'
[C-GENERIC](https://rust-lang.github.io/api-guidelines/flexibility.html#functions-minimize-assumptions-about-parameters-by-using-generic-types-c-generic)
advice is to minimize assumptions about parameters — taking `&[T]`
instead of a specific owning type is exactly that, for any function that
only reads.

### Scenario: Sharing data with multiple references

Slicing borrows rather than copies, so splitting one collection into
several logical views for different readers is essentially free, and
any number of shared slices can coexist over the same data at once.

```
let sensor_log = vec![10.1, 10.3, 9.8, 11.0, 10.5];

let morning = &sensor_log[0..2]; // <- borrows part of sensor_log; no elements are copied
let evening = &sensor_log[2..5]; // <- a second, independent shared view into the same Vec

println!("morning avg: {}", morning.iter().sum::<f64>() / morning.len() as f64);
println!("evening avg: {}", evening.iter().sum::<f64>() / evening.len() as f64);
```

**Why this way:** the
[Rust Book](https://doc.rust-lang.org/book/ch04-03-slices.html) covers
slices as borrowed views specifically so multiple readers can look at
different (or overlapping) parts of the same data without any of them
needing ownership — the borrow checker still guarantees none of these
slices can outlive `sensor_log` itself.

## Embedded Rust Notes

**Full support.** `[T]` lives in `core` — no allocator needed. Slices
over statically-sized buffers (`&mut [u8]` for a DMA transfer, for
instance) are a staple of allocator-free embedded code.
