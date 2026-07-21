---
title: "#[used]"
kind: attribute
embedded_support: full
groups: ["FFI & Linkage", "Memory & Unsafe"]
related_concepts: ["FFI (foreign function interface)", "Unsafe Rust"]
related_syntax: [static, "#[no_mangle] / #[link(...)] / #[link_name] / #[link_ordinal] / #[link_section] / #[no_link] / #[export_name]"]
see_also: ["#[no_mangle] / #[link(...)] / #[link_name] / #[link_ordinal] / #[link_section] / #[no_link] / #[export_name]"]
---

## Explanation

`#[used]` is placed directly above a `static` item and takes no
arguments. It tells the compiler to keep that static in the final binary
even if ordinary dead-code analysis would conclude nothing in the Rust
program ever reads it and remove it.

The compiler's dead-code elimination only sees references from *within*
the compiled program. A static placed into a specific section with
`#[link_section = "..."]` so that a linker script, a bootloader, or a
debugger can find it by address or by section — never by an ordinary Rust
expression — looks, from the compiler's point of view, completely unused:
nothing loads it, nothing calls a function that touches it. Without
`#[used]`, the compiler is free to (and often will) strip such a static
out entirely, silently breaking whatever external mechanism depended on
its presence at that address.

`#[used]` doesn't change the static's type, value, or visibility — it is
purely a directive to the compiler's optimizer and dead-code eliminator:
"keep this in the output regardless of what your reachability analysis
concludes." It has no effect on ordinary statics that are already read
somewhere in the Rust program; those are never eligible for elimination
in the first place.

## Basic usage example

```
#[used] // <- keeps this static in the binary even though nothing in Rust reads it
#[unsafe(link_section = ".init_array")]
static INIT_ENTRY: extern "C" fn() = init;

extern "C" fn init() {}
```

## Best practices & deeper information

### Scenario: Crossing an FFI boundary

A microcontroller's linker script expects a table of function pointers at
a fixed section so a boot ROM can find and call them; nothing in the Rust
program itself ever reads that table, so it must be marked `#[used]` or
the compiler discards it as dead code.

```
#[used] // <- linker-script-only consumer: the compiler can't see this reference on its own
#[unsafe(link_section = ".vector_table.reset")]
static RESET_HANDLER: unsafe extern "C" fn() -> ! = reset_handler;

unsafe extern "C" fn reset_handler() -> ! {
    loop {}
}
```

**Why this way:** the linker script, not any Rust code, is what reads
`RESET_HANDLER` by address, so from the compiler's reachability analysis
this static has zero readers and would otherwise be eliminated — the
[Rust Reference on the `used` attribute](https://doc.rust-lang.org/reference/abi.html#the-used-attribute)
documents exactly this "referenced only outside the compiler's view" case
as its purpose.

## Embedded Rust Notes

**Full support.** `#[used]` is one of the more common embedded-only
attributes: interrupt vector tables, linker-script-placed configuration
tables, and boot-stage function pointer tables are all read by something
outside Rust's visibility (a linker script, a ROM bootloader, a debugger
probe), so they need `#[used]` to survive dead-code elimination. It has
essentially no use case in hosted `std` binaries, where nothing outside
the program's own object files typically inspects statics by raw address.
