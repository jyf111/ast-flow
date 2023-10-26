use crate::graph;
use crate::syntaxtree;

pub trait Analyzer {
  fn extract_nodes(&mut self, syntax_tree: &syntaxtree::SyntaxTree, graph: &mut graph::Graph);

  fn extract_edges(&mut self, syntax_tree: &syntaxtree::SyntaxTree, graph: &mut graph::Graph);
}
