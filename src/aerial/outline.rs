mod arl_css;
mod arl_html;
mod arl_javascript;
mod arl_json;
mod arl_python;
mod arl_rust;
mod arl_tsx;
mod arl_typescript;
mod arl_yaml;
mod registry;

pub use registry::ARL_LANGUAGES;

use std::collections::{HashMap, HashSet};

use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

use crate::types::{Position, Symbol};

pub trait ArlLanguage: Send + Sync {
  fn extensions(&self) -> &'static [&'static str];

  #[allow(unused_variables)]
  fn matches_filename(&self, filename: &str) -> bool {
    false
  }

  fn priority(&self) -> u8 {
    0
  }

  fn outline(&self, source: &str, query: &str) -> Vec<Symbol>;
}

/// Internal representation of a symbol with byte ranges for containment checking
#[derive(Debug, Clone)]
struct RawSymbol {
  name: String,
  kind: String,
  start_byte: usize,
  end_byte: usize,
  start_pos: Position,
  end_pos: Position,
}

impl RawSymbol {
  /// Check if this symbol's range fully contains another symbol's range
  fn contains(&self, other: &RawSymbol) -> bool {
    self.start_byte <= other.start_byte && other.end_byte <= self.end_byte && self != other
  }
}

impl PartialEq for RawSymbol {
  fn eq(&self, other: &Self) -> bool {
    self.start_byte == other.start_byte && self.end_byte == other.end_byte
  }
}

impl From<&RawSymbol> for Symbol {
  fn from(value: &RawSymbol) -> Self {
    let RawSymbol {
      name,
      kind,
      start_pos,
      end_pos,
      ..
    } = value.clone();
    Symbol {
      name,
      kind,
      start: start_pos,
      end: end_pos,
      children: None,
    }
  }
}

pub struct Outliner;

impl Outliner {
  pub fn outline(lang: &Language, source: &str, query: &str) -> Vec<Symbol> {
    // Parse the source code
    let mut parser = Parser::new();
    if parser.set_language(lang).is_err() {
      return vec![];
    }

    let tree = match parser.parse(source, None) {
      Some(t) => t,
      None => return vec![],
    };

    // Compile the query
    let query = match Query::new(lang, query) {
      Ok(q) => q,
      Err(_) => return vec![],
    };

    let capture_names = query.capture_names();

    // Collect all raw symbols with byte ranges
    let mut raw_symbols: Vec<RawSymbol> = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    while let Some(m) = matches.next() {
      // Extract the kind from property settings
      let props = query.property_settings(m.pattern_index);
      let kind = props
        .iter()
        .find(|p| p.key.as_ref() == "kind")
        .and_then(|p| p.value.as_deref())
        .unwrap_or("Unknown")
        .to_string();

      // Extract the name from @name capture
      let name_capture = m
        .captures
        .iter()
        .find(|c| capture_names[c.index as usize] == "name");

      let name = if let Some(cap) = name_capture {
        source[cap.node.byte_range()].to_string()
      } else {
        // Fallback for Rust impl items: try @rust_type
        if let Some(type_cap) = m
          .captures
          .iter()
          .find(|c| capture_names[c.index as usize] == "rust_type")
        {
          format!("impl {}", &source[type_cap.node.byte_range()])
        } else {
          continue; // Skip symbols without a name
        }
      };

      // Extract the range from @symbol and/or @start captures
      let symbol_cap = m
        .captures
        .iter()
        .find(|c| capture_names[c.index as usize] == "symbol");
      let start_cap = m
        .captures
        .iter()
        .find(|c| capture_names[c.index as usize] == "start");

      let (start_byte, end_byte, start_pos, end_pos) = match (symbol_cap, start_cap) {
        (Some(sym), Some(sta)) => {
          // Combined range: min start to max end
          let start_byte = sym.node.start_byte().min(sta.node.start_byte());
          let end_byte = sym.node.end_byte().max(sta.node.end_byte());
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
          let start_pos = start_node.start_position();
          let end_pos = end_node.end_position();
          (start_byte, end_byte, start_pos, end_pos)
        }
        (Some(cap), None) | (None, Some(cap)) => {
          let start_pos = cap.node.start_position();
          let end_pos = cap.node.end_position();
          (
            cap.node.start_byte(),
            cap.node.end_byte(),
            start_pos,
            end_pos,
          )
        }
        (None, None) => continue, // Skip symbols without range
      };

      raw_symbols.push(RawSymbol {
        name,
        kind,
        start_byte,
        end_byte,
        start_pos: Position(start_pos.row, start_pos.column),
        end_pos: Position(end_pos.row, end_pos.column),
      });
    }

    // Build hierarchical structure
    Self::build_hierarchy(raw_symbols)
  }

