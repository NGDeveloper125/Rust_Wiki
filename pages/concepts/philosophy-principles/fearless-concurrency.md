---
title: "Fearless concurrency"
area: "Rust Philosophy & Design Principles"
embedded_support: full
groups: ["Rust Philosophy & Design Principles", "Concurrent / Message-Passing", "Writing Concurrent & Parallel Code", "Unique to Rust", "Multithreading"]
related_syntax: []
see_also: ["Send & Sync", "Threads (std::thread)", "Shared-state concurrency (Mutex, RwLock)", "Message passing (channels / mpsc)", "The borrow checker", "Zero-cost abstractions"]
---

## Explanation

"Fearless concurrency" is the Rust team's own name for a specific promise:
writing multi-threaded code shouldn't require the constant, low-grade
vigilance that concurrent programming demands in C or C++, where sharing
mutable data across threads compiles cleanly whether it's safe or not, and
a mistake might not surface until a customer hits it under production load
months later. Rust backs this promise with the same mechanism it uses for
memory safety generally: ownership and borrowing, checked entirely at
compile time, extended across the thread boundary by two marker traits,
[`Send` and `Sync`](../concurrency-async/send-and-sync.md).

The mechanics are a direct extension of single-threaded borrowing. A data
race requires two things: more than one thread touching the same memory,
with at least one of them writing, and no synchronization between them.
Rust's borrow checker already guarantees that a value has either any
number of live shared references or exactly one mutable reference, never
both at once — see [the borrow checker](../ownership-borrowing/borrow-checker.md)
and [mutable borrowing](../ownership-borrowing/mutable-borrowing.md) for
the single-threaded version of this rule. `Send` and `Sync` simply state
whether that guarantee still holds once a value crosses into another
thread: `Send` governs whether ownership can move to another thread at
all, `Sync` governs whether `&T` can safely be held by more than one
thread at once. Because `std::thread::spawn` requires its closure's
captures to satisfy these bounds, code that would share genuinely
thread-unsafe data — an `Rc<T>`'s non-atomic reference count, for
instance — simply fails to compile, rather than shipping a race condition.
[Threads](../concurrency-async/threads.md),
[shared-state locking](../concurrency-async/shared-state-concurrency.md),
and [message-passing channels](../concurrency-async/message-passing-channels.md)
are the three concrete places this guarantee actually does its work.

It's important to be precise about what "fearless" covers, because it's
narrower than "concurrency bugs are impossible." The compiler eliminates
*data races* specifically — simultaneous, unsynchronized access with at
least one write — because that class of bug is exactly what ownership and
`Send`/`Sync` are built to reject. It does not, and cannot, eliminate
deadlocks (two threads each waiting on a lock the other already holds),
logic races (two threads correctly synchronized individually but combined
in the wrong order to produce a wrong result), or plain algorithmic bugs.
The type system checks that access to shared data is synchronized; it has
no opinion on whether the synchronization you wrote is logically correct.
Even the failure modes Rust does surface are handled as visible, typed
signals rather than silent corruption — a panic while holding a `Mutex`
poisons the lock rather than leaving the guarded data in a
partially-updated state nobody notices.

Async code carries a version of the same story into a different
execution model: `async`/`await` (see [async/await](../concurrency-async/async-await.md)
and [futures](../concurrency-async/futures.md)) still checks `Send` bounds
on anything a spawned task captures across an `.await` point, so the same
"won't compile if it isn't safe to hand to the executor" guarantee applies
there too — the risk profile just shifts from data races toward
accidentally blocking an executor thread, which the type system does not
catch and which is why blocking calls inside async code are treated as a
correctness bug in their own right rather than merely a performance one.

Taken together, this is why Rust gets reached for in domains that used to
be considered too risky to parallelize casually — browser engine
components, database internals, high-throughput network services. The
promise was never "concurrent code is easy to design correctly." It's
narrower and more honest than that: an entire, historically brutal-to-debug
category of bug is caught before the program ever runs, by the same
compiler that was already checking ownership for the single-threaded case.

## Basic usage example

```
use std::thread;

let total = 0;

// thread::spawn(|| { total += 1; }); // would fail to compile: this closure would need to outlive `total`,
                                       // which it can't safely borrow across a `'static`-bound thread

