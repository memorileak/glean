use std::{env, path::Path};
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

fn language_for_ext(ext: &str) -> Option<(Language, &'static str)> {
  match ext {
    "rs" => Some((
      tree_sitter_rust::LANGUAGE.into(),
      include_str!("../queries/rust/aerial.scm"),
    )),
    "js" | "mjs" | "cjs" | "jsx" => Some((
      tree_sitter_javascript::LANGUAGE.into(),
      include_str!("../queries/javascript/aerial.scm"),
    )),
    "ts" | "mts" | "cts" => Some((
      tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
      include_str!("../queries/typescript/aerial.scm"),
    )),
    "tsx" => Some((
      tree_sitter_typescript::LANGUAGE_TSX.into(),
      include_str!("../queries/tsx/aerial.scm"),
    )),
    "html" | "htm" => Some((
      tree_sitter_html::LANGUAGE.into(),
      include_str!("../queries/html/aerial.scm"),
    )),
    "css" | "scss" | "less" => Some((
      tree_sitter_css::LANGUAGE.into(),
      include_str!("../queries/css/aerial.scm"),
    )),
    "json" => Some((
      tree_sitter_json::LANGUAGE.into(),
      include_str!("../queries/json/aerial.scm"),
    )),
    "yaml" | "yml" => Some((
      tree_sitter_yaml::LANGUAGE.into(),
      include_str!("../queries/yaml/aerial.scm"),
    )),
    "py" => Some((
      tree_sitter_python::LANGUAGE.into(),
      include_str!("../queries/python/aerial.scm"),
    )),
    _ => None,
  }
}

/// Truncate to the first line, capped at `max` chars.
fn truncate(s: &str, max: usize) -> String {
  let line = s.lines().next().unwrap_or("").trim();
  if line.len() > max {
    format!("{}…", &line[..max])
  } else {
    line.to_string()
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    eprintln!("Usage: {} <file>", args[0]);
    std::process::exit(1);
  }

  let file_path = &args[1];
  let ext = Path::new(file_path)
    .extension()
    .and_then(|e| e.to_str())
    .unwrap_or("");

  let (language, query_src) = match language_for_ext(ext) {
    Some(v) => v,
    None => {
      eprintln!("Unsupported file extension: .{ext}");
      std::process::exit(1);
    }
  };

  let source = std::fs::read_to_string(file_path).unwrap_or_else(|e| {
    eprintln!("Failed to read {file_path}: {e}");
    std::process::exit(1);
  });

  let mut parser = Parser::new();
  parser
    .set_language(&language)
    .expect("Failed to set language");

  let tree = parser.parse(&source, None).expect("Failed to parse source");

  let query = Query::new(&language, query_src).unwrap_or_else(|e| {
    eprintln!("Failed to compile query: {e}");
    std::process::exit(1);
  });

  let capture_names = query.capture_names().to_vec();

  let sep = "─".repeat(60);
  println!("File     : {file_path}");
  println!("Language : .{ext}");
  println!("Patterns : {}", query.pattern_count());
  println!("Captures : {}", capture_names.join(", "));
  println!("{sep}");

  let mut cursor = QueryCursor::new();
  let mut iter = cursor.matches(&query, tree.root_node(), source.as_bytes());
  let mut match_count = 0usize;

  while let Some(m) = iter.next() {
    match_count += 1;

    // Collect #set! property settings for this pattern (e.g. kind = "Function")
    let props = query.property_settings(m.pattern_index);
    let kind = props
      .iter()
      .find(|p| p.key.as_ref() == "kind")
      .and_then(|p| p.value.as_deref())
      .unwrap_or("?");

    // The human-readable name of the symbol
    let name_text = m
      .captures
      .iter()
      .find(|c| capture_names[c.index as usize] == "name")
      .map(|c| &source[c.node.byte_range()]);

    // Full implementation range.
    // When only one anchor exists, that node's span is used directly.
    // When both exist (e.g. CSS selector + block, or JS `const f = () => {}`),
    // take the outermost span: min(start_byte) → max(end_byte).
    let symbol_cap = m
      .captures
      .iter()
      .find(|c| capture_names[c.index as usize] == "symbol");
    let start_cap = m
      .captures
      .iter()
      .find(|c| capture_names[c.index as usize] == "start");

    let full_range = match (symbol_cap, start_cap) {
      (Some(sym), Some(sta)) => {
        let begin = sym.node.start_byte().min(sta.node.start_byte());
        let finish = sym.node.end_byte().max(sta.node.end_byte());
        // Re-derive line:col from byte offsets
        let start_node = if sym.node.start_byte() <= sta.node.start_byte() {
          sym.node
        } else {
          sta.node
        };
        let end_node = if sym.node.end_byte() >= sta.node.end_byte() {
          sym.node
        } else {
          sta.node
        };
        let _ = (begin, finish); // byte offsets used only for the comparison above
        Some((start_node.start_position(), end_node.end_position()))
      }
      (Some(cap), None) | (None, Some(cap)) => {
        Some((cap.node.start_position(), cap.node.end_position()))
      }
      (None, None) => None,
    };

    print!(
      "Match #{:<3}  pattern={:<3}  kind={:<12}",
      match_count, m.pattern_index, kind
    );
    if let Some(name) = name_text {
      print!("  name=`{name}`");
    }
    if let Some((start, end)) = full_range {
      print!(
        "  impl=[{}:{} → {}:{}]",
        start.row + 1,
        start.column + 1,
        end.row + 1,
        end.column + 1,
      );
    }
    println!();

    // All captures for this match
    for cap in m.captures.iter() {
      let cap_name = &capture_names[cap.index as usize];
      let node = cap.node;
      let start = node.start_position();
      let end = node.end_position();
      let text = truncate(&source[node.byte_range()], 60);
      println!(
        "  @{:<14} [{:>4}:{:<3}→{:>4}:{:<3}]  {}",
        cap_name,
        start.row + 1,
        start.column + 1,
        end.row + 1,
        end.column + 1,
        text,
      );
    }

    println!("{sep}");
  }

  println!("Total: {match_count} matches");
}
