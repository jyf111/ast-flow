mod analyzer;
mod cli;
mod color;
mod cpp;
mod display;
mod driver;
mod graph;
mod node;
mod pathwalk;
mod syntaxtree;

use clap::Parser;
use cpp::call::CallAnalyzer;
use cpp::class::ClassAnalyzer;

fn main() {
  let mut pager = pager::Pager::with_pager("less -i -r -F");
  pager.setup();

  let mut cli = cli::Cli::parse();
  match &mut cli.command {
    cli::Command::Class(arg) | cli::Command::Call(arg) => {
      if arg.color && !pager.is_on() {
        arg.color = false;
      }
    }
  }
  print!(
    "{}",
    match cli.command {
      cli::Command::Class(arg) => {
        let mut class_analyzer = ClassAnalyzer::new();
        driver::Driver::run(&mut class_analyzer, &arg)
      }
      cli::Command::Call(arg) => {
        let mut call_analyzer = CallAnalyzer::new();
        driver::Driver::run(&mut call_analyzer, &arg)
      }
    }
  )
}
