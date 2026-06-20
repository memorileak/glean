use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Position(
  pub usize, // line
  pub usize, // column
);

#[derive(Debug, Clone, Serialize)]
pub struct Match {
  pub start: Position,
  pub end: Position,
}

#[derive(Debug, Clone, Serialize)]
pub struct Symbol {
  pub name: String,
  pub kind: String,
  pub start: Position,
  pub end: Position,
  pub children: Option<Vec<Symbol>>,
}
