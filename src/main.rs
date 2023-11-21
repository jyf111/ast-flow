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
use std::io::{self, Write};

fn main() {
  let mut pager =
    pager::Pager::with_pager("less --raw-control-chars --ignore-case --quit-if-one-screen");
  pager.setup();

  let mut cli = cli::Cli::parse();
  match &mut cli.command {
    cli::Command::Class(arg) | cli::Command::Call(arg) => {
      if arg.color && !pager.is_on() {
        arg.color = false;
      }
    }
  }

  if let Err(e) = write!(
    io::stdout(),
    "{}",
    match cli.command {
      cli::Command::Class(arg) => driver::Driver::run(&mut ClassAnalyzer::new(), &arg),
      cli::Command::Call(arg) => driver::Driver::run(&mut CallAnalyzer::new(), &arg),
    }
  ) {
    eprintln!("[Warning] {}", e);
  }
}
