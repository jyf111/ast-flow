use crate::analyzer;
use crate::graph;
use crate::node;
use crate::syntaxtree;

use std::collections::HashMap;

type Call = node::Node;

pub struct CallAnalyzer {
  qualified_function_pool: HashMap<String, Option<String>>,
}

enum Context {
  ClassSpecifier(usize),
  ClassIdentifier(String),
  FunctionDefinition(usize),
  FunctionDeclarator,
  FunctionIdentifier(Call),
  CallExpression(usize),
}

impl CallAnalyzer {
  pub fn new() -> Self {
    CallAnalyzer {
      qualified_function_pool: HashMap::new(),
    }
  }
}

impl analyzer::Analyzer for CallAnalyzer {
  fn extract_nodes(&mut self, syntax_tree: &syntaxtree::SyntaxTree, graph: &mut graph::Graph) {
    let mut context = Vec::<Context>::new();

    syntax_tree.iter().for_each(|node| match context.len() {
      0 if matches!(node.kind(), "struct_specifier" | "class_specifier") => {
        let source = syntax_tree.source(&node);
        if source.find('{').is_some() && source.ends_with('}') {
          context.push(Context::ClassSpecifier(node.end_byte()));
        }
      }
      1 if matches!(node.kind(), "type_identifier") => {
        if let Context::ClassSpecifier(_) = context[0] {
          context.push(Context::ClassIdentifier(String::from(
            syntax_tree.source(&node),
          )));
        }
      }
      0 | 2 if matches!(node.kind(), "function_definition") => {
        let compact_source = syntax_tree
          .source(&node)
          .chars()
          .filter(|c| !c.is_whitespace())
          .collect::<String>();
        if !compact_source.contains("=delete;") {
          context.push(Context::FunctionDefinition(node.end_byte()));
        }
      }
      1 | 3 if matches!(node.kind(), "function_declarator") => {
        if let Some(Context::FunctionDefinition(_)) = context.last() {
          context.push(Context::FunctionDeclarator);
        }
      }
      2 if matches!(
        node.kind(),
        "identifier"
          | "field_identifier"
          | "qualified_identifier"
          | "destructor_name"
          | "operator_name"
      ) =>
      {
        if let Context::FunctionDeclarator = context[1] {
          context.clear();
          let function = &format!("{}()", syntax_tree.source(&node));
          let call = Call::new(function, &syntax_tree.file, node.start_position().row + 1);
          graph.add_node(&call);
          if let Some(index) = function.rfind(':') {
            let qualified_function = function;
            let function = &qualified_function[index + 1..];
            if !self.qualified_function_pool.contains_key(function) {
              self.qualified_function_pool.insert(
                String::from(function),
                Some(String::from(qualified_function)),
              );
            } else {
              *self.qualified_function_pool.get_mut(function).unwrap() = None;
            }
          }
        }
      }
      4 if matches!(
        node.kind(),
        "identifier"
          | "field_identifier"
          | "qualified_identifier"
          | "destructor_name"
          | "operator_name"
      ) =>
      {
        if let Context::FunctionDeclarator = context[3] {
          context.truncate(2);
          if let Context::ClassIdentifier(ref class) = context[1] {
            let function = &format!("{}()", syntax_tree.source(&node));
            let qualified_function = &format!("{}::{}", class, function);
            let call = Call::new(
              qualified_function,
              &syntax_tree.file,
              node.start_position().row + 1,
            );
            graph.add_node(&call);
            if !self.qualified_function_pool.contains_key(function) {
              self
                .qualified_function_pool
                .insert(function.clone(), Some(qualified_function.clone()));
            } else {
              *self.qualified_function_pool.get_mut(function).unwrap() = None;
            }
          } else {
            context.clear();
          }
        }
      }
      1.. => {
        if let Some(Context::FunctionDefinition(pos)) = context.get(2) {
          if node.start_byte() + 1 >= *pos {
            context.truncate(2);
          }
        }

        if let Context::ClassSpecifier(pos) = context[0] {
          if node.start_byte() + 1 >= pos {
            context.clear();
          }
        } else if let Context::FunctionDefinition(pos) = context[0] {
          if node.start_byte() + 1 >= pos {
            context.clear();
          }
        }
      }
      _ => (),
    });
  }

