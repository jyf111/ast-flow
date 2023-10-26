use std::hash::Hash;

#[derive(Clone, Default, Hash, Eq, PartialEq)]
pub struct Node {
  pub name: String,
  pub location: Vec<Location>,
}

#[derive(Clone, Default, Hash, Eq, PartialEq)]
pub struct Location {
  pub file: String,
  row: usize,
}

impl std::fmt::Display for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    if self.location.is_empty() {
      write!(f, "{}", self.name)
    } else {
      write!(
        f,
        "{} {}",
        self.name,
        self
          .location
          .iter()
          .map(|loc| format!("[{} +{}]", loc.file, loc.row))
          .collect::<Vec<String>>()
          .join(" ")
      )
    }
  }
}

impl Node {
  pub fn new(name: &str, file: &str, row: usize) -> Self {
    Node {
      name: String::from(name),
      location: vec![Location {
        file: String::from(file),
        row,
      }],
    }
  }

  pub fn new_without_loc(name: &str) -> Self {
    Node {
      name: String::from(name),
      location: vec![],
    }
  }

  pub fn merge_node(&mut self, node: &Node) {
    if self.name == node.name {
      self.location.extend_from_slice(&node.location);
      self.location.sort_by_key(|loc| loc.file.to_lowercase());
    }
  }
}
