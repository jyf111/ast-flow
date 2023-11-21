use anyhow::{Context, Result};
use std::fs;
use std::iter;
use std::path;

pub struct SyntaxTree {
  pub file: String,
  source: String,
  tree: tree_sitter::Tree,
}

pub struct NodeIterator<'a> {
  cursor: tree_sitter::TreeCursor<'a>,
  dir: Direction,
}

#[derive(Eq, PartialEq)]
enum Direction {
  Down,
  Up,
  None,
}

impl SyntaxTree {
  pub fn new(file: path::PathBuf, ignore_macros: &[String]) -> Result<Self> {
    let mut parser = tree_sitter::Parser::new();
    parser
      .set_language(tree_sitter_cpp::language())
      .context("Failed to load Cpp grammar!")?;

    let mut source = fs::read_to_string(&file)
      .with_context(|| format!("Failed to read file: {}!", file.display()))?;
    ignore_macros.iter().for_each(|ignore_macro| {
      source = source.replace(ignore_macro, &"".repeat(ignore_macro.len())); // Replace with blank placeholder
    });

    if let Some(tree) = parser.parse(&source, None) {
      Ok(SyntaxTree {
        file: file.display().to_string(),
        source,
        tree,
      })
    } else {
      Err(anyhow::anyhow!("Failed to parse file: {}!", file.display()))
    }
  }

  pub fn iter(&self) -> NodeIterator {
    NodeIterator::new(&self.tree)
  }

  pub fn source(&self, node: &tree_sitter::Node) -> &str {
    &self.source[node.start_byte()..node.end_byte()]
  }
}

impl<'a> NodeIterator<'a> {
  fn new(tree: &'a tree_sitter::Tree) -> Self {
    NodeIterator {
      cursor: tree.walk(),
      dir: Direction::None,
    }
  }
}

// Depth first search sequence
impl<'a> iter::Iterator for NodeIterator<'a> {
  type Item = tree_sitter::Node<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.dir == Direction::None {
        self.dir = Direction::Down;
        return Some(self.cursor.node());
      } else if (self.dir == Direction::Down && self.cursor.goto_first_child())
        || self.cursor.goto_next_sibling()
      {
        self.dir = Direction::None;
      } else {
        self.dir = Direction::Up;
        if !self.cursor.goto_parent() {
          return None;
        }
      }
    }
  }
}