  fn extract_edges(&mut self, syntax_tree: &syntaxtree::SyntaxTree, graph: &mut graph::Graph) {
    let mut context = Vec::<Context>::new();
    let mut call_stack = Vec::<(usize, Call)>::new();

    syntax_tree.iter().for_each(|node| match context.len() {
      0 if matches!(node.kind(), "struct_specifier" | "class_specifier") => {
        let source = syntax_tree.source(&node);
        if source.find('{').is_some() && source.ends_with('}') {
          context.push(Context::ClassSpecifier(node.end_byte()));
        }
      }
      1 if matches!(node.kind(), "type_identifier") => {
        if let Context::ClassSpecifier(_) = context[0] {
          context.push(Context::ClassIdentifier(String::from(
            syntax_tree.source(&node),
          )));
        }
      }
      0 | 2 if matches!(node.kind(), "function_definition") => {
        let compact_source = syntax_tree
          .source(&node)
          .chars()
          .filter(|c| !c.is_whitespace())
          .collect::<String>();
        if !compact_source.contains("=default;") && !compact_source.contains("=delete;") {
          context.push(Context::FunctionDefinition(node.end_byte()));
        }
      }
      1 | 3 if matches!(node.kind(), "function_declarator") => {
        if let Some(Context::FunctionDefinition(_)) = context.last() {
          context.push(Context::FunctionDeclarator);
        }
      }
      2 if matches!(
        node.kind(),
        "identifier"
          | "field_identifier"
          | "qualified_identifier"
          | "destructor_name"
          | "operator_name"
      ) =>
      {
        if let Context::FunctionDeclarator = context[1] {
          if let Some(call) = graph.get_node(&format!("{}()", syntax_tree.source(&node))) {
            context[1] = Context::FunctionIdentifier(call.clone());
          } else {
            context.clear();
          }
        }
      }
      4 if matches!(
        node.kind(),
        "identifier"
          | "field_identifier"
          | "qualified_identifier"
          | "destructor_name"
          | "operator_name"
      ) =>
      {
        if let Context::FunctionDeclarator = context[3] {
          if let Context::ClassIdentifier(ref class) = context[1] {
            if let Some(call) =
              graph.get_node(&format!("{}::{}()", class, syntax_tree.source(&node)))
            {
              context[3] = Context::FunctionIdentifier(call.clone());
            } else {
              context.truncate(2);
            }
          } else {
            context.clear();
          }
        }
      }
      2 | 4 if matches!(node.kind(), "call_expression") => {
        if let Some(Context::FunctionIdentifier(_)) = context.last() {
          context.push(Context::CallExpression(node.end_byte()));
        }
      }
      3 | 5
        if matches!(
          node.kind(),
          "identifier" | "qualified_identifier" | "field_expression"
        ) =>
      {
        if let Some(Context::CallExpression(pos)) = context.last() {
          let pos = *pos;
          context.pop();
          if let Some(Context::FunctionIdentifier(_)) = context.last() {
            let source = syntax_tree.source(&node);
            let call_function = if let Some(index) = source.find('<') {
              &source[..index]
            } else {
              source
            };
            let mut function_name = if let Some(mut index) = call_function.rfind('.') {
              if let Some(index2) = call_function.rfind("->") {
                if index2 > index {
                  index = index2 + 1;
                }
              }
              &call_function[index + 1..]
            } else if let Some(index) = call_function.rfind("->") {
              &call_function[index + 2..]
            } else {
              call_function
            };
            if let Some(index) = function_name.find('(') {
              function_name = &function_name[..index];
            }
            let function = &format!("{}()", function_name);
            if node.kind() == "field_expression" {
              if let Some(Some(qualified_function)) = self.qualified_function_pool.get(function) {
                if let Some(callee) = graph.get_node(qualified_function) {
                  call_stack.push((pos, callee.clone()));
                }
              } else {
                let function = &format!("unknown::{}", function);
                let callee = Call::new_without_loc(function);
                graph.add_node(&callee);
                call_stack.push((pos, callee));
              }
            } else if let Some(callee) = graph.get_node(function) {
              call_stack.push((pos, callee.clone()));
            } else if let Some(Some(qualified_function)) =
              self.qualified_function_pool.get(function)
            {
              if let Some(callee) = graph.get_node(qualified_function) {
                call_stack.push((pos, callee.clone()));
              }
            } else {
              let callee = Call::new_without_loc(function);
              graph.add_node(&callee);
              call_stack.push((pos, callee));
            }
          } else {
            context.pop();
            context.pop();
          }
        } else {
          context.pop();
        }
      }
      1.. => {
        if let Some(Context::FunctionIdentifier(call)) = context.last() {
          if let Some((pos, callee)) = call_stack.last() {
            if node.start_byte() + 1 >= *pos {
              graph.add_edge(call, callee);
              call_stack.pop();
            }
          }
        }

        if let Some(Context::FunctionDefinition(pos)) = context.get(2) {
          if node.start_byte() + 1 >= *pos {
            context.truncate(2);
          }
        }

        if let Context::ClassSpecifier(pos) = context[0] {
          if node.start_byte() + 1 >= pos {
            context.clear();
          }
        } else if let Context::FunctionDefinition(pos) = context[0] {
          if node.start_byte() + 1 >= pos {
            context.clear();
          }
        }
      }
      _ => (),
    });
  }
}