let handle = thread::spawn(move || { // <- ownership of `total` moves into the thread instead of being borrowed
    println!("received {total}");
});

handle.join().unwrap();
```

Fixing the commented-out version means making the choice explicit: move
`total`'s ownership into the thread (as the working version above does),
or wrap it in something the compiler already knows is safe to share, like
an atomic or a `Mutex`. Either way, the compiler forces that decision
before two threads can quietly race.

## Best practices & deeper information

### Scenario: Multi-threading

A pending-orders queue touched by more than one worker thread has to be
reshaped before it compiles at all — sharing a bare `&mut` across threads
is rejected outright, which is exactly the guarantee "fearless" refers to.

```
use std::sync::{Arc, Mutex};
use std::thread;

struct OrderQueue { pending: Vec<u32> }

let queue = OrderQueue { pending: vec![101, 102, 103] };

// AVOID: sharing `&mut queue` across threads — rejected at compile time
// thread::spawn(|| queue.pending.push(104));
// thread::spawn(|| queue.pending.pop());
// error: closure may outlive the current function, but it borrows `queue`, which is owned by the current function

// PREFER: make the shared, synchronized access explicit
let queue = Arc::new(Mutex::new(queue));
let producer = Arc::clone(&queue);
let consumer = Arc::clone(&queue);

thread::spawn(move || producer.lock().unwrap().pending.push(104)); // <- Send/Sync + Mutex checked at compile time
thread::spawn(move || { consumer.lock().unwrap().pending.pop(); });
```

**Why this way:** the compiler rejects the shared `&mut` before the
program can run at all, rather than letting two threads race on the same
`Vec` — the
[Rust Book's shared-state chapter](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
covers `Arc<Mutex<T>>` as the standard fix once data genuinely needs to be
touched from more than one thread.

### Scenario: Sharing state across threads

A shared job counter needs to survive one worker panicking mid-update
without silently corrupting the count — Rust's `Mutex` shows fearless
concurrency's honest edge: it doesn't prevent the panic, but it turns what
would otherwise be silent data corruption into a visible, typed error.

```
use std::sync::{Arc, Mutex};
use std::thread;

let jobs_completed = Arc::new(Mutex::new(0u32));

let worker = Arc::clone(&jobs_completed);
let handle = thread::spawn(move || {
    let mut count = worker.lock().unwrap();
    *count += 1;
    panic!("worker crashed mid-update"); // <- lock is still held by this guard when the panic unwinds
});
let _ = handle.join(); // the panic is caught here, not silently swallowed

match jobs_completed.lock() { // <- returns Err: the Mutex is now marked poisoned
    Ok(count) => println!("{count} jobs completed"),
    Err(poisoned) => println!("recovering after a worker panic: {}", *poisoned.into_inner()),
}
```

**Why this way:** the
[`std::sync::Mutex` docs](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
describe poisoning as turning a potential silent-corruption bug into an
explicit `Err` a caller must decide how to handle — even Rust's failure
mode for a threading bug is a compile-time-typed signal, not an
undiagnosed data race.

## Explanation (Embedded)

On bare metal there's usually no OS and no `std::thread::spawn`, so it's
tempting to think "fearless concurrency" simply doesn't apply — but almost
every microcontroller program is concurrent in the sense that actually
matters here: an interrupt handler can preempt `fn main`'s loop at any
instruction boundary, run to completion, and hand control back, and from
the compiler's point of view that's a second concurrent context every bit
as real as a second OS thread. Any state touched by both `main` and a
`#[interrupt]` handler is exactly the shared-mutable-data situation
`Send`/`Sync` and the borrow checker exist to police — the guarantee
"fearless concurrency" describes for threads is the same guarantee,
applied to the main-loop/ISR boundary instead. The mechanics of *why* —
`Send`/`Sync` as compile-time-checked marker traits, `critical-section`'s
`Mutex` as the no-OS analog of `std::sync::Mutex` — are covered in full on
[Send & Sync](../concurrency-async/send-and-sync.md); this page won't
repeat that ground, only the reframing.

