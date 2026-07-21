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

## Usage examples

### Keeping a linker-section static from being stripped

```
#[used] // <- keeps this static in the binary even though nothing in Rust reads it
#[unsafe(link_section = ".init_array")]
static INIT_ENTRY: extern "C" fn() = init;

extern "C" fn init() {}
```

### Crossing an FFI boundary

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

The linker script, not any Rust code, is what reads
`RESET_HANDLER` by address, so from the compiler's reachability analysis
this static has zero readers and would otherwise be eliminated — the
[Rust Reference on the `used` attribute](https://doc.rust-lang.org/reference/abi.html#the-used-attribute)
documents exactly this "referenced only outside the compiler's view" case
as its purpose.

## Explanation (Embedded)

Firmware routinely has two entirely separate "readers" of a given
static: the Rust program itself, and something completely outside it —
a linker script, a ROM bootloader, a debug probe, or an external tool
scanning the compiled image for a known byte pattern. The compiler's
dead-code analysis only understands the first kind of reader. Anything
that's read exclusively by the second kind looks, from the compiler's
point of view, entirely unreachable, and gets stripped unless `#[used]`
overrides that conclusion — which is why `#[used]` shows up constantly
in firmware and almost never in hosted `std` code, where nothing external
to the compiled binary typically inspects a static by raw address at all.

Beyond the single-function vector-table entry, the same reasoning applies
to a whole table at once — an array of exception/interrupt vectors, or a
fixed metadata block a boot ROM or bootloader expects at a specific flash
offset (a chip's "boot header," an application's version/checksum
footer read by an over-the-air update tool). None of these are ever read
by an expression anywhere in the Rust program; they're read by address,
by something that isn't Rust at all, which is exactly the case `#[used]`
exists for.

## Usage examples (Embedded)

### Placing a full vector table, not just one entry

```
#[used] // <- the whole table, not only individual entries, has zero Rust-visible readers
#[unsafe(link_section = ".vector_table.exceptions")]
static EXCEPTIONS: [Option<unsafe extern "C" fn()>; 14] = [
    Some(nmi_handler),
    Some(hard_fault_handler),
    None, None, None, None, None, None, // reserved entries
    Some(sv_call_handler),
    None, None,
    Some(pend_sv_handler),
    Some(sys_tick_handler),
];

unsafe extern "C" fn nmi_handler() { loop {} }
unsafe extern "C" fn hard_fault_handler() { loop {} }
unsafe extern "C" fn sv_call_handler() {}
unsafe extern "C" fn pend_sv_handler() {}
unsafe extern "C" fn sys_tick_handler() {}
```

### Marking a firmware version footer for an OTA updater

```
#[used] // <- read only by an external OTA/flashing tool scanning the compiled image, never by Rust
#[unsafe(link_section = ".fw_metadata")]
static FIRMWARE_VERSION: [u8; 4] = [1, 4, 0, 0]; // major, minor, patch, reserved
```

Neither static is ever named by a Rust expression — the vector table is
read by the CPU's exception hardware directly from its fixed address, and
the version footer is read by a separate tool entirely outside the
firmware's own execution — so both would be silently eliminated as dead
code without `#[used]`, exactly the failure mode the
[Rust Reference on the `used` attribute](https://doc.rust-lang.org/reference/abi.html#the-used-attribute)
describes.
