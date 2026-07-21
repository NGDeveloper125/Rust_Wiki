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

## Explanation (Embedded)

`[T]` lives in `core`, and a slice reference is genuinely just a
pointer-plus-length — no allocator is involved in creating one, borrowing
one, or passing one around, which makes `&[T]`/`&mut [T]` one of the most
heavily used types in allocator-free embedded code. What makes slices
specifically valuable on constrained hardware is that a slice doesn't
care *where* the memory it views came from: the same `&mut [u8]`
parameter accepts a view into a stack-local array, a `'static` buffer
placed at a fixed address for DMA, or the currently-occupied portion of a
`heapless::Vec`'s inline backing storage. A function written to take
`&mut [u8]` is, without any extra work, already agnostic to which of
those three the caller is using — it's the mechanism that lets embedded
code pass "a view into a buffer" around as a single, uniform concept
regardless of the buffer's actual storage class.

This matters most concretely for peripheral APIs: a DMA-driven UART or
SPI transfer needs a buffer at a stable address for the duration of the
transfer, so HAL APIs commonly take `&mut [u8]` — a caller supplies a
`'static mut` buffer (or one otherwise guaranteed to outlive the
transfer) as a slice, and the peripheral driver reads or writes into it
without ever needing to know whether that memory happens to be a fixed
`static` array, a stack buffer scoped correctly against the transfer, or
backing storage borrowed out of a `heapless` collection.

## Basic usage example (Embedded)

```
fn fill_zero(buf: &mut [u8]) { // <- doesn't care whether buf's storage is a stack array or 'static memory
    for byte in buf.iter_mut() {
        *byte = 0;
    }
}

let mut stack_buf = [0xFFu8; 8];
fill_zero(&mut stack_buf); // <- a stack array, viewed as a slice

static mut DMA_BUF: [u8; 8] = [0xFF; 8];
unsafe { fill_zero(&mut DMA_BUF); } // <- 'static memory at a fixed address, viewed the same way
```

## Best practices & deeper information (Embedded)

### Scenario: Working with collections

A UART driver's "read some bytes" function should accept `&mut [u8]`
rather than committing to any one buffer's storage class, so the same
function serves a quick stack-allocated command buffer and a `'static`
buffer wired up for DMA alike.

```
fn read_bytes(available: &[u8], into: &mut [u8]) -> usize { // <- neither slice commits to a storage class
    let n = available.len().min(into.len());
    into[..n].copy_from_slice(&available[..n]);
    n
}

let mut quick_cmd = [0u8; 4]; // stack-allocated, short-lived
let n1 = read_bytes(&[0x01, 0x02], &mut quick_cmd);

static mut DMA_RX: [u8; 256] = [0; 256]; // 'static, address-stable for DMA
let n2 = unsafe { read_bytes(&[0xAA; 10], &mut DMA_RX) };
```

**Why this way:** writing the function against `&mut [u8]` instead of a
concrete array size or a specific buffer type means the exact same
function serves a throwaway stack buffer and a long-lived DMA target,
which matters on hardware where duplicating the function per buffer
storage class would just be more code competing for the same limited
flash.

### Scenario: Sharing data with multiple references

Splitting one fixed buffer into non-overlapping views for a frame's
header and payload is a common DMA-adjacent pattern, and it costs nothing
beyond the original buffer's own storage — no copying, no second
allocation, because both views point into the same memory.

```
let mut frame = [0u8; 32]; // one fixed 'static-or-stack buffer, however it's stored
let (header, payload) = frame.split_at_mut(4); // <- two independent &mut [u8] views, no copy, same backing memory

header.copy_from_slice(&[0xAA, 0x01, 0x00, 0x1C]);
payload[0] = 0x42;
```

**Why this way:** `split_at_mut` produces two views into the *same*
buffer without any new storage, which is exactly what a resource-
constrained target needs when a single fixed-size DMA buffer has to be
addressed as distinct header/payload regions by different parts of a
driver — allocating a second buffer just to separate the two would cost
memory the target may not have to spare.