Async executors built for embedded, like `embassy`, add a third kind of
concurrent context: cooperatively-scheduled tasks that can be preempted at
`.await` points. The same `Send` bound the classic explanation described
for `std`'s async runtimes applies again here — anything a task captures
across an `.await`, or moves into `spawner.spawn(...)`, is checked at
compile time the same way.

The honest edges carry over unchanged, too, and if anything matter more on
a microcontroller. The type system still can't catch a logic race, and it
still can't catch a new failure mode specific to this context: holding a
`critical-section` too long doesn't corrupt data, but it does delay every
interrupt for as long as the section is held, which on a real-time target
can itself blow a timing deadline. That's a genuine embedded-specific
tradeoff the compiler has no opinion on — keeping critical sections short
is a discipline the type system doesn't enforce, exactly the way it
doesn't enforce deadlock-freedom on a hosted target.

## Basic usage example (Embedded)

```
use core::cell::Cell;
use critical_section::Mutex;

static BUTTON_PRESSED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false)); // <- Sync only because Cell<bool> is Send

#[interrupt]
fn EXTI0() { // <- second concurrent context: can preempt `main` at any instruction boundary
    critical_section::with(|cs| BUTTON_PRESSED.borrow(cs).set(true));
}

fn main() -> ! {
    loop {
        if critical_section::with(|cs| BUTTON_PRESSED.borrow(cs).get()) {
            // ... handle the button press
        }
    }
}
```

## Best practices & deeper information (Embedded)

### Scenario: Sharing state across threads

A motor controller's target speed is written by a UART-receive interrupt
and read every tick by the main control loop — reshaping it through a
`critical-section` mutex is what turns "shared mutable state touched from
two contexts" from a potential race into something the compiler has
actually checked.

```
use core::cell::Cell;
use critical_section::Mutex;

static TARGET_RPM: Mutex<Cell<u16>> = Mutex::new(Cell::new(0)); // <- static requires TARGET_RPM: Sync, checked at compile time

#[interrupt]
fn USART1() { // <- runs as a second concurrent context alongside `main`
    let new_target = read_rpm_from_uart_frame();
    critical_section::with(|cs| TARGET_RPM.borrow(cs).set(new_target));
}

fn control_loop_tick() {
    let target = critical_section::with(|cs| TARGET_RPM.borrow(cs).get());
    drive_motor_towards(target);
}

fn read_rpm_from_uart_frame() -> u16 { 1200 }
fn drive_motor_towards(_rpm: u16) {}
```

**Why this way:** a bare `static mut TARGET_RPM: u16` would compile but
gives the compiler nothing to check — reading and writing it from two
contexts is undefined behavior the moment they overlap; wrapping it in
`critical-section`'s `Mutex` makes the type genuinely `Sync` and forces
every access through a section with interrupts disabled, the mechanism
[Send & Sync](../concurrency-async/send-and-sync.md) covers in depth for
exactly this main-loop/ISR pairing.

### Scenario: Async tasks

An `embassy` executor running a sensor-polling task alongside a
button-watching task needs the same "won't compile if it isn't safe to
hand to the executor" guarantee async code gets on a hosted target — the
risk profile is identical, just running cooperatively on one core instead
of preemptively across OS threads.

```
use embassy_executor::Spawner;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

static LATEST_READING: Mutex<CriticalSectionRawMutex, u16> = Mutex::new(0);

#[embassy_executor::task]
async fn poll_sensor() {
    loop {
        let reading = read_sensor().await;
        *LATEST_READING.lock().await = reading; // <- checked Send at this .await, same as a spawned std task
    }
}

#[embassy_executor::task]
async fn report_reading() {
    loop {
        let reading = *LATEST_READING.lock().await;
        // ... transmit `reading`
    }
}

async fn read_sensor() -> u16 { 0 }
```

**Why this way:** `embassy`'s executor requires spawned futures to be
`'static` and the values they move across an `.await` to be `Send`, so a
task that captured non-thread-safe state without going through a shared,
lock-protected type would fail to compile rather than corrupt
`LATEST_READING` under task preemption — the same compile-time check the
classic explanation describes for `tokio`, applied to a single-core
cooperative executor instead of an OS-backed one.
