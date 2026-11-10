---
title: "tempfile"
version: "3.27.0"
publisher: "Steven Allen (Stebalien)"
publisher_url: "https://crates.io/users/Stebalien"
no_std: "no"
author: "NGDeveloper125"
github: "NGDeveloper125"
date: "2026-11-10"
summary: "Temporary files and directories that delete themselves. Four entry points depending on whether you need a path, a plain `File`, a whole directory, or a buffer that only touches disk once it gets large — plus the `persist` half, which is how you write a file atomically."
domain: "Files, paths & OS"
categories: ["filesystem", "development-tools::testing"]
repository: "https://github.com/Stebalien/tempfile"
docs: "https://docs.rs/tempfile/3.27.0/tempfile/"
---

## Overview

Creating a temporary file by hand gets two things wrong. The name is guessable,
so on a shared `/tmp` another process can win the race and plant a symlink
where your file was about to be; and the cleanup is yours to remember, which
means it does not happen on the error paths. `tempfile` solves both: names come
from a CSPRNG, files are created with `O_EXCL` so a collision is an error rather
than an open of somebody else's file, and deletion is tied to a destructor.

There are four entry points, and picking the right one is most of the work:

```
use std::io::Write;

// 1. A real `File` with no path at all. The OS deletes it when the last
//    handle closes, so nothing can leave it behind.
let mut unnamed = tempfile::tempfile().unwrap();
unnamed.write_all(b"scratch").unwrap();

// 2. A file you can refer to by path -- needed to hand it to a subprocess,
//    or to rename it into place later.
let named = tempfile::NamedTempFile::new().unwrap();
assert!(named.path().exists());

// 3. A directory, deleted recursively on drop.
let dir = tempfile::tempdir().unwrap();
assert!(dir.path().is_dir());

// 4. A buffer that lives in memory until it outgrows a threshold.
let spooled = tempfile::spooled_tempfile(64 * 1024);
assert!(!spooled.is_rolled());
```

**The cleanup guarantee is not the same for all four.** `tempfile()` hands the
problem to the operating system — the file is unlinked at creation on Unix and
opened `FILE_FLAG_DELETE_ON_CLOSE` on Windows — so it is removed even if your
process is killed. `NamedTempFile` and `TempDir` rely on Rust destructors, which
means `std::process::exit`, an abort, a `SIGKILL`, or simply storing one in a
`static` will leave files behind. If that matters, prefer the unnamed form.

It is a small, old, heavily used crate under active maintenance: four
dependencies ([`fastrand`](fastrand.md), `once_cell`, `getrandom`, and `rustix`
or `windows-sys` depending on the platform), `MIT OR Apache-2.0`, and MSRV 1.63
held deliberately low to suit Debian stable. It is `std`-only, which is inherent
— there is no filesystem to be temporary on without one.

**Two things that are easy to get wrong**, both covered below: a `TempDir` or
`NamedTempFile` passed *by value* to a function taking `impl AsRef<Path>` is
dropped inside that call, deleting the thing you were about to use; and
temporary *directories* are created with your umask rather than privately, so
unlike temporary files they are typically world-readable.

## When to use it

### Use case: A test that needs real files on disk

The commonest use by a wide margin. A test that exercises a path-taking API
needs somewhere to put fixtures, and must not leave them behind or collide with
a parallel test.

```
use std::fs;
use tempfile::tempdir;

// The function under test -- anything that reads real files.
fn count_lines(dir: &std::path::Path) -> usize {
    fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| fs::read_to_string(e.unwrap().path()).ok())
        .map(|s| s.lines().count())
        .sum()
}

let dir = tempdir().unwrap();
fs::write(dir.path().join("a.txt"), "one\ntwo\n").unwrap();
fs::write(dir.path().join("b.txt"), "three\n").unwrap();
fs::create_dir(dir.path().join("nested")).unwrap();

assert_eq!(count_lines(dir.path()), 3);

// Keep the path to prove the cleanup afterwards.
let path = dir.path().to_path_buf();
drop(dir);
assert!(!path.exists()); // <- the nested directory went too
```

**Why it fits:** each test gets its own directory under the system temp
directory, so tests running in parallel cannot tread on each other, and the
removal is recursive — you do not have to track what the code under test
created. Pass `dir.path()`, not `dir`, which is the subject of the next
paragraph and the single most common mistake with this crate.

### Use case: Replacing a file without a window where it is broken

Writing straight over a config or data file means a crash mid-write leaves a
truncated file. The fix is to write a temporary file in the **same directory**
and rename it into place, because a rename within one filesystem is atomic.

