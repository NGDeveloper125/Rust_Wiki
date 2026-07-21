---
title: "'static"
kind: lifetime
embedded_support: full
groups: ["Ownership & Borrowing"]
related_concepts: ["Lifetimes", "Closures & capturing"]
related_syntax: ["'a (named lifetime)", "&"]
see_also: ["'a (named lifetime)"]
---

## Explanation

`'static` is a lifetime built into the language — unlike `'a`, it never
needs to be declared in a generic parameter list before use; it (along with
the elided placeholder `'_`) is simply always available. It appears in two
positions with related but distinct meanings.

As the lifetime of a **reference type**, `&'static T` means the reference is
valid for the entire remainder of the running program. String literals are
the most common example: a literal like `"engine-status"` is baked directly
into the compiled binary's read-only data and never deallocated, so its type
is `&'static str` automatically, with no annotation needed.

As a **bound** on a generic type parameter, `T: 'static`, the meaning is
different enough that it's the most commonly misread piece of syntax in
this area: it does **not** mean "values of `T` live for the whole program."
It means "`T` contains no borrowed data with a lifetime shorter than
`'static`" — every reference nested anywhere inside `T`, if there are any at
all, is itself valid for the program's duration. This is why fully owned
types — `String`, `i32`, `Vec<u8>`, or a struct made entirely of owned
fields — satisfy `T: 'static` trivially and unconditionally, no matter how
briefly a particular value of that type actually lives. A `String` created
and dropped a microsecond later still satisfies `String: 'static`, because
the bound describes the type's *structure* (can it possibly contain a
dangling short-lived borrow?), not any individual value's runtime lifespan.
A type like `&'a str`, or a struct holding one, only satisfies a `'static`
bound if `'a: 'static` — that is, if the reference it holds is itself
program-long-lived.

`T: 'static` is required wherever a consumer can't prove how long it will
keep hold of a value — most visibly, `thread::spawn`'s closure parameter is
bounded `F: FnOnce() -> T + Send + 'static`, because the compiler has no way
to know when (or whether) the spawned thread finishes relative to the
caller's stack frame, so it must rule out the closure holding any borrow
that frame's locals could invalidate. This is the place most Rust
programmers first run into `'static` as a bound and, misreading it as "the
closure must run forever," reach for `.clone()` or `Arc` before realizing an
already-owned value satisfies the bound for free (see
[Move semantics](../../concepts/ownership-borrowing/move-semantics.md) and
[`move`](../keywords/move.md) for how an owned capture gets there).

## Usage examples

### Binding a string literal as &'static str

```
let name: &'static str = "engine-status"; // <- string literals are `&'static str` automatically
println!("{name}");
```

### Multi-threading

`thread::spawn`'s closure must be `'static`; moving an owned `String` into
it satisfies that bound trivially, since owned data can't contain a
dangling borrow of anything.

```
use std::thread;

fn start_worker(label: String) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // <- the closure passed to `spawn` must be `'static`; `label` is owned, so it qualifies for free
        println!("[{label}] worker running");
    })
}

let handle = start_worker(String::from("indexer"));
handle.join().unwrap();
```

The spawned thread's lifetime isn't bounded by the
caller's stack frame, so nothing it captures may borrow from that frame —
the
[Book's concurrency chapter](https://doc.rust-lang.org/book/ch16-01-threads.html#using-move-closures-with-threads)
covers `move`-ing an owned value in as the standard way to satisfy
`thread::spawn`'s `'static` bound without cloning anything unnecessarily.

### Designing a public API

A scheduler that stores heterogeneous jobs behind `Box<dyn Trait>` needs
those trait objects to be `'static` — not because a job runs forever, but
because the scheduler can't predict how long it will hold onto any given
job.

```
trait Job: Send {
    fn run(&self);
}

struct Scheduler {
    jobs: Vec<Box<dyn Job + 'static>>, // <- `'static` here means "no non-'static borrows inside," not "lives forever"
}

