use std::hash::Hash;

#[derive(Clone, Default, Hash, Eq, PartialEq)]
pub struct Node {
  pub name: String,
  pub alias: bool,
  pub location: Vec<Location>,
}

#[derive(Clone, Default, Hash, Eq, PartialEq)]
pub struct Location {
  pub file: String,
  row: usize,
}

impl Location {
  pub fn new(file: String, row: usize) -> Location {
    Location { file, row }
  }

  pub fn new_empty() -> Location {
    Location {
      file: String::new(),
      row: 0,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.file.is_empty()
  }
}

impl std::fmt::Display for Location {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "[{} +{}]", self.file, self.row)
  }
}

impl Node {
  pub fn new(name: &str, file: &str, row: usize) -> Self {
    Node {
      name: String::from(name),
      alias: false,
      location: vec![Location::new(String::from(file), row)],
    }
  }

  pub fn new_alias(name: &str, file: &str, row: usize) -> Self {
    Node {
      name: String::from(name),
      alias: true,
      location: vec![Location::new(String::from(file), row)],
    }
  }

  pub fn new_without_loc(name: &str) -> Self {
    Node {
      name: String::from(name),
      alias: false,
      location: vec![],
    }
  }

  // If `self.name` and `node.name` are different, we squeeze them together using '#' separator,
  // and `self` becomes undefined, which associates with multiple names
  pub fn merge_node(&mut self, node: &Node) {
    if self.name != node.name {
      self.name.push('#');
      self.name += &node.name;
      self.location.push(Location::new_empty());
    }
    self.location.extend_from_slice(&node.location);
  }

  pub fn is_undefined(&self) -> bool {
    self.name.find('#').is_some()
  }
}

impl std::fmt::Display for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    if !self.is_undefined() {
      if self.location.is_empty() {
        write!(f, "{}", self.name)
      } else {
        let mut sorted_location = self.location.clone();
        sorted_location.sort_by_key(|loc| loc.file.to_lowercase());
        write!(
          f,
          "{} {}",
          self.name,
          sorted_location
            .into_iter()
            .map(|loc| loc.to_string())
            .collect::<Vec<_>>()
            .join(" ")
        )
      }
    } else {
      let names = self.name.split('#');
      let mut text = String::new();
      let mut i = 0;
      for name in names {
        let mut nxt_i = i;
        while nxt_i < self.location.len() && !self.location[nxt_i].is_empty() {
          nxt_i += 1;
        }
        let mut sorted_location: Vec<Location> = self.location[i..nxt_i].into();
        sorted_location.sort_by_key(|loc| loc.file.to_lowercase());
        text.push_str(&format!(
          "{} {}",
          name,
          sorted_location
            .into_iter()
            .map(|loc| loc.to_string())
            .collect::<Vec<_>>()
            .join(" ")
        ));
        if nxt_i < self.location.len() {
          text.push('\n');
        }
        i = nxt_i + 1;
      }
      write!(f, "{}", text)
    }
  }
}
