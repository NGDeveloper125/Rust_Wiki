# Rusty Yellow Pages — VS Code theme

One colour per role in Rust syntax, used for that role and nothing else.
Reaching a function through a type is painted differently from reaching one
through a value; declaring a struct is painted differently from naming it as
a type.

Generated from the same palette that paints the site, so the two cannot drift.
Edit `tools/sitegen/src/palette.rs` and re-run the generator — never edit the
JSON in `themes/` by hand.

## Install

Copy this folder into your extensions directory and reload the window:

- Windows: `%USERPROFILE%\.vscode\extensions\`
- macOS / Linux: `~/.vscode/extensions/`

Then pick it from **Preferences: Color Theme**.

## Requires rust-analyzer

The role colours ride on semantic tokens, which rust-analyzer provides. Without
it you get the coarse fallback — comments, literals, keywords and punctuation —
because TextMate scopes cannot tell an associated call from a method call.

To see what any identifier actually resolves to, run **Developer: Inspect
Editor Tokens and Scopes** from the command palette and click it.

## What does not survive the trip

- `type-return` — VS Code has no return-position token — rust-analyzer reports a return type as an ordinary type, so it falls back to the `type` colour.
