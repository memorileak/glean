mod angular;

use anyhow::Result;
use ast_grep_core::language::Language;
use ast_grep_language::{Html, JavaScript, LanguageExt, Rust, SupportLang, TypeScript};
use std::fmt;
use std::fs;
use std::path::Path;
use tree_sitter::Parser;

use angular::Angular;

// ── helpers ───────────────────────────────────────────────────────────────────

struct Hit {
  file: String,
  text: String,
  line: usize,
}

impl fmt::Display for Hit {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "  [{}:{}] {:?}",
      self.file,
      self.line + 1,
      self.text.trim()
    )
  }
}

fn search<L: LanguageExt + Clone>(
  lang: L,
  src: &str,
  file: &str,
  pattern: &'static str,
) -> Vec<Hit> {
  let root = lang.ast_grep(src);
  root
    .root()
    .find_all(pattern)
    .map(|m| Hit {
      file: file.to_string(),
      text: m.text().to_string(),
      line: m.start_pos().line(),
    })
    .collect()
}

fn print_section(title: &str, hits: &[Hit]) {
  println!("\n{}", title);
  println!("{}", "─".repeat(title.len()));
  if hits.is_empty() {
    println!("  (no matches)");
  } else {
    for h in hits {
      println!("{h}");
    }
  }
}

// ── searches ──────────────────────────────────────────────────────────────────

fn search_javascript(src: &str) -> Vec<(String, Vec<Hit>)> {
  let file = "sample.js";
  vec![
    (
      "console.log($$$ARGS)".to_string(),
      search(JavaScript, src, file, "console.log($$$ARGS)"),
    ),
    (
      "describe($NAME, $BODY)".to_string(),
      search(JavaScript, src, file, "describe($NAME, $BODY)"),
    ),
    (
      "new $CLASS($$$ARGS)".to_string(),
      search(JavaScript, src, file, "new $CLASS($$$ARGS)"),
    ),
  ]
}

fn search_typescript(src: &str) -> Vec<(String, Vec<Hit>)> {
  let file = "sample.ts";
  vec![
    (
      "async $NAME($$$ARGS): Promise<$RET>".to_string(),
      search(TypeScript, src, file, "async $NAME($$$ARGS): Promise<$RET>"),
    ),
    (
      "class $NAME implements $IFACE".to_string(),
      search(TypeScript, src, file, "class $NAME implements $IFACE"),
    ),
    (
      "new Map<$K, $V>()".to_string(),
      search(TypeScript, src, file, "new Map<$K, $V>()"),
    ),
  ]
}

fn search_rust(src: &str) -> Vec<(String, Vec<Hit>)> {
  let file = "sample.rs";
  vec![
    (
      "fn $NAME(&self) -> $RET".to_string(),
      search(Rust, src, file, "fn $NAME(&self) -> $RET"),
    ),
    (
      "impl $TRAIT for $TYPE".to_string(),
      search(Rust, src, file, "impl $TRAIT for $TYPE"),
    ),
    (
      "pub struct $NAME".to_string(),
      search(Rust, src, file, "pub struct $NAME"),
    ),
  ]
}

fn search_html(src: &str) -> Vec<(String, Vec<Hit>)> {
  let file = "sample.html";
  vec![
    (
      "<a href=\"$HREF\">$TEXT</a>".to_string(),
      search(Html, src, file, "<a href=\"$HREF\">$TEXT</a>"),
    ),
    (
      "<h1>$TEXT</h1>".to_string(),
      search(Html, src, file, "<h1>$TEXT</h1>"),
    ),
    (
      "<button type=\"$TYPE\">$LABEL</button>".to_string(),
      search(Html, src, file, "<button type=\"$TYPE\">$LABEL</button>"),
    ),
  ]
}

