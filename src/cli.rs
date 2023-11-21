use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  /// Generate function call graph
  Call(Arg),
  /// Generate class hierarchy graph
  Class(Arg),
}

#[derive(Args)]
pub struct Arg {
  /// Specify the path to the file or directory to analyze
  #[arg(short = 'p', long = "path", default_value("./"))]
  pub path: std::path::PathBuf,

  /// Specify a path to exclude (this option can be provided multiple times)
  #[arg(long = "exclude-path")]
  pub exclude_globset: Vec<String>,

  /// Show only entries matching the given regex pattern (this option can be provided multiple times)
  #[arg(short = 'w', long = "word")]
  pub patterns: Vec<String>,

  /// Whether to use color when displaying
  #[arg(long = "color", default_value_t = true)]
  pub color: bool,

  /// Control the depth of the displayed tree
  #[arg(short = 'd', long = "depth", default_value_t = -1)]
  pub depth: i32,

  /// Whether to exclude some useless paths (e.g., test/, benchmark/)
  #[arg(long = "no-default-exclude-path", default_value_t = false)]
  pub no_exclude: bool,

  /// Display in text mode
  #[arg(long = "text", default_value_t = true)]
  pub text: bool,

  /// Display in dot mode
  #[arg(long = "dot", default_value_t = false)]
  pub dot: bool,

  /// Whether to display in succinct mode (only show root entries)
  #[arg(long = "succinct", default_value_t = false)]
  pub succinct: bool,

  /// Display in reverse direction
  #[arg(short = 'r', long = "reverse", default_value_t = false)]
  pub reverse: bool,

  /// Whether to ignore unknown entries
  #[arg(long = "ignore-unknown", default_value_t = false)]
  pub ignore_unknown: bool,

  /// Ignore a macro that confuses tree-sitter (this option can be provided multiple times)
  #[arg(long = "ignore-macro")]
  pub ignore_macros: Vec<String>,

  /// Whether to sort child nodes by name
  #[arg(long = "sort-children", default_value_t = false)]
  pub sort_children: bool,
}
