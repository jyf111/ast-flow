use crate::color;
use crate::graph;
use crate::node;

use colorful::Color;
use regex::Regex;
use std::collections::HashSet;

pub struct Display<'a> {
  graph: &'a graph::Graph,
  patterns: Vec<Regex>,
  succinct: bool,
  color: bool,
  max_depth: i32,
  ignore_unknown: bool,
}

impl<'a> Display<'a> {
  pub fn new(
    graph: &'a graph::Graph,
    patterns: &[String],
    succinct: bool,
    color: bool,
    max_depth: i32,
    ignore_unknown: bool,
  ) -> Display<'a> {
    Display {
      graph,
      patterns: patterns
        .iter()
        .map(|pattern| Regex::new(pattern).unwrap())
        .collect::<Vec<_>>(),
      succinct,
      color,
      max_depth,
      ignore_unknown,
    }
  }

  fn filter_root_nodes(&self, nodes: Vec<&'a node::Node>) -> Vec<&'a node::Node> {
    if self.succinct {
      nodes
        .into_iter()
        .filter(|node| {
          !self.graph.nodes.values().any(|u| {
            self.graph.edges.contains_key(u)
              && self.graph.edges.get(u).unwrap().iter().any(|v| *node == v)
          })
        })
        .collect::<Vec<_>>()
    } else {
      nodes
    }
  }

  fn filter_nodes(&self, nodes: Vec<&'a node::Node>) -> Vec<&'a node::Node> {
    nodes
      .into_iter()
      .filter(|node| {
        if self.patterns.is_empty() {
          true
        } else {
          self
            .patterns
            .iter()
            .any(|pattern| pattern.is_match(&node.name))
        }
      })
      .collect::<Vec<_>>()
  }

  pub fn to_text(&self) -> String {
    let nodes = self.filter_root_nodes(self.graph.nodes.values().collect::<Vec<_>>());
    let nodes = self.filter_nodes(nodes);

    let mut files = HashSet::new();
    for node in nodes.iter() {
      for loc in node.location.iter() {
        files.insert(&loc.file);
      }
    }
    let mut files = Vec::from_iter(files);
    files.sort_by_key(|file| file.to_lowercase());

    let mut text = String::new();
    for file in files {
      let mut nodes_in_file = nodes
        .iter()
        .filter(|node| node.location.iter().any(|loc| loc.file == *file))
        .collect::<Vec<_>>();
      if !nodes_in_file.is_empty() {
        nodes_in_file.sort_by_key(|node| node.name.to_lowercase());
        text.push_str(&format!(
          "{}\n",
          if self.color {
            color::color(file, Color::LightMagenta)
          } else {
            file.to_string()
          }
        ));
        let num = nodes_in_file.len();
        for (i, u) in nodes_in_file.into_iter().enumerate() {
          text.push_str(&self.node_to_text(u, vec![i + 1 == num], &mut HashSet::new()));
        }
        text.push('\n');
      }
    }
    if !self.ignore_unknown {
      let mut nodes_in_unknown = nodes
        .iter()
        .filter(|node| node.location.is_empty())
        .collect::<Vec<_>>();
      if !nodes_in_unknown.is_empty() {
        nodes_in_unknown.sort_by_key(|node| node.name.to_lowercase());
        text.push_str(&format!(
          "{}\n",
          if self.color {
            color::color("unknown", Color::LightRed)
          } else {
            "unknown".to_string()
          }
        ));
        let num = nodes_in_unknown.len();
        for (i, u) in nodes_in_unknown.into_iter().enumerate() {
          text.push_str(&self.node_to_text(u, vec![i + 1 == num], &mut HashSet::new()));
        }
      }
    }

    if text.ends_with('\n') {
      text.pop();
    }
    text
  }

  fn node_to_text(&self, u: &node::Node, end: Vec<bool>, visited: &mut HashSet<String>) -> String {
    let depth = end.len() - 1;
    if self.max_depth != -1 && depth as i32 > self.max_depth {
      String::default()
    } else {
      let mut text = String::new();
      (0..depth).for_each(|d| {
        if end[d] {
          text.push_str("    ")
        } else {
          text.push_str("│   ")
        }
      });
      if end[depth] {
        text.push_str("└── ")
      } else {
        text.push_str("├── ")
      }
      let is_recursive = visited.contains(&u.name);
      if u.location.len() > 1 {
        text.push_str(&color::color(&format!("{}\n", u), Color::LightYellow));
      } else if u.location.is_empty() {
        text.push_str(&color::color(&format!("{}\n", u), Color::LightRed));
      } else if is_recursive {
        text.push_str(&color::color(&format!("{}\n", u), Color::LightCyan));
      } else {
        text.push_str(&format!("{}\n", u));
      }
      if !is_recursive {
        visited.insert(u.name.clone());
        if self.graph.edges.contains_key(u) {
          let mut nodes = self.graph.edges.get(u).unwrap().clone();
          if self.ignore_unknown {
            nodes = nodes
              .into_iter()
              .filter(|v| !v.location.is_empty())
              .collect::<Vec<_>>();
          }
          let num = nodes.len();
          for (i, v) in nodes.into_iter().enumerate() {
            let mut end = end.clone();
            end.push(i + 1 == num);
            text.push_str(&self.node_to_text(&v, end, &mut visited.clone()));
          }
        }
      }
      text
    }
  }

  pub fn to_dot(&self) -> String {
    let nodes = self.filter_root_nodes(self.graph.nodes.values().collect::<Vec<_>>());
    let nodes = self.filter_nodes(nodes);

    let mut files = HashSet::new();
    for node in nodes.iter() {
      for loc in node.location.iter() {
        files.insert(&loc.file);
      }
    }

    let mut text = String::from(
      "digraph g {\nnode [margin=0,width=.5,height=.2];edge [arrowsize=.5,arrowhead=vee];\n",
    );
    for file in files {
      let mut nodes_in_file = nodes
        .iter()
        .filter(|node| node.location.iter().any(|loc| loc.file == *file))
        .collect::<Vec<_>>();
      if !nodes_in_file.is_empty() {
        nodes_in_file.sort_by_key(|node| node.name.to_lowercase());
        for u in nodes_in_file {
          text.push_str(&self.node_to_dot(u));
        }
      }
    }
    if !self.ignore_unknown {
      let mut nodes_in_unknown = nodes
        .iter()
        .filter(|node| node.location.is_empty())
        .collect::<Vec<_>>();
      if !nodes_in_unknown.is_empty() {
        nodes_in_unknown.sort_by_key(|node| node.name.to_lowercase());
        for u in nodes_in_unknown {
          text.push_str(&self.node_to_dot(u));
        }
      }
    }
    text.push_str("}\n");
    text
  }

  fn node_to_dot(&self, u: &node::Node) -> String {
    let mut text = String::new();
    if self.max_depth != 0 && self.graph.edges.contains_key(u) {
      let nodes = self.graph.edges.get(u).unwrap().clone();
      for v in nodes {
        if !self.ignore_unknown || !v.location.is_empty() {
          text.push_str(&format!("\"{}\"->\"{}\";", u.name, v.name));
        }
      }
    }
    text
  }
}