/// Angular templates are searched with the custom Angular language backed by
/// the vendored tree-sitter-angular C grammar (compatible with tree-sitter 0.26).
///
/// Important: Angular attribute bindings (e.g. `[(ngModel)]="x"`) are NOT valid
/// as standalone patterns — the Angular grammar only parses them inside an element
/// tag. Patterns must therefore include the surrounding element.  For elements with
/// multiple attributes, every attribute must be present in the pattern so that the
/// structural match succeeds.
fn search_angular(src: &str) -> Vec<(String, Vec<Hit>)> {
  let file = "sample.angular.html";
  vec![
    // *ngIf structural directive on <section> (single attribute).
    (
      r#"<section *ngIf="$COND">$$$</section>"#.to_string(),
      search(
        Angular,
        src,
        file,
        r#"<section *ngIf="$COND">$$$</section>"#,
      ),
    ),
    // *ngIf structural directive on <ng-container> with else clause.
    (
      r#"<ng-container *ngIf="$COND">$$$</ng-container>"#.to_string(),
      search(
        Angular,
        src,
        file,
        r#"<ng-container *ngIf="$COND">$$$</ng-container>"#,
      ),
    ),
    // *ngFor structural directive — the <div> also carries class="item-card".
    (
      r#"<div *ngFor="$EXPR" class="item-card">$$$</div>"#.to_string(),
      search(
        Angular,
        src,
        file,
        r#"<div *ngFor="$EXPR" class="item-card">$$$</div>"#,
      ),
    ),
    // Two-way binding [(ngModel)] — the <input> has three attributes.
    (
      r#"<input type="text" [(ngModel)]="$VAL" [placeholder]="$HINT" />"#.to_string(),
      search(
        Angular,
        src,
        file,
        r#"<input type="text" [(ngModel)]="$VAL" [placeholder]="$HINT" />"#,
      ),
    ),
    // Delete button: [disabled] + (click) event binding.
    (
      r#"<button class="btn btn-danger" [disabled]="$GUARD" (click)="$HANDLER">$$$</button>"#
        .to_string(),
      search(
        Angular,
        src,
        file,
        r#"<button class="btn btn-danger" [disabled]="$GUARD" (click)="$HANDLER">$$$</button>"#,
      ),
    ),
    // Reactive form: [formGroup] + (ngSubmit) bindings.
    (
      r#"<form [formGroup]="$GROUP" (ngSubmit)="$SUBMIT">$$$</form>"#.to_string(),
      search(
        Angular,
        src,
        file,
        r#"<form [formGroup]="$GROUP" (ngSubmit)="$SUBMIT">$$$</form>"#,
      ),
    ),
    // Interpolation {{ expr }} — the interpolation node is a first-class
    // Angular AST node and can be matched without an element wrapper.
    (
      r#"{{ $EXPR }}"#.to_string(),
      search(Angular, src, file, r#"{{ $EXPR }}"#),
    ),
  ]
}

// ── SupportLang dispatch ──────────────────────────────────────────────────────

fn dispatch_support_lang(path: &Path, src: &str) {
  let Some(lang) = SupportLang::from_path(path) else {
    return;
  };
  let file = path.file_name().unwrap().to_str().unwrap();
  // Example: pick a language-appropriate pattern and search.
  let (pattern, hits): (&'static str, Vec<Hit>) = match lang {
    SupportLang::JavaScript => ("await $EXPR", search(JavaScript, src, file, "await $EXPR")),
    SupportLang::TypeScript => ("await $EXPR", search(TypeScript, src, file, "await $EXPR")),
    SupportLang::Rust => (
      "self.$METHOD($$$)",
      search(Rust, src, file, "self.$METHOD($$$)"),
    ),
    _ => return,
  };
  if !hits.is_empty() {
    print_section(
      &format!("[SupportLang dispatch] `{pattern}` in {file}"),
      &hits,
    );
  }
}

// ── main ──────────────────────────────────────────────────────────────────────

/// Print the tree-sitter sexp of a source file parsed with the given language.
/// Useful during development to discover node kinds for new grammars.
fn debug_ast<L: LanguageExt + Clone>(lang: L, src: &str) {
  let mut parser = Parser::new();
  parser
    .set_language(&lang.get_ts_language())
    .expect("failed to set language");
  let tree = parser.parse(src.as_bytes(), None).unwrap();
  println!("{}", tree.root_node().to_sexp());
}

fn main() -> Result<()> {
  let fixtures = Path::new("fixtures");

  let js = fs::read_to_string(fixtures.join("sample.js"))?;
  let ts = fs::read_to_string(fixtures.join("sample.ts"))?;
  let rs = fs::read_to_string(fixtures.join("sample.rs"))?;
  let html = fs::read_to_string(fixtures.join("sample.html"))?;
  let angular_html = fs::read_to_string(fixtures.join("sample.angular.html"))?;

  // Uncomment to inspect the Angular template AST while developing new patterns:
  // debug_ast(Angular, &angular_html);

  println!("╔══════════════════════════════════════════════════════╗");
  println!("║     ast-grep Pattern Search POC — glean project      ║");
  println!("╚══════════════════════════════════════════════════════╝");

  for (pattern, hits) in search_javascript(&js) {
    print_section(&format!("[JS] {pattern}"), &hits);
  }
  for (pattern, hits) in search_typescript(&ts) {
    print_section(&format!("[TS] {pattern}"), &hits);
  }
  for (pattern, hits) in search_rust(&rs) {
    print_section(&format!("[Rust] {pattern}"), &hits);
  }
  for (pattern, hits) in search_html(&html) {
    print_section(&format!("[HTML] {pattern}"), &hits);
  }
  for (pattern, hits) in search_angular(&angular_html) {
    print_section(&format!("[Angular] {pattern}"), &hits);
  }

  // SupportLang enum dispatch demo
  for entry in fs::read_dir(fixtures)? {
    let path = entry?.path();
    if path.extension().map(|e| e == "html").unwrap_or(false) {
      continue; // angular + html handled above
    }
    if let Ok(src) = fs::read_to_string(&path) {
      dispatch_support_lang(&path, &src);
    }
  }

  Ok(())
}
