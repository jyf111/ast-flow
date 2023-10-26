use crate::node;

use std::collections::HashMap;

#[derive(Default)]
pub struct Graph {
  pub nodes: HashMap<String, node::Node>,
  pub edges: HashMap<node::Node, Vec<node::Node>>,
}

impl Graph {
  pub fn new() -> Self {
    Graph::default()
  }

  pub fn add_node(&mut self, u: &node::Node) {
    if !self.nodes.contains_key(&u.name) {
      self.nodes.insert(u.name.clone(), u.clone());
    } else {
      self.nodes.get_mut(&u.name).unwrap().merge_node(u);
    }
  }

  pub fn get_node(&self, name: &str) -> Option<&node::Node> {
    self.nodes.get(name)
  }

  pub fn add_edge(&mut self, u: &node::Node, v: &node::Node) {
    if !self.edges.contains_key(u) {
      self.edges.insert(u.clone(), Vec::new());
    }
    self.edges.get_mut(u).unwrap().push(v.clone());
  }

  pub fn reverse(&self) -> Self {
    let mut reverse_graph = Graph::new();
    reverse_graph.nodes = self.nodes.clone();
    for (u, out_edges) in self.edges.iter() {
      for v in out_edges {
        reverse_graph.add_edge(v, u);
      }
    }
    reverse_graph
  }
}
