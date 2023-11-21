[![Github Actions](https://github.com/jyf111/ast-flow/actions/workflows/build.yaml/badge.svg)](https://github.com/jyf111/ast-flow/actions/workflows/build.yaml)

# ast-flow

ast-flow is a CLI tool written in Rust for static analysis of C/C++ codebases, to help read the source code.
It provides class hierarchy graph and function call graph to visualize the code.

## Examples

- Show the hierarchy graph of all classes containing `Socket`

  `ast-flow class -p ./libsponge -w "Socket"`

  [![class-demo1.png](https://z1.ax1x.com/2023/11/21/pia20BV.png)](https://imgse.com/i/pia20BV)

  Green denotes name alias.

- Show the call graph of all functions containing `connect`, limit the depth to less than or equal to 1

  `ast-flow call -p ./libsponge -w "connect" -d 1`

  [![call-demo1.png](https://z1.ax1x.com/2023/11/21/pia2wn0.png)](https://imgse.com/i/pia2wn0)

  Yellow denotes functions with unknown definition locations (most are library calls).

- Show the call graph of all functions related to planner in nebula

  `ast-flow call -p ./src/graph/planner/plan`

  [![call-demo2.png](https://z1.ax1x.com/2023/11/21/piaR7ZV.png)](https://imgse.com/i/piaR7ZV)

  Cyan denotes recursive calls.

- Visualize the hierarchy graph of all classes in LevelDB using Graphviz

  `ast-flow class --dot | fdp -Tpng -o class.png`

  [![class-demo2.png](https://z1.ax1x.com/2023/11/21/piaRGb6.png)](https://imgse.com/i/piaRGb6)

## Details

ast-flow employes Tree-sitter to build ASTs for each file in the specified directory.
It then traverses each AST to extract all class and function definitions, and finally gathers relationship information from base class declarations and function call expressions.

ast-flow is faster and more accurate than regular expression-based methods, since it understands complex templates and name aliases.

## Install

```shell
$ cargo install ast-flow --path .
$ ast-flow --help
Usage: ast-flow <COMMAND>

Commands:
  call   Generate function call graph
  class  Generate class hierarchy graph
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
  ast-flow call --help
Generate function call graph
$ ast-flow call --help
Generate function call graph

Usage: ast-flow call [OPTIONS]

Options:
  -p, --path <PATH>                     Specify the path to the file or directory to analyze [default: ./]
      --exclude-path <EXCLUDE_GLOBSET>  Specify a path to exclude (this option can be provided multiple times)
  -w, --word <PATTERNS>                 Show only entries matching the given regex pattern (this option can be provided multiple times)
      --color                           Whether to use color when displaying
  -d, --depth <DEPTH>                   Control the depth of the displayed tree [default: -1]
      --no-default-exclude-path         Whether to exclude some useless paths (e.g., test/, benchmark/)
      --text                            Display in text mode
      --dot                             Display in dot mode
      --succinct                        Whether to display in succinct mode (only show root entries)
  -r, --reverse                         Display in reverse direction
      --ignore-unknown                  Whether to ignore unknown entries
      --ignore-macro <IGNORE_MACROS>    Ignore a macro that confuses tree-sitter (this option can be provided multiple times)
      --sort-children                   Whether to sort child nodes by name
  -h, --help                            Print help
```

## Known Issues

- Tree-sitter lacks type information. When there are duplicate symbols, they cannot be distinguished. In such cases, ast-flow just stupidly lists all possibilities
  - Certainly, we can leverage more powerful weapons, e.g., LSP and clang AST, but I opt to keep this tool simple and fast, yet sufficient for code browsing
- Tree-sitter doesn't understand C macros, which may cause confusion
  - You can try the `--ignore-macro` option to ignore an annoying macro (e.g., MAYBE_UNUSED)
- In theory, ast-flow can analyze any language, but currently, it only supports C/C++. To extend support to other languages, you simply need to implement the `analyzer` trait
