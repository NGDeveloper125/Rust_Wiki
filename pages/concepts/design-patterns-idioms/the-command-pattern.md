---
title: "The command pattern"
area: "Design Patterns & Idioms"
embedded_support: partial
groups: ["Design Patterns", "Design Patterns & Idioms", "Object-Oriented-ish Patterns"]
related_syntax: [dyn]
see_also: ["Trait objects & dynamic dispatch (dyn Trait)", "The strategy pattern", "Fn / FnMut / FnOnce"]
---

## Explanation

The command pattern turns a request or action into a value, so it can be
stored, passed around, queued, logged, or undone independently of
whatever triggered it and whatever eventually executes it. Classic
object-oriented form is a `Command` interface with an `execute` method
(often paired with `undo`), one concrete class per action, and an
invoker that holds commands without knowing their concrete type. Rust
expresses the same idea in one of two shapes, chosen by how much state
and how many operations a command actually needs.

When a command is genuinely just "run this one closure later," a boxed
closure (`Box<dyn FnMut()>` and similar — see
[`Fn`/`FnMut`/`FnOnce`](../functions-closures/fn-fnmut-fnonce.md)) already
*is* a command: a closure is an anonymous struct holding captured state
plus one callable method, which is all a single-operation command needs.
When a command needs more than one operation — `execute` *and* `undo`,
or metadata like a display name for a menu entry — a real `Command`
trait implemented once per action is more idiomatic, dispatched through
[`Box<dyn Command>`](../traits-polymorphism/trait-objects-dynamic-dispatch.md)
wherever a collection needs to hold different kinds of commands side by
side.

Either shape decouples *what* should happen (a toolbar button, an HTTP
route, a scheduled task) from *when* it happens (a queue, an undo stack,
a worker loop) — the invoker only ever calls `execute`, never needing to
know which concrete action it's holding. This is the same runtime
polymorphism trick behind the [visitor](the-visitor-pattern.md) and
[strategy](the-strategy-pattern.md) patterns, applied specifically to
*actions* rather than *data* (visitor) or *algorithms invoked repeatedly*
(strategy): a `Vec<Box<dyn Command>>` is a to-do list of heterogeneous
actions the invoker will get to, in order, one at a time.

Because commands are ordinary values, storing them costs nothing extra
beyond whatever data they capture — an undo stack is just a `Vec` that
grows and shrinks as actions are applied and reversed, with no special
undo-specific machinery beyond the trait itself.

## Basic usage example

```
trait Command {
    fn execute(&self);
}

struct TurnOnLight;
impl Command for TurnOnLight {
    fn execute(&self) {
        println!("light: on");
    }
}

struct TurnOffLight;
impl Command for TurnOffLight {
    fn execute(&self) {
        println!("light: off");
    }
}

let queue: Vec<Box<dyn Command>> = vec![Box::new(TurnOnLight), Box::new(TurnOffLight)];
// <- each queued command is a different concrete type, held behind one trait object
for command in &queue {
    command.execute();
}
```

## Best practices & deeper information

### Scenario: Runtime polymorphism

A text editor needs an undo stack where every user action — typing,
deleting, formatting — can be reversed, and the set of action types
keeps growing as features are added, which rules out a single fixed
enum.

```
trait Command {
    fn execute(&self, doc: &mut String);
    fn undo(&self, doc: &mut String);
}

struct Insert { text: String, at: usize }
impl Command for Insert {
    fn execute(&self, doc: &mut String) {
        doc.insert_str(self.at, &self.text);
    }
    fn undo(&self, doc: &mut String) {
        doc.replace_range(self.at..self.at + self.text.len(), "");
    }
}

struct Editor {
    text: String,
    undo_stack: Vec<Box<dyn Command>>, // <- heterogeneous history of every reversible action taken so far
}

impl Editor {
    fn apply(&mut self, command: Box<dyn Command>) {
        command.execute(&mut self.text);
        self.undo_stack.push(command);
    }

    fn undo(&mut self) {
        if let Some(command) = self.undo_stack.pop() {
            command.undo(&mut self.text);
        }
    }
}

let mut editor = Editor { text: String::new(), undo_stack: Vec::new() };
editor.apply(Box::new(Insert { text: "hello".into(), at: 0 }));
editor.undo();
```