```
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

let home = TempDir::new().unwrap();          // stands in for /etc/myapp
let target = home.path().join("config.toml");
fs::write(&target, "old = true\n").unwrap();

// In the SAME directory as the target, so the rename stays on one filesystem.
let mut tmp = NamedTempFile::new_in(home.path()).unwrap();
tmp.write_all(b"new = true\n").unwrap();
tmp.as_file().sync_all().unwrap();           // <- durable before the rename

let file = tmp.persist(&target).unwrap();    // atomic replace; returns the File
drop(file);

assert_eq!(fs::read_to_string(&target).unwrap(), "new = true\n");

// `persist` overwrites. To refuse an existing target, ask for that:
let tmp2 = NamedTempFile::new_in(home.path()).unwrap();
let kept = tmp2.path().to_path_buf();
let err = tmp2.persist_noclobber(&target).unwrap_err();
assert_eq!(err.error.kind(), std::io::ErrorKind::AlreadyExists);

// The error hands the file back, so the attempt is recoverable.
assert_eq!(err.file.path(), kept);
assert!(kept.exists());
```

**Why it fits:** a reader of `config.toml` sees either the old contents or the
new ones and never a half-written file. The two details that make it work are
easy to miss. `new_in(same_directory)` is required — `persist` across
filesystems fails, and the default temp directory is usually a different one, so
`NamedTempFile::new()` followed by `persist` into `/etc` is a bug that only
shows up on real deployments. And `sync_all` before the rename is what makes the
new contents survive a power loss; without it the rename can land before the
data does.

### Use case: Buffering something whose size you do not know

An upload, a response body, a generated report. Small ones should stay in
memory; large ones must not. `spooled_tempfile` switches over for you.

```
use std::io::{Read, Seek, SeekFrom, Write};
use tempfile::{spooled_tempfile, SpooledData};

// Stay in memory up to 8 bytes.
let mut buf = spooled_tempfile(8);
assert!(!buf.is_rolled());

buf.write_all(b"12345678").unwrap();
assert!(!buf.is_rolled());        // <- exactly at the threshold: still in RAM

buf.write_all(b"9").unwrap();
assert!(buf.is_rolled());         // <- one byte past: now a file on disk

// It behaves the same either way: Read + Write + Seek throughout.
buf.seek(SeekFrom::Start(0)).unwrap();
let mut out = String::new();
buf.read_to_string(&mut out).unwrap();
assert_eq!(out, "123456789");

// And you can take whichever it ended up being.
assert!(matches!(buf.into_inner(), SpooledData::OnDisk(_)));

let mut small = spooled_tempfile(1024);
small.write_all(b"tiny").unwrap();
assert!(matches!(small.into_inner(), SpooledData::InMemory(_)));
```

**Why it fits:** the common case costs no syscalls and no cleanup, while the
large case cannot exhaust memory, and the calling code does not branch on which
happened. Note the threshold is an *exceeded* bound, not a reached one — at
exactly `max_size` bytes the data is still in memory, as the assertions above
show. Once rolled it stays rolled; shrinking the contents with `set_len` does
not bring it back into memory.

## API map

Grouped by which of the four kinds of temporary thing you want, then by the two
cross-cutting concerns: controlling the name and location, and getting a
temporary file to stop being temporary.

### Unnamed temporary files

#### `tempfile` and `tempfile_in`

A `std::fs::File` with no path, and the only form that cannot leak.

```
use std::io::{Read, Seek, SeekFrom, Write};

let mut f = tempfile::tempfile().unwrap();

// It is an ordinary File -- no wrapper, no deref, no new methods.
f.write_all(b"scratch data").unwrap();
f.seek(SeekFrom::Start(0)).unwrap();          // <- writing leaves the cursor at the end
let mut s = String::new();
f.read_to_string(&mut s).unwrap();
assert_eq!(s, "scratch data");

// `tempfile_in` puts it on a filesystem you choose -- a big scratch volume,
// or the same device as a file you are about to rewrite.
let dir = tempfile::tempdir().unwrap();
let g = tempfile::tempfile_in(dir.path()).unwrap();
assert_eq!(g.metadata().unwrap().len(), 0);
```

**When to use it:** the default choice. Reach for it whenever the file is pure
scratch space — sorting more data than fits in memory, staging a download you
will read back yourself — because the OS removes it on the last close whatever
happens to your process. The reason to use something else is needing a *path*:
you cannot hand this to a subprocess or to an API that takes a filename.

### Named temporary files

