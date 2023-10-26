use crate::analyzer;
use crate::cli;
use crate::display;
use crate::graph;
use crate::pathwalk;
use crate::syntaxtree::SyntaxTree;

pub struct Driver;

impl Driver {
  pub fn run<T: analyzer::Analyzer>(analyzer: &mut T, arg: &cli::Arg) -> String {
    let files = pathwalk::FileIterator::new(&arg.path, &arg.exclude_globset, arg.no_exclude);
    let syntax_trees = files.map(SyntaxTree::new).collect::<Vec<_>>();

    let mut graph = graph::Graph::new();
    syntax_trees.iter().for_each(|tree| {
      analyzer.extract_nodes(tree, &mut graph);
    });
    syntax_trees.iter().for_each(|tree| {
      analyzer.extract_edges(tree, &mut graph);
    });

    if arg.reverse {
      graph = graph.reverse();
    }

    let display = display::Display::new(
      &graph,
      &arg.patterns,
      arg.succinct,
      arg.color,
      arg.depth,
      arg.ignore_unknown,
    );
    if arg.dot {
      display.to_dot()
    } else {
      display.to_text()
    }
  }
}
