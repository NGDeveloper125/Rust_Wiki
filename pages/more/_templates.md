<!--
  THE TEMPLATES PAGE INTRODUCTION.

  General prose for `more/templates/`, above the cards. It states what cargo
  generates, what it does not, and what a template here generates instead.
  Anything true of one template and not the others belongs on that template's
  own page, in pages/more/templates/<name>.md.

  Write it as documentation, not as an argument: short sentences, concrete
  nouns, no case-making. A reader wants to know what they get.

  Two rules for editing this file:

  - No code fences. Every fence on this site is painted by the Rust
    highlighter, and the commands on this page are shell. Inline `code` is
    fine; whole commands belong in the generated part of the page.
  - Links are ordinary relative paths to built pages (../../concepts/….html),
    not the `.md` links used elsewhere. This file does not go through the
    link rewriter.
-->

`cargo new` creates a package containing a
[Cargo.toml](../../concepts/modules-crates-visibility/cargo-and-cargo-toml.html),
a `.gitignore` and one source file. `--bin` writes a `main` that prints
`Hello, world!`; `--lib` writes an `add` function and a unit test. There are no
other variants, and the two cannot be combined.

Everything past that is set up by hand: further crates and the dependencies
between them, a [workspace](../../concepts/modules-crates-visibility/workspaces.html)
manifest, [modules](../../concepts/modules-crates-visibility/modules.html) and
their declarations, third-party dependencies and the code that initialises
them.

The templates here generate that setup. Each one is a project layout with the
structure and configuration already in place, and a single worked example of
whatever the template covers. What a template covers varies:

- a console crate and a library crate in one workspace, with the path
  dependency and the call between them written
- a module containing code, declared and wired into the crate around it
- a web API with a route, a handler and server startup, so the generated
  project can be run and the endpoint called
- logging configured and initialised, so a log statement compiles and outputs
  without further setup

The example code is meant to be replaced. Deleting the bodies leaves the
structure, which is the part being generated.

Templates are generated with [cargo-generate](https://cargo-generate.github.io/cargo-generate/),
a cargo subcommand. It copies the template, substitutes the names given on the
command line, and writes the project. Names not given are derived from the
project name.
