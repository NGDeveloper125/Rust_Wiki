---
title: "#[expect(...)]"
kind: attribute
embedded_support: full
groups: ["Lints & Diagnostics", "Testing & Tooling"]
related_concepts: []
related_syntax: ["#[allow(...)] / #[warn(...)] / #[deny(...)] / #[forbid(...)]"]
see_also: ["#[allow(...)] / #[warn(...)] / #[deny(...)] / #[forbid(...)]"]
---

## Explanation

`#[expect(lint_name)]`, stabilized in Rust 2024, silences a named lint for
the scope it decorates exactly like [`#[allow(lint_name)]`](allow-and-friends.md)
does — but with one additional check: the lint must actually **fire at
least once** somewhere in that scope. If it never fires, `#[expect(...)]`
itself produces a warning — `unfulfilled_lint_expectation` — flagging that
the expectation is no longer needed.

This solves a real, easy-to-miss failure mode of `#[allow(...)]`: an
`#[allow(dead_code)]` written to silence a specific, currently-unused
function stays completely silent forever, even after that function starts
being called somewhere and the lint would no longer have fired anyway.
Nothing about a stale `#[allow]` ever calls attention to itself — it just
sits there, technically unnecessary, indistinguishable at a glance from
one that's still actively suppressing something. `#[expect(...)]` inverts
that: the moment the code changes enough that the lint stops firing, the
`#[expect(...)]` itself lights up as a new warning, prompting whoever sees
it to remove the now-pointless annotation.

The natural use is exactly the same shape as reaching for `#[allow(...)]`
to suppress a lint you plan to address later — a known, temporary issue
being tracked rather than a permanent policy — except `#[expect(...)]`
makes that temporariness self-enforcing: it disappears from view (no
warning) for as long as it's genuinely still needed, and reappears (a new
warning) the moment it stops being needed, instead of lingering silently
like a stale `#[allow]` would.

## Usage examples

### Suppressing a lint that must eventually fire again

```
#[expect(dead_code)] // <- suppresses the lint, but warns if dead_code stops firing here
fn planned_for_next_release() {}
```

### Designing a public API

A refactor leaves one helper function temporarily unused while a caller
is being rewritten in a follow-up change — `#[expect(dead_code)]` silences
the warning for now, but will itself start warning the moment the
function either gets called again or is actually deleted, so the
suppression can't quietly outlive its reason for existing.

```
#[expect(dead_code, reason = "will be wired into the new pricing pipeline in a follow-up change")]
fn legacy_discount_calculation(price_cents: u32) -> u32 {
    price_cents / 2
}

fn current_pricing(price_cents: u32) -> u32 {
    price_cents // legacy_discount_calculation not called yet — dead_code fires, #[expect] fulfills its purpose
}
```

An `#[allow(dead_code)]` here would keep silencing the
lint indefinitely even after `legacy_discount_calculation` gets wired in
and the lint would no longer fire anyway, leaving a stale, misleading
annotation behind; `#[expect(...)]` instead produces
`unfulfilled_lint_expectation` the moment that happens, which the
[Rust Reference](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-expect-attribute)
documents as the attribute's specific purpose — catching suppressions
that have outlived their reason for existing.

### Testing

A test module temporarily has one helper assertion function that isn't
called by any test yet, while the rest of the suite is being written —
`#[expect]` tracks that as a known, temporary gap rather than a
permanently silenced warning.

```
#[cfg(test)]
mod tests {
    #[expect(dead_code, reason = "will be used once the boundary-condition tests are added")]
    fn assert_within_cents(actual: u32, expected: u32, tolerance: u32) {
        assert!(actual.abs_diff(expected) <= tolerance);
    }

    #[test]
    fn placeholder_until_more_tests_land() {
        assert_eq!(2 + 2, 4);
    }
}
```

Once a real test starts calling `assert_within_cents`,
`dead_code` stops firing and the `#[expect]` itself becomes a visible
`unfulfilled_lint_expectation` warning — a natural nudge to delete the now
pointless attribute — instead of the suppression silently persisting
forever the way a plain `#[allow(dead_code)]` would.

## Explanation (Embedded)

`#[expect(...)]` is a pure compile-time lint-tracking mechanism with no
runtime footprint, and nothing about it changes under `#![no_std]` — the
embedded story here is identical to [`#[allow(...)]`](allow-and-friends.md)'s.
It's genuinely useful in firmware code for the same reason it is
anywhere else: a HAL's work-in-progress peripheral module with a
temporarily-unused helper function can track that as a known,
self-expiring gap instead of a silently-stale suppression.

## Usage examples (Embedded)

### Tracking a temporarily-unused HAL helper

```
#[expect(dead_code, reason = "will be called once the SPI DMA path lands")] // <- warns again once dead_code stops firing
fn configure_spi_dma_burst(len: usize) {
    let _ = len;
}
```
