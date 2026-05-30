use std::env;
use std::path::PathBuf;

fn main() {
    // tree-sitter emits `cargo:include=<path>` (the `include/` dir) from its
    // build.rs.  The internal C headers grammar authors need (parser.h,
    // array.h, alloc.h …) live in the sibling `src/` directory.
    // We expose them as `tree_sitter/<name>.h` by copying them into
    // a shim directory: <OUT_DIR>/ts_headers/tree_sitter/
    let ts_include = env::var("DEP_TREE_SITTER_INCLUDE")
        .expect("DEP_TREE_SITTER_INCLUDE not set — ensure tree-sitter is a direct dependency");

    let ts_src = PathBuf::from(&ts_include)
        .parent()
        .expect("DEP_TREE_SITTER_INCLUDE has no parent")
        .join("src");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let shim = out_dir.join("ts_headers").join("tree_sitter");
    std::fs::create_dir_all(&shim).unwrap();

    // Copy every .h from tree-sitter's src/ into the shim tree_sitter/ dir.
    for entry in std::fs::read_dir(&ts_src).unwrap().flatten() {
        let p = entry.path();
        if p.extension().map(|e| e == "h").unwrap_or(false) {
            std::fs::copy(&p, shim.join(p.file_name().unwrap())).unwrap();
        }
    }

    cc::Build::new()
        .include(out_dir.join("ts_headers"))            // tree_sitter/parser.h, array.h …
        .include("vendor/tree-sitter-angular/src")       // tag.h
        .file("vendor/tree-sitter-angular/src/parser.c")
        .file("vendor/tree-sitter-angular/src/scanner.c")
        .warnings(false)
        .compile("tree-sitter-angular");

    println!("cargo:rerun-if-changed=vendor/tree-sitter-angular/src/parser.c");
    println!("cargo:rerun-if-changed=vendor/tree-sitter-angular/src/scanner.c");
}