#### `NamedTempFile::new` and friends

A temporary file you can name, and the constructors that shape that name.

```
use tempfile::NamedTempFile;

let plain = NamedTempFile::new().unwrap();
let name = plain.path().file_name().unwrap().to_string_lossy().to_string();
// `.tmp` plus six random characters, by default.
assert!(name.starts_with(".tmp"));
assert_eq!(name.len(), ".tmp".len() + 6);

// Three variants when the name matters, each with an `_in` sibling.
let dir = tempfile::tempdir().unwrap();
let suffixed = NamedTempFile::with_suffix_in(".json", dir.path()).unwrap();
assert_eq!(suffixed.path().extension().unwrap(), "json");

let prefixed = NamedTempFile::with_prefix_in("upload-", dir.path()).unwrap();
assert!(prefixed.path().file_name().unwrap().to_string_lossy().starts_with("upload-"));
```

**When to use it:** when something other than your own code has to find the
file — a subprocess argument, a library that only accepts a path, a format
whose reader insists on an extension. `with_suffix` earns its keep there, since
plenty of tools dispatch on the extension. Use `_in` whenever you intend to
`persist` later, so the eventual rename stays on one filesystem. Be aware of
the security trade: a path exists, which means a sufficiently hostile shared
temp directory can interfere with it in ways the unnamed form is immune to.

#### `path`, and the early-drop trap

The path is borrowed from the handle, and that is load-bearing.

```
use std::path::{Path, PathBuf};
use tempfile::tempdir;

// Any ordinary path-taking helper. `impl AsRef<Path>` takes ownership.
fn remember<P: AsRef<Path>>(p: P) -> PathBuf {
    p.as_ref().to_path_buf()
}

// Wrong: `dir` is moved into the call and dropped when it returns.
let dir = tempdir().unwrap();
let gone = remember(dir);
assert!(!gone.exists()); // <- deleted before you could use it

// Right: pass a reference, or better, pass the path.
let dir = tempdir().unwrap();
let here = remember(&dir);
assert!(here.exists());
assert_eq!(here, remember(dir.path()));
```

**When to use it:** `path()` is how you get at the location; the trap is what to
watch for. `TempDir`, `NamedTempFile` and `TempPath` all implement
`AsRef<Path>`, which is convenient and means a by-value argument compiles
cleanly and then deletes your file — classically with
`Command::new(..).current_dir(dir)`, where the directory is gone before the
child starts. Pass `dir.path()` and the problem cannot arise, because the
borrow keeps the handle alive for you.

#### `persist` and `persist_noclobber`

Move the file to a chosen path and stop treating it as temporary.

```
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

let home = TempDir::new().unwrap();
let target = home.path().join("out.bin");

let mut tmp = NamedTempFile::new_in(home.path()).unwrap();
tmp.write_all(b"payload").unwrap();
let temp_path = tmp.path().to_path_buf();

// Returns the open File, positioned where you left it.
let file = tmp.persist(&target).unwrap();
assert_eq!(file.metadata().unwrap().len(), 7);
assert!(!temp_path.exists());
assert!(target.exists());

// Failure is recoverable: `PersistError` carries both the cause and the file.
let tmp2 = NamedTempFile::new_in(home.path()).unwrap();
let err = tmp2.persist(home.path().join("missing-dir").join("x")).unwrap_err();
assert_eq!(err.error.kind(), std::io::ErrorKind::NotFound);
let recovered: NamedTempFile = err.file;     // <- still a live temporary file
assert!(recovered.path().exists());
```

**When to use it:** for the atomic-write pattern, which is the main reason to
choose `NamedTempFile` over `tempfile()`. `persist` replaces an existing target
silently; `persist_noclobber` returns `AlreadyExists` instead, and is what you
want when the target appearing means another process got there first. Both
require the target to be on the same filesystem as the temporary file — a
cross-device attempt fails rather than silently copying — so create it with
`new_in` next to the destination.

#### `keep`, `close` and `disable_cleanup`

The three other ways a `NamedTempFile` can end.

