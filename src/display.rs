use crate::color;
use crate::graph;
use crate::node;

use colorful::Color;
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct Display<'a> {
  graph: &'a graph::Graph,
  patterns: Vec<Regex>,
  succinct: bool,
  color: bool,
  max_depth: i32,
  ignore_unknown: bool,
  sort_children: bool,
}

impl<'a> Display<'a> {
  pub fn new(
    graph: &'a graph::Graph,
    patterns: &[String],
    succinct: bool,
    color: bool,
    max_depth: i32,
    ignore_unknown: bool,
    sort_children: bool,
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
      sort_children,
    }
  }

  fn filter_root_nodes(&self, nodes: Vec<&'a node::Node>) -> Vec<&'a node::Node> {
    // Succinct is applied only if no pattern is specified
    if self.succinct && self.patterns.is_empty() {
      let mut degree = HashMap::<&'a node::Node, usize>::new();
      for (_, edge) in self.graph.edges.iter() {
        for v in edge {
          degree.entry(v).or_insert(1);
        }
      }
      let mut queue = Vec::<&'a node::Node>::new();
      let mut head = 0;
      let mut filtered_nodes = Vec::<&'a node::Node>::new();
      nodes.iter().for_each(|node| match degree.get(node) {
        None | Some(0) => {
          queue.push(node);
          filtered_nodes.push(node);
        }
        _ => (),
      });
      while queue.len() < nodes.len() {
        if head == queue.len() {
          let node = nodes
            .iter()
            .max_by_key(|node| match degree.get(*node) {
              None | Some(0) => 0,
              _ => {
                if let Some(nodes) = self.graph.get_adjacencies(node) {
                  nodes.len()
                } else {
                  0
                }
              }
            })
            .unwrap();
          *degree.get_mut(node).unwrap() = 0;
          queue.push(node);
          filtered_nodes.push(node);
        }
        loop {
          let u = queue[head];
          head += 1;
          if let Some(nodes) = self.graph.get_adjacencies(u) {
            for v in nodes {
              let deg_v = degree.get_mut(v).unwrap();
              if *deg_v > 0 {
                *deg_v = 0;
                queue.push(v);
              }
            }
          }
          if head == queue.len() {
            break;
          }
        }
      }
      filtered_nodes
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
            color::color("unknown", Color::LightYellow)
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
          text += "    ";
        } else {
          text += "│   ";
        }
      });
      let mut indent = text.clone();
      if end[depth] {
        text += "└── ";
        indent += "    ";
      } else {
        text += "├── ";
        indent += "│   ";
      }

      let is_recursive = visited.contains(&u.name);
      let color = if self.color {
        if u.alias {
          Some(Color::LightGreen)
        } else if is_recursive {
          Some(Color::LightCyan)
        } else if u.location.is_empty() {
          Some(Color::LightYellow)
        } else {
          None
        }
      } else {
        None
      };

      let lines = format!("{}", u);
      let lines = lines.split('\n');
      let mut first_line = true;
      const DISPLAY_WIDTH: usize = 100;
      for line in lines {
        let mut i = 0;
        let name_indent = line.find(' ').unwrap_or(0);
        while i < line.len() {
          let mut nxt_i = i + DISPLAY_WIDTH - indent.len();
          if i > 0 {
            nxt_i -= name_indent;
          }
          let mut refined_nxt_i = nxt_i;
          while refined_nxt_i < line.len() && line.as_bytes()[refined_nxt_i] != b']' {
            refined_nxt_i += 1;
          }
          nxt_i = line.len().min(refined_nxt_i + 1);

          if !first_line {
            text += &indent;
          } else {
            first_line = false;
          }
          if i > 0 {
            text += &" ".repeat(name_indent);
          }

          if let Some(color) = color {
            text.push_str(&color::color(line[i..nxt_i].trim(), color));
          } else {
            text += &line[i..nxt_i];
          }
          text += "\n";
          i = nxt_i;
        }
      }

      if !is_recursive {
        visited.insert(u.name.clone());
        if let Some(nodes) = self.graph.get_adjacencies(u) {
          let mut nodes = nodes.clone();
          if self.ignore_unknown {
            nodes = nodes
              .into_iter()
              .filter(|v| !v.location.is_empty())
              .collect::<Vec<_>>();
          }
          if self.sort_children {
            nodes.sort_by_key(|node| node.name.to_lowercase());
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
    text += "}\n";
    text
  }

  fn node_to_dot(&self, u: &node::Node) -> String {
    let mut text = String::new();
    if self.max_depth != 0 {
      if let Some(nodes) = self.graph.get_adjacencies(u) {
        for v in nodes {
          if !self.ignore_unknown || !v.location.is_empty() {
            text.push_str(&format!("\"{}\"->\"{}\";", u.name, v.name));
          }
        }
      }
    }
    text
  }
}