  /// Build a hierarchical tree from flat symbols based on containment
  fn build_hierarchy(mut raw_symbols: Vec<RawSymbol>) -> Vec<Symbol> {
    if raw_symbols.is_empty() {
      return vec![];
    }

    // Sort by start position, then by end position (descending) for stable nesting
    raw_symbols.sort_by(|a, b| {
      a.start_byte
        .cmp(&b.start_byte)
        .then_with(|| b.end_byte.cmp(&a.end_byte))
    });

    let raw_symbols = raw_symbols;
    let mut stack: Vec<usize> = Vec::new();
    let mut parents: Vec<(usize, usize)> = Vec::new();
    let mut roots: HashSet<usize> = HashSet::new();

    for (i, cur_sym) in raw_symbols.iter().enumerate() {
      if stack.is_empty() {
        roots.insert(i);
        stack.push(i);
        continue;
      }

      let mut parent_found = false;
      while !parent_found && !stack.is_empty() {
        let &top = stack.last().unwrap();
        let top_sym = &raw_symbols[top];
        if top_sym.contains(cur_sym) {
          parent_found = true;
          parents.push((i, top));
          stack.push(i);
        } else {
          stack.pop();
        }
      }

      if !parent_found {
        roots.insert(i);
        stack.push(i);
      }
    }

    let mut parent_to_children: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, p) in parents.into_iter() {
      parent_to_children.entry(p).or_insert(Vec::new()).push(i);
    }

    let parent_to_children = parent_to_children;
    let mut symbols: Vec<Symbol> = Vec::new();
    for r in roots.into_iter() {
      let symbol = Self::populate(r, &raw_symbols, &parent_to_children);
      symbols.push(symbol);
    }

    symbols
  }

  fn populate(
    n: usize,
    raw_symbols: &Vec<RawSymbol>,
    parent_to_children: &HashMap<usize, Vec<usize>>,
  ) -> Symbol {
    let children = if let Some(child_nodes) = parent_to_children.get(&n) {
      let mut children: Vec<Symbol> = Vec::with_capacity(child_nodes.len());
      for &c in child_nodes.iter() {
        let child = Self::populate(c, raw_symbols, parent_to_children);
        children.push(child);
      }
      Some(children)
    } else {
      None
    };
    let mut symbol: Symbol = (&raw_symbols[n]).into();
    symbol.children = children;
    symbol
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::aerial::queries::{ARL_QUERY_JAVASCRIPT, ARL_QUERY_RUST};

  #[test]
  fn test_outline_with_nested_symbols() {
    let source = r#"
      pub struct Point {
        pub x: f64,
        pub y: f64,
      }

      pub trait Shape {
        fn area(&self) -> f64;
      }

      impl Shape for Point {
        fn area(&self) -> f64 {
          0.0
        }
      }
    "#;

    let lang = tree_sitter_rust::LANGUAGE.into();
    let symbols = Outliner::outline(&lang, source, ARL_QUERY_RUST);

    println!("Test 1: {:?}", symbols);

    // Should find struct, trait, and impl
    assert!(!symbols.is_empty(), "Should find symbols");

    // Find the impl block
    let impl_symbol = symbols
      .iter()
      .find(|s| s.kind == "Class" && s.name.starts_with("impl"));

    assert!(impl_symbol.is_some(), "Should find impl block");

    let impl_symbol = impl_symbol.unwrap();

    // Verify impl has nested method
    assert!(
      impl_symbol.children.is_some(),
      "impl block should have children"
    );

    let children = impl_symbol.children.as_ref().unwrap();
    assert_eq!(children.len(), 1, "impl should have 1 method");
    assert_eq!(children[0].name, "area", "Method should be named 'area'");
    assert_eq!(
      children[0].kind, "Function",
      "Child should be Function kind"
    );

    // Verify struct and trait are at root level
    let root_names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(root_names.iter().any(|&n| n == "Point"));
    assert!(root_names.iter().any(|&n| n == "Shape"));
  }

  #[test]
  fn test_flat_symbols_without_nesting() {
    let source = r#"
      class A {
        getName() {
          function abc() {
             return "ABC";
          }
          function def() {
             return "DEF";
          }
          return "Name";
        }
        getAge() {
          return 0;
        }
      }

      class B {
        getName() {
          return "Name";
        }
        getSalutation() {
          return "Hello, Mr. ";
        }
      }
    "#;

    let lang = tree_sitter_javascript::LANGUAGE.into();
    let symbols = Outliner::outline(&lang, source, ARL_QUERY_JAVASCRIPT);
    println!("Test 2: {:#?}", symbols);

    assert_eq!(symbols.len(), 2, "Should find 2 root symbols");
  }
}