```
use tempfile::NamedTempFile;

// `keep` stops the deletion and hands back the file and its current path.
let (file, path) = NamedTempFile::new().unwrap().keep().unwrap();
drop(file);
assert!(path.exists());
std::fs::remove_file(&path).unwrap();

// `close` deletes now and reports what `drop` would have swallowed.
let f = NamedTempFile::new().unwrap();
let p = f.path().to_path_buf();
f.close().unwrap();
assert!(!p.exists());

// The error drop cannot tell you about -- here the file vanished underneath,
// exactly what a temporary file cleaner does to a long-lived one.
let f = NamedTempFile::new().unwrap();
std::fs::remove_file(f.path()).unwrap();
assert_eq!(f.close().unwrap_err().kind(), std::io::ErrorKind::NotFound);

// `disable_cleanup` is the in-place version of `keep`.
let mut f = NamedTempFile::new().unwrap();
let p = f.path().to_path_buf();
f.disable_cleanup(true);
drop(f);
assert!(p.exists());
std::fs::remove_file(&p).unwrap();
```

**When to use it:** `close` whenever you care that the removal worked —
`Drop` returns nothing and therefore discards the error, so a cleanup that
quietly fails is invisible unless you ask. `keep` for a file you decided to
promote but do not want to move, and for debugging a failing test: flip it on
and the artefacts are still there afterwards. `disable_cleanup` suits the case
where the decision is made somewhere that only has a `&mut`.

#### `reopen`

A second, independent handle on the same file.

```
use std::io::{Read, Seek, Write};
use tempfile::NamedTempFile;

let mut writer = NamedTempFile::new().unwrap();
writer.write_all(b"shared").unwrap();
writer.as_file().sync_all().unwrap();

let mut reader = writer.reopen().unwrap();
let mut s = String::new();
reader.read_to_string(&mut s).unwrap();      // <- its own cursor, starting at 0
assert_eq!(s, "shared");

// The original is untouched: still positioned after the write.
assert_eq!(writer.as_file_mut().stream_position().unwrap(), 6);
```

**When to use it:** when one part of the program writes while another reads, and
you do not want to coordinate a single cursor. It is better than opening
`path()` yourself because it is immune to the path having been replaced in the
meantime. On Unix the handle survives the file being unlinked, so a cleaner
removing the name does not break a reader that already has one.

#### `into_parts`, `from_parts` and `TempPath`

Splitting the handle into the file and the deletion responsibility.

```
use tempfile::{NamedTempFile, TempPath};

let f = NamedTempFile::new().unwrap();
let original = f.path().to_path_buf();

// Take the two halves apart...
let (file, temp_path): (std::fs::File, TempPath) = f.into_parts();
assert_eq!(&*temp_path, &*original);         // TempPath derefs to Path
drop(file);
assert!(temp_path.exists());                 // the TempPath still owns the deletion

// ...and put them back.
let rejoined = NamedTempFile::from_parts(std::fs::File::open(&temp_path).unwrap(), temp_path);
assert_eq!(rejoined.path(), original);
drop(rejoined);
assert!(!original.exists());

// `into_file` and `into_temp_path` discard the other half instead.
let leaked = NamedTempFile::new().unwrap().into_file();
assert_eq!(leaked.metadata().unwrap().len(), 0);
```

**When to use it:** `into_temp_path` when you want the deletion but not the open
handle — reserving a name for a subprocess to write. `into_file` when you want
the opposite, though note the name is then never cleaned up, which makes it a
deliberate leak rather than a convenience. `into_parts`/`from_parts` are for
passing the two responsibilities through a boundary that cannot hold a
`NamedTempFile`, and `TempPath::try_from_path` adopts a path you created
yourself. `TempPath::from_path` still exists but is deprecated in favour of the
fallible version.

### Temporary directories

#### `tempdir` and `TempDir::new`

A directory removed recursively on drop.

```
use tempfile::{tempdir, TempDir};

// `tempdir()` and `TempDir::new()` are the same thing.
let a = tempdir().unwrap();
let b = TempDir::new().unwrap();
assert!(a.path().is_dir() && b.path().is_dir());
assert_ne!(a.path(), b.path());

// Nested content is no problem.
std::fs::create_dir_all(a.path().join("x/y/z")).unwrap();
std::fs::write(a.path().join("x/y/z/f.txt"), b"data").unwrap();

let path = a.path().to_path_buf();
drop(a);
assert!(!path.exists());

// `with_prefix` and `_in` work as they do for files.
let parent = tempdir().unwrap();
let child = TempDir::with_prefix_in("build-", parent.path()).unwrap();
assert!(child.path().starts_with(parent.path()));
```

**When to use it:** whenever the unit of work is a tree rather than one file —
test fixtures, unpacking an archive, a scratch build directory, anything you
hand to a subprocess as a working directory. One caveat worth repeating from the
crate's own documentation: temporary *directories* are created with the default
permissions and so are usually world-readable, where temporary *files* are
private. If you will put secrets in one, set the mode yourself with
`Builder::permissions`.

