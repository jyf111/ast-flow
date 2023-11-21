use crate::analyzer;
use crate::cli;
use crate::display;
use crate::graph;
use crate::pathwalk;
use crate::syntaxtree;

pub struct Driver;

impl Driver {
  pub fn run<T: analyzer::Analyzer>(analyzer: &mut T, arg: &cli::Arg) -> String {
    let files = pathwalk::FileIterator::new(&arg.path, &arg.exclude_globset, arg.no_exclude);
    match files {
      Err(e) => {
        eprintln!("[Error] {}", e);
        String::default()
      }
      Ok(files) => {
        let syntax_trees = files
          .map(|file| syntaxtree::SyntaxTree::new(file, &arg.ignore_macros))
          .collect::<Vec<_>>();

        let mut graph = graph::Graph::new();
        syntax_trees.iter().for_each(|tree| match tree {
          Err(e) => eprintln!("[Error] {}", e),
          Ok(tree) => analyzer.extract_nodes(tree, &mut graph),
        });
        syntax_trees.into_iter().for_each(|ref tree| match tree {
          Err(e) => eprintln!("[Error] {}", e),
          Ok(tree) => analyzer.extract_edges(tree, &mut graph),
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
          arg.sort_children,
        );
        if arg.dot {
          display.to_dot()
        } else {
          display.to_text()
        }
      }
    }
  }
}
