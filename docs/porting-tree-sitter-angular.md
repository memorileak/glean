# Porting `tree-sitter-angular` to glean

## Why vendoring was necessary

The published crate [`tree-sitter-angular`](https://crates.io/crates/tree-sitter-angular)
(latest version **0.9.2** at time of writing) declares a hard dependency on
`tree-sitter ~0.25`.  This project uses `tree-sitter = "0.26.9"`.  Cargo cannot
reconcile `~0.25` with `0.26.x` because tree-sitter follows a semver-breaking
release cadence (the C ABI changes between minor versions), so `cargo add
tree-sitter-angular` fails with an irreconcilable conflict.

There is no newer version of `tree-sitter-angular` on crates.io that supports
the `0.26` series.

---

## Solution: vendor the C sources and compile them in-tree

A tree-sitter grammar is ultimately two C files (`parser.c` and `scanner.c`)
plus a header (`tag.h` for Angular's external scanner).  These files contain no
Rust code and are completely independent of the *crate* version — they only
depend on the tree-sitter C API headers, which ship with every version of the
`tree-sitter` crate itself.

By compiling those C files directly against the headers from our own
`tree-sitter 0.26.9` dependency we get a compatible binary with no crate-level
version conflict.

### Step 1 — Download the grammar source files

The three C source files were downloaded from the upstream GitHub repository
[dlvandenberg/tree-sitter-angular](https://github.com/dlvandenberg/tree-sitter-angular)
at tag `v0.9.2` and placed into:

```
vendor/
└── tree-sitter-angular/
    └── src/
        ├── parser.c    # generated parser (≈ 684 KB)
        ├── scanner.c   # hand-written external scanner
        └── tag.h       # tag constants used by scanner.c
```

`tree-sitter-angular` was removed from `[dependencies]` in `Cargo.toml`.

### Step 2 — Add build infrastructure

`Cargo.toml` was updated with two entries:

```toml
[package]
build = "build.rs"          # tell Cargo to run our build script

[build-dependencies]
cc = "1"                    # C compiler wrapper crate
```

`tree-sitter-language = "0.1.7"` was also added to `[dependencies]` — this
crate provides the `LanguageFn` type that tree-sitter 0.26 uses to represent a
language handle (see Step 4).

### Step 3 — Write `build.rs`

`build.rs` compiles the two C files into a static library called
`tree-sitter-angular`.  The tricky part is supplying the right include paths so
that `#include "tree_sitter/parser.h"` inside `parser.c` resolves correctly.

See the [section below](#what-buildrs-does) for a detailed explanation.

### Step 4 — Write `src/angular.rs`

A thin Rust module declares the FFI symbol that `build.rs` produced and wraps it
in the types expected by `ast-grep-core`:

```rust
// Declare the C symbol emitted by the compiled grammar.
unsafe extern "C" {
    fn tree_sitter_angular() -> *const ();
}

// Wrap it in tree-sitter 0.26's LanguageFn handle.
const ANGULAR_LANGUAGE: LanguageFn =
    unsafe { LanguageFn::from_raw(tree_sitter_angular) };

// Implement ast-grep's Language + LanguageExt traits so Angular
// can be used anywhere a SupportLang can be used.
#[derive(Clone, Copy, Debug)]
pub struct Angular;

impl Language for Angular { /* kind_to_id, field_to_id, build_pattern */ }
impl LanguageExt for Angular {
    fn get_ts_language(&self) -> TSLanguage {
        ANGULAR_LANGUAGE.into()
    }
}
```

---

## What `build.rs` does

```
build.rs
│
├─ 1. Locate tree-sitter's C headers via DEP_TREE_SITTER_INCLUDE
├─ 2. Copy those headers into a shim directory so the grammar can find them
└─ 3. Compile parser.c + scanner.c into a static library
```

### 1 · Locate the tree-sitter C headers

When `tree-sitter` itself is compiled, its own `build.rs` emits a Cargo
metadata line:

```
cargo:include=/path/to/tree-sitter-0.26.9/include
```

Cargo transforms this into an environment variable available to *downstream*
build scripts:

```
DEP_TREE_SITTER_INCLUDE=/path/to/tree-sitter-0.26.9/include
```

`build.rs` reads this variable to discover where tree-sitter's headers live:

```rust
let ts_include = env::var("DEP_TREE_SITTER_INCLUDE")
    .expect("DEP_TREE_SITTER_INCLUDE not set …");
```

### 2 · Set up the header shim

`DEP_TREE_SITTER_INCLUDE` points to the `include/` subdirectory of the
tree-sitter source tree, but **the C headers that grammar files actually
`#include`** (such as `parser.h`, `array.h`, `alloc.h`) live in the sibling
`src/` directory.

Grammar C code uses this include form:

```c
#include "tree_sitter/parser.h"
```

So the compiler needs to find a directory that contains a `tree_sitter/`
subdirectory holding `parser.h`.  We create one inside `OUT_DIR` (Cargo's
per-build scratch space) by:

1. Walking up one level from `include/` to reach the tree-sitter source root.
2. Descending into `src/` to find the real headers.
3. Creating `<OUT_DIR>/ts_headers/tree_sitter/`.
4. Copying every `.h` file from `src/` into that directory.

```rust
let ts_src = PathBuf::from(&ts_include)
    .parent()          // tree-sitter-0.26.9/
    .unwrap()
    .join("src");      // tree-sitter-0.26.9/src/

let shim = out_dir.join("ts_headers").join("tree_sitter");
std::fs::create_dir_all(&shim).unwrap();

for entry in std::fs::read_dir(&ts_src).unwrap().flatten() {
    let p = entry.path();
    if p.extension().map(|e| e == "h").unwrap_or(false) {
        std::fs::copy(&p, shim.join(p.file_name().unwrap())).unwrap();
    }
}
```

After this step the shim directory looks like:

```
<OUT_DIR>/ts_headers/
└── tree_sitter/
    ├── parser.h
    ├── array.h
    ├── alloc.h
    └── …
```

Passing `<OUT_DIR>/ts_headers` as an include path to the C compiler makes
`#include "tree_sitter/parser.h"` resolve correctly.

### 3 · Compile the grammar

The `cc` crate compiles both C files into a static library named
`tree-sitter-angular` (Cargo links it automatically as `libtree-sitter-angular.a`):

```rust
cc::Build::new()
    .include(out_dir.join("ts_headers"))           // tree_sitter/parser.h, array.h …
    .include("vendor/tree-sitter-angular/src")      // tag.h (Angular's own header)
    .file("vendor/tree-sitter-angular/src/parser.c")
    .file("vendor/tree-sitter-angular/src/scanner.c")
    .warnings(false)          // the generated C is noisy; suppress it
    .compile("tree-sitter-angular");
```

Two `rerun-if-changed` directives ensure Cargo only rebuilds the library when
the vendored C sources are actually modified:

```rust
println!("cargo:rerun-if-changed=vendor/tree-sitter-angular/src/parser.c");
println!("cargo:rerun-if-changed=vendor/tree-sitter-angular/src/scanner.c");
```

---

## File map

| File | Purpose |
|------|---------|
| `build.rs` | Compiles vendored C grammar against tree-sitter 0.26 headers |
| `src/angular.rs` | FFI binding + `Language` / `LanguageExt` impl for ast-grep |
| `vendor/tree-sitter-angular/src/parser.c` | Generated tree-sitter parser (from v0.9.2) |
| `vendor/tree-sitter-angular/src/scanner.c` | Hand-written external scanner |
| `vendor/tree-sitter-angular/src/tag.h` | Tag constants used by the scanner |

---

## Upgrading in the future

When a version of `tree-sitter-angular` that supports tree-sitter `0.26.x` (or
whichever version this project uses) is published to crates.io, the vendoring
approach can be replaced by a normal `cargo add tree-sitter-angular` and the
following files can be deleted:

- `build.rs` (or simplified to remove the Angular-specific logic)
- `vendor/tree-sitter-angular/`
- The `cc = "1"` build-dependency
- The `tree-sitter-language = "0.1.7"` dependency (unless needed elsewhere)

`src/angular.rs` would be replaced by a call to the crate's exported
`language()` function, and `ANGULAR_LANGUAGE` would be obtained from there
instead of via `LanguageFn::from_raw`.