**Why this way:** storing each applied action as a `Box<dyn Command>`
means the undo stack never needs to know which concrete action types
exist — new command types can be added later without touching `Editor`
at all, exactly the decoupling the
[Rust Design Patterns' command entry](https://rust-unofficial.github.io/patterns/patterns/behavioural/command.html)
describes.

### Scenario: Working with collections

A background worker processes a queue of jobs enqueued from unrelated
parts of an application; each job is a different concrete type, but the
worker only ever needs to call one method on whatever it's holding.

```
trait Job {
    fn run(&self);
}

struct SendEmail { to: String }
impl Job for SendEmail {
    fn run(&self) {
        println!("emailing {}", self.to);
    }
}

struct ResizeImage { path: String }
impl Job for ResizeImage {
    fn run(&self) {
        println!("resizing {}", self.path);
    }
}

struct Worker {
    jobs: Vec<Box<dyn Job>>, // <- FIFO queue of unrelated job types behind one trait object
}

impl Worker {
    fn enqueue(&mut self, job: Box<dyn Job>) {
        self.jobs.push(job);
    }

    fn run_all(&mut self) {
        for job in self.jobs.drain(..) { // <- processes jobs in the order they were enqueued
            job.run();
        }
    }
}

let mut worker = Worker { jobs: Vec::new() };
worker.enqueue(Box::new(SendEmail { to: "a@example.com".into() }));
worker.enqueue(Box::new(ResizeImage { path: "photo.png".into() }));
worker.run_all();
```

**Why this way:** `Vec::drain` hands ownership of each queued command
out one at a time while leaving the `Vec` empty and ready for more work,
the standard shape for a job queue per the
[standard library's `Vec` docs](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.drain).

### Scenario: Handling and propagating errors

Not every queued command can be guaranteed to succeed — a command that
performs a network call might fail — so `execute` should return a
`Result` the runner can collect, instead of panicking and losing the
rest of the batch.

```
trait Command {
    fn execute(&self) -> Result<(), String>;
}

struct SyncRemoteFile { url: String }
impl Command for SyncRemoteFile {
    fn execute(&self) -> Result<(), String> {
        if self.url.starts_with("https://") {
            Ok(())
        } else {
            Err(format!("refusing insecure url: {}", self.url)) // <- command reports failure instead of panicking
        }
    }
}

fn run_all(commands: &[Box<dyn Command>]) -> Vec<String> {
    commands
        .iter()
        .filter_map(|command| command.execute().err()) // <- collects every failure without stopping the queue
        .collect()
}

let commands: Vec<Box<dyn Command>> = vec![Box::new(SyncRemoteFile { url: "http://x".into() })];
let errors = run_all(&commands);
println!("{errors:?}");
```

**Why this way:** letting each command report failure through a
`Result` instead of panicking keeps one bad command from taking down an
entire queued batch, matching the propagate-don't-panic guidance in the
[Rust Book's error handling chapter](https://doc.rust-lang.org/book/ch09-00-error-handling.html).

## Explanation (Embedded)

**Partial support — the caveat is specifically about ownership and heap
allocation, not about the pattern's decoupling idea.** The classic shape
shown above — `Vec<Box<dyn Command>>` as a queue or undo stack — needs
the `alloc` crate for two independent reasons: the `Vec` itself grows on
the heap, and each `Box<dyn Command>` stores its command behind a heap
allocation so the queue can hold different concrete command types side
by side (the same `Box<dyn Trait>` mechanism the
[`box` keyword page](../../syntax/keywords/box.md) covers — see that page
for why there's no fixed-capacity substitute for `Box<T>` itself, unlike
`heapless::Vec<T, N>` for `Vec`). Neither requirement disappears just
because the target is embedded, but there are two genuinely idiomatic
no-heap alternatives, chosen by whether the command set is closed and
whether the invoker needs to *own* the command:

If the set of commands a firmware image will ever issue is fixed and
known at compile time — set a GPIO pin, start an ADC conversion, toggle
a PWM channel — the idiomatic no-heap shape is a fixed-size `enum
Command { ... }` dispatched with one `match`, stored in an ordinary
array or a `heapless::spsc::Queue<Command, N>` instead of a `Vec`. This
gives up the ability for downstream/plugin crates to add new command
types (the enum has to be extended in this crate), which is exactly the
tradeoff the [visitor](the-visitor-pattern.md) and
[strategy](the-strategy-pattern.md) pages make the same way — but it's
usually the right one for firmware, where the full command set really is
closed at build time anyway.

If the command set genuinely needs to stay open (a plugin-style
architecture is unusual but not impossible in embedded, e.g. a scripting
layer registering handlers), and the invoker only needs to *call* the
command rather than *own and store* it for later, `&dyn Command`/`&mut
dyn FnMut()` gets runtime polymorphism with no allocation at all — the
same [on-stack dynamic dispatch](on-stack-dynamic-dispatch.md) trick
applied to actions instead of data. This only works when something else
already owns the command for the reference's whole lifetime; an undo
stack that needs to *keep* commands around after the call that created
them still needs either `alloc` or a fixed-capacity array of a single
concrete command enum, since a reference alone can't outlive its
referent.

## Basic usage example (Embedded)

```
enum Command { // <- fixed-size enum in place of Box<dyn Command>: no allocator needed
    SetPin { pin: u8, high: bool },
    StartAdcConversion { channel: u8 },
}

fn execute(cmd: Command) {
    match cmd { // <- static dispatch via match, standing in for Command::execute
        Command::SetPin { pin, high } => { let _ = (pin, high); }
        Command::StartAdcConversion { channel } => { let _ = channel; }
    }
}

execute(Command::SetPin { pin: 13, high: true });
```

## Best practices & deeper information (Embedded)

### Scenario: Branching on data (pattern matching)

A motor-control firmware image issues only a handful of command kinds,
all known at build time, queued from an interrupt handler and drained in
the main loop — a closed set that never needs a new command type without
a firmware rebuild anyway.

```
enum MotorCommand {
    SetSpeed { rpm: u16 },
    Stop,
    ReverseDirection,
}

const QUEUE_LEN: usize = 8;

struct CommandQueue {
    items: [Option<MotorCommand>; QUEUE_LEN], // <- fixed-capacity queue: no Vec, no allocator
    len: usize,
}

impl CommandQueue {
    fn push(&mut self, cmd: MotorCommand) {
        if self.len < QUEUE_LEN {
            self.items[self.len] = Some(cmd);
            self.len += 1;
        }
    }

    fn drain(&mut self) {
        for slot in self.items.iter_mut().take(self.len) {
            if let Some(cmd) = slot.take() {
                match cmd { // <- match plays Command::execute's role, dispatched statically
                    MotorCommand::SetSpeed { rpm } => { let _ = rpm; }
                    MotorCommand::Stop => {}
                    MotorCommand::ReverseDirection => {}
                }
            }
        }
        self.len = 0;
    }
}
```

**Why this way:** every command kind this firmware will ever issue is
known at compile time, so a fixed-size enum matched exhaustively gives
the same "decouple what happens from when it happens" benefit as
`Box<dyn Command>` with zero heap allocation and zero vtable indirection
— per the [`box` keyword's embedded notes](../../syntax/keywords/box.md),
there is no fixed-capacity substitute for `Box<dyn Trait>` itself, so a
closed command set is precisely the case where reaching for an enum
instead is the right call, not a workaround.

### Scenario: Runtime polymorphism

A firmware image supports a small scripting layer where a handler can be
registered for the duration of a single command dispatch pass; the
handler set isn't a fixed enum the core crate can enumerate, but nothing
needs to *own* the handler past the call that runs it, so a reference
avoids allocating entirely.

```
trait Command {
    fn execute(&self);
}

struct SetPinCommand { pin: u8 }
impl Command for SetPinCommand {
    fn execute(&self) {
        let _ = self.pin; // stand-in for a real register write
    }
}

fn dispatch(cmd: &dyn Command) { // <- reference, not Box: no heap allocation required
    cmd.execute();
}

let set_pin = SetPinCommand { pin: 5 }; // <- an ordinary stack value, not boxed
dispatch(&set_pin);
```

**Why this way:** `&dyn Command` gets the same "invoker doesn't know the
concrete type" decoupling as `Box<dyn Command>` without needing `alloc`,
as long as something else keeps the command alive for the call — the
[on-stack dynamic dispatch](on-stack-dynamic-dispatch.md) idiom is this
exact technique applied to commands specifically; once a command needs
to *outlive* the call that dispatches it (a real undo stack), a
reference alone is no longer enough and either `alloc` or a closed enum
is the honest choice instead.