#### `TempDir::keep` and `close`

Ending a temporary directory deliberately.

```
use tempfile::TempDir;

// `close` removes it now and tells you whether that worked.
let d = TempDir::new().unwrap();
let p = d.path().to_path_buf();
d.close().unwrap();
assert!(!p.exists());

// The failure `drop` would have hidden.
let d = TempDir::new().unwrap();
std::fs::remove_dir_all(d.path()).unwrap();
assert_eq!(d.close().unwrap_err().kind(), std::io::ErrorKind::NotFound);

// `keep` gives up ownership of the deletion and returns the path.
let kept = TempDir::new().unwrap().keep();
assert!(kept.is_dir());
std::fs::remove_dir_all(&kept).unwrap();
```

**When to use it:** `close` at the end of a test that should fail loudly if it
left a file locked; `keep` when a directory turned out to be the output rather
than scratch, or when you want to inspect it after a failure. `keep` replaced
`into_path`, which is deprecated and otherwise identical — the rename was to
stop it reading as a cheap conversion when it actually changes who is
responsible for the directory.

### Controlling names, location and permissions

#### `Builder`

Every knob, for the cases the shorthand constructors do not cover.

```
use tempfile::Builder;

let dir = tempfile::tempdir().unwrap();

let f = Builder::new()
    .prefix("report-")
    .suffix(".json")
    .rand_bytes(10)                  // <- default is 6
    .tempfile_in(dir.path())
    .unwrap();

let name = f.path().file_name().unwrap().to_string_lossy().to_string();
assert!(name.starts_with("report-") && name.ends_with(".json"));
assert_eq!(name.len(), "report-".len() + 10 + ".json".len());

// `append(true)` makes every write go to the end, seek or no seek.
use std::io::{Read, Seek, SeekFrom, Write};
let mut a = Builder::new().append(true).tempfile_in(dir.path()).unwrap();
a.write_all(b"one").unwrap();
a.seek(SeekFrom::Start(0)).unwrap();
a.write_all(b"two").unwrap();
a.seek(SeekFrom::Start(0)).unwrap();
let mut s = String::new();
a.read_to_string(&mut s).unwrap();
assert_eq!(s, "onetwo");             // <- not "twoe"

// `make` builds a NamedTempFile<F> over something other than a File.
let wrapped: tempfile::NamedTempFile<std::io::BufWriter<std::fs::File>> =
    Builder::new()
        .make_in(dir.path(), |p| std::fs::File::create(p).map(std::io::BufWriter::new))
        .unwrap();
assert!(wrapped.path().exists());

// Zero random bytes means a fixed, predictable name. Rarely what you want.
let fixed = Builder::new().prefix("exact").rand_bytes(0).tempfile_in(dir.path()).unwrap();
assert_eq!(fixed.path().file_name().unwrap(), "exact");
```

**When to use it:** `suffix` for a tool that dispatches on extension, `prefix`
to make stray files identifiable when something does leak, `permissions` for a
directory that will hold secrets. Leave `rand_bytes` alone unless you have a
reason: six characters already means an attacker must create billions of files
to collide, and the generator reseeds from system randomness after three failed
attempts. Setting it to `0` removes the protection entirely and turns a
collision into an error on every run — occasionally useful for a fixture with a
known name, never right for a real temporary file.

#### `env::temp_dir` and `env::override_temp_dir`

Where temporary things go when you do not say.

```
use tempfile::env as tf_env;

// By default this is exactly `std::env::temp_dir()`.
assert_eq!(tf_env::temp_dir(), std::env::temp_dir());

let sandbox = tempfile::tempdir().unwrap();
let chosen = sandbox.path().to_path_buf();

// Process-global, and only the first call takes effect.
assert!(tf_env::override_temp_dir(&chosen).is_ok());
assert_eq!(tf_env::temp_dir(), chosen);
assert!(tempfile::NamedTempFile::new().unwrap().path().starts_with(&chosen));

// A later call fails and returns the override already in force.
assert_eq!(tf_env::override_temp_dir(std::path::Path::new("elsewhere")), Err(chosen));
```

**When to use it:** `temp_dir` to report or inspect the location. `override_temp_dir`
only in an application, only at startup, and only when the platform's default is
genuinely unsuitable — a world-writable `/tmp` managed by an aggressive cleaner,
or a sandbox with a different writable location. It is process-global, so a
library must not call it; a library that needs control should use the `_in`
constructors instead, which affect only its own files. Note it does not check
that the directory exists or is writable, so a bad path turns into a creation
error later rather than an error here.