impl Scheduler {
    fn add(&mut self, job: impl Job + 'static) {
        self.jobs.push(Box::new(job));
    }
}
```

`dyn Trait` defaults to a `'static` lifetime bound unless
a shorter one is spelled out explicitly, which the
[Rust Reference's trait-object lifetime-bounds section](https://doc.rust-lang.org/reference/types/trait-object.html#trait-object-lifetime-bounds)
documents as the reason a stored trait object needs the data behind it to
be fully owned or itself `'static`.

### Sharing data with multiple references

Constants and literals baked into the binary are naturally `&'static`, so
many call sites can share the same reference without any explicit lifetime
parameter appearing anywhere.

```
const DEFAULT_REGION: &str = "us-east-1"; // <- elided, but the real type is `&'static str`

fn region_label(region: Option<&'static str>) -> &'static str {
    region.unwrap_or(DEFAULT_REGION)
}

println!("{}", region_label(Some("eu-west-1")));
println!("{}", region_label(None));
```

The
[standard library's `str` documentation](https://doc.rust-lang.org/std/primitive.str.html)
notes that string literals are `&'static str` because they live directly in
the compiled binary and are never deallocated, which is what lets code pass
them around anywhere a `'static` reference is required with no lifetime
annotation to write.

## Explanation (Embedded)

`'static` means exactly what it means on a hosted target — a lifetime
built into the language, valid for the whole remainder of the running
program, in both of its positions (reference type and bound). What's
different in embedded code isn't the meaning; it's how central and
unavoidable it becomes. On a hosted target, most `'static` bounds trace
back to one thing: handing data to something that outlives the current
call stack, most visibly `thread::spawn`. Embedded code runs into that
same problem far more often, and in a sharper form, because interrupts
don't have anything resembling a caller's stack frame to bound them: an
ISR can fire at essentially any point after the interrupt table is
installed, completely independent of whatever function happens to be
running in `main` at that instant. Any data an interrupt handler needs
to touch — a shared counter, a ring buffer, a peripheral handle —
therefore can't be a `main`-local variable borrowed by reference; it has
to be `'static`, almost always as a `static`/`static mut` item guarded
by a `Mutex<Cell<_>>`/`RefCell<_>` and a critical section, so both
`main` and the handler can reach the same data without either side
outliving the other in a way the compiler can verify.

The same shape recurs in a handful of other embedded-specific places: a
`#[global_allocator]` is declared as a `static`, because the allocator
has to be reachable for the entire life of the program, not scoped to
whichever function first touches the heap; `heapless`/RTIC-style shared
resources are frequently split from a `static mut` queue or buffer
specifically because the producer/consumer halves need to be usable
from both an interrupt context and `main` without either one's stack
frame bounding how long the split halves are allowed to live; and DMA
transfers, which keep writing into a buffer autonomously after the
function that started them has already returned, need that buffer to
outlive the call that set it up — which a `'static` buffer guarantees
and a stack-local one can't. None of this changes what `'static`
*means* — it's the same "no non-`'static` borrows inside," same "valid
for the rest of the program" as on a hosted target — embedded code just
runs into the *reason* for that bound far more routinely than most
hosted code does.

## Usage examples (Embedded)

### Sharing a counter between `main` and an interrupt handler

```
use core::cell::Cell;
use critical_section::Mutex;
use stm32f4xx_hal::pac::interrupt;

static TICKS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
// <- `'static` storage: TIM2 can fire at any point after main starts, so this can't live on main's stack

#[interrupt]
fn TIM2() {
    critical_section::with(|cs| {
        let cell = TICKS.borrow(cs);
        cell.set(cell.get() + 1);
    });
}

fn ticks() -> u32 {
    critical_section::with(|cs| TICKS.borrow(cs).get())
}
```

### Declaring a global allocator

```
use embedded_alloc::Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty(); // <- must be `'static`: the allocator has to be reachable for the program's whole life

fn init_heap() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 1024;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}
```

### Splitting a heapless queue into interrupt-safe producer/consumer halves

```
use heapless::spsc::{Consumer, Producer, Queue};

static mut READINGS: Queue<u16, 8> = Queue::new();

fn split_queue() -> (Producer<'static, u16, 8>, Consumer<'static, u16, 8>) {
    // <- `split` requires `&'static mut self`: the producer/consumer halves must outlive
    //    any interrupt that later uses them, not just this function's own call
    unsafe { READINGS.split() }
}
```
