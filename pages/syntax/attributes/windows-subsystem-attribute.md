---
title: "#[windows_subsystem = \"...\"]"
kind: attribute
embedded_support: none
groups: ["FFI & Linkage", "Memory & Unsafe"]
related_concepts: []
related_syntax: []
see_also: []
---

## Explanation

`#![windows_subsystem = "..."]` is an inner attribute placed at the top
of a crate root that selects which Windows PE subsystem the compiled
executable is linked as. It is meaningful only when compiling for a
Windows target; on every other target it has no effect at all.

Windows executables declare, in their binary header, which of two
subsystems they belong to. `"console"` is the default: the OS
automatically attaches a console window to the process if it wasn't
already launched from one, which is the right behavior for a command-line
tool. `"windows"` marks the executable as a GUI application: launching it
does **not** open a console window, which is what any ordinary graphical
desktop application wants — a windowed app popping up a blank black
console alongside its actual window looks broken to a user.

This attribute exists purely to control that one piece of platform
metadata; it does not change how the program compiles, what APIs are
available, or anything about `std`'s own behavior. Note the practical
consequence of choosing `"windows"`: `println!`/`eprintln!` output has
nowhere visible to go, since there is no attached console to write to —
a GUI application generally needs its own logging or message-box-based
error reporting instead of relying on standard output.

## Basic usage example

```
#![windows_subsystem = "windows"] // <- Windows-only: suppresses the console window at launch

fn main() {
    // a real GUI application would open its window here instead
}
```

## Best practices & deeper information

### Scenario: Designing a public API

A desktop configuration utility with its own graphical window looks
unpolished if launching it flashes a console window behind the GUI —
`#![windows_subsystem = "windows"]` on the Windows build suppresses that
console entirely.

```
#![cfg_attr(windows, windows_subsystem = "windows")] // <- applies only when actually targeting Windows

fn main() {
    // launch the application's GUI window here
}
```

**Why this way:** wrapping the attribute in `cfg_attr(windows, ...)`
keeps the crate portable — the attribute would otherwise need to be
present unconditionally, which is harmless on Windows but reads as
misleading boilerplate on every other target, since the attribute has no
effect there at all; the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/no_std_no_core.html#the-windows_subsystem-attribute)
documents `windows_subsystem` as a Windows-target-only piece of PE
metadata with `"console"` as the implicit default when the attribute is
absent.

## Embedded Rust Notes

**No embedded relevance.** This attribute controls a Windows-specific
executable header field and has no meaning on any embedded/bare-metal
target — there is no Windows PE subsystem to select on a microcontroller
with no OS at all. The toggle is disabled because the attribute simply
doesn't apply outside a hosted Windows build.
