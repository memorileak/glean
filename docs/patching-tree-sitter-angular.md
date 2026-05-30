# Patching `tree-sitter-angular` for tree-sitter 0.26

## The problem

The published crate [`tree-sitter-angular`](https://crates.io/crates/tree-sitter-angular)
(latest version **0.9.2** at time of writing) declares a hard dependency on
`tree-sitter ~0.25`.  This project uses `tree-sitter = "0.26.9"`.  Cargo cannot
reconcile `~0.25` with `0.26.x` because tree-sitter follows a semver-breaking
release cadence (the C ABI changes between minor versions), so the crate cannot
be added directly.

There is no newer version of `tree-sitter-angular` on crates.io that supports
the `0.26` series.

---

## Solution: patch the crate with `[patch.crates-io]`

Cargo's [`[patch.crates-io]`](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html)
feature lets you redirect any crate name to a local path.  The local copy is
a minimal fork of the 0.9.2 source with only the changes needed to compile
against tree-sitter 0.26.  The crate's own `build.rs` (which already handles
C compilation correctly) remains completely untouched.

### Step 1 — Create the local patch directory

Download the `tree-sitter-angular 0.9.2` crate from crates.io (e.g. via
`cargo download` or by extracting it from `~/.cargo/registry/`) and place it
under:

```
patches/
└── tree-sitter-angular/   # full crate source, modified as described below
```

### Step 2 — Update the version constraints in `Cargo.toml` / `Cargo.toml.orig`

The crate ships with two manifest files.  Both must be updated (Cargo reads
`Cargo.toml.orig` for path dependencies):

```toml
# patches/tree-sitter-angular/Cargo.toml  (and Cargo.toml.orig)

[dependencies]
tree-sitter = "0.26"           # was "~0.25"
tree-sitter-language = "0.1"  # new — provides LanguageFn for the 0.26 API
```

### Step 3 — Update the bundled C headers

`tree-sitter-angular 0.9.2` bundles its own copies of the tree-sitter C headers
in `src/tree_sitter/` and uses `.include("src")` in its `build.rs` so that
`#include "tree_sitter/parser.h"` inside the generated C resolves to those
bundled files.  The bundled headers are 0.25 versions; compiling them while
linking against the 0.26 runtime would cause ABI mismatches.

Replace every `.h` file under `patches/tree-sitter-angular/src/tree_sitter/`
with the matching files from the tree-sitter 0.26 source tree (found in
`~/.cargo/registry/src/.../tree-sitter-0.26.9/src/`):

```
patches/tree-sitter-angular/src/tree_sitter/
├── parser.h      # 0.26 version
├── array.h       # 0.26 version  (includes ./ts_assert.h — see note below)
├── alloc.h       # 0.26 version
├── ts_assert.h   # new in 0.26 — must also be copied
└── …             # copy ALL .h files, not just the obvious three
```

> **Note**: tree-sitter 0.26's `array.h` uses a relative include
> `#include "./ts_assert.h"`, so `ts_assert.h` must be in the *same directory*.
> Copy all `.h` files to be safe.

### Step 4 — Update the Rust FFI binding (`bindings/rust/lib.rs`)

The 0.25 API exposed the grammar's entry point as a C function returning a
`Language` struct directly.  In 0.26 the C function returns a raw pointer and
the Rust side must wrap it using `LanguageFn::from_raw`.

Replace the old unsafe extern block:

```rust
// BEFORE (0.25 API)
use tree_sitter::Language;
extern "C" { fn tree_sitter_angular() -> Language; }
pub fn language() -> Language {
    unsafe { tree_sitter_angular() }
}
```

With the new pattern:

```rust
// AFTER (0.26 API)
use tree_sitter::Language;
use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_angular() -> *const ();
}

pub fn language() -> Language {
    unsafe { LanguageFn::from_raw(tree_sitter_angular) }.into()
}
```

### Step 5 — Wire up the patch in the main `Cargo.toml`

Add `tree-sitter-angular` as a normal dependency, then redirect it to the local
patch:

```toml
[dependencies]
tree-sitter-angular = "0.9.2"

[patch.crates-io]
tree-sitter-angular = { path = "patches/tree-sitter-angular" }
```

No `build = "build.rs"` entry or `[build-dependencies]` are needed in the main
manifest — all C compilation is handled by the patched crate's own `build.rs`.

### Step 6 — Simplify `src/angular.rs`

Because the patched crate now exports a proper `language()` function, the
main project no longer needs any unsafe FFI code:

```rust
// src/angular.rs — just call the crate's public API
impl LanguageExt for Angular {
    fn get_ts_language(&self) -> TSLanguage {
        tree_sitter_angular::language().into()
    }
}
```

---

## How the crate's own `build.rs` works

The patched crate's `build.rs` is unchanged from the upstream 0.9.2 release.
Understanding it explains why the header replacement in Step 3 is all that is
needed:

```
build.rs
│
└─ cc::Build::new()
       .include("src")                        // makes "tree_sitter/parser.h" resolve
       .file("src/parser.c")
       .file("src/scanner.c")
       .compile("tree-sitter-angular")
```

The key is `.include("src")` — the compiler search path is set to the crate's
own `src/` directory.  When `parser.c` does `#include "tree_sitter/parser.h"`,
the compiler looks for `src/tree_sitter/parser.h` inside the crate.  This is
exactly the directory we replaced with 0.26 headers in Step 3.

This design means the crate is self-contained: it never reads
`DEP_TREE_SITTER_INCLUDE` or any other environment variable from the tree-sitter
crate itself.

---

## File map

| File | Purpose |
|------|---------|
| `patches/tree-sitter-angular/Cargo.toml` + `Cargo.toml.orig` | Version constraints bumped to 0.26 |
| `patches/tree-sitter-angular/bindings/rust/lib.rs` | `LanguageFn::from_raw` pattern for 0.26 API |
| `patches/tree-sitter-angular/src/tree_sitter/*.h` | Replaced with tree-sitter 0.26 C headers |
| `patches/tree-sitter-angular/bindings/rust/build.rs` | **Unchanged** — the crate's original build script |
| `src/angular.rs` | `Language` / `LanguageExt` impl; calls `tree_sitter_angular::language()` |

---

## Upgrading in the future

When a version of `tree-sitter-angular` that natively supports tree-sitter
`0.26.x` is published to crates.io:

1. Remove the `[patch.crates-io]` block from `Cargo.toml`.
2. Delete `patches/tree-sitter-angular/`.
3. Run `cargo update` — Cargo will resolve the new version directly.

`src/angular.rs` needs no changes because it already calls
`tree_sitter_angular::language()`, which is the standard API the upstream crate
will export.
