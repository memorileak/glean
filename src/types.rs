use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Position(
  pub usize, // line
  pub usize, // column
);

#[derive(Clone, Serialize)]
pub struct Match {
  pub start: Position,
  pub end: Position,
}
