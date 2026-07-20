---
title: "The command pattern"
area: "Design Patterns & Idioms"
embedded_support: partial
groups: ["Design Patterns & Idioms", "Object-Oriented-ish Patterns"]
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

## Embedded Rust Notes

**Partial support.** The `Command` trait itself, and calling `execute`
through a reference, cost nothing and need no allocator. The common
shape shown here — a `Vec<Box<dyn Command>>` queue or undo stack — needs
the `alloc` crate, since both the `Vec` and each `Box` allocate. Where no
allocator is available at all, the same decoupling works with static
dispatch instead: a fixed `enum Command { TurnOnLight, TurnOffLight }`
matched in one `execute` function, giving up runtime extensibility in
exchange for zero allocation.
