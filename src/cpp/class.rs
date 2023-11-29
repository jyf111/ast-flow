use crate::analyzer;
use crate::graph;
use crate::node;
use crate::syntaxtree;

type Class = node::Node;

pub struct ClassAnalyzer;

enum Context {
  ClassSpecifier(usize),
  ClassIdentifier(Class),
  BaseClassClause(usize),
  AliasDeclaration,
  TypeDefinition(usize),
}

impl ClassAnalyzer {
  pub fn new() -> Self {
    ClassAnalyzer {}
  }
}

impl analyzer::Analyzer for ClassAnalyzer {
  fn extract_nodes(&mut self, syntax_tree: &syntaxtree::SyntaxTree, graph: &mut graph::Graph) {
    let mut context = Vec::<Context>::new();

    syntax_tree.iter().for_each(|node| match context.len() {
      0 if matches!(node.kind(), "struct_specifier" | "class_specifier") => {
        let source = syntax_tree.source(&node);
        if source.find('{').is_some() && source.ends_with('}') {
          context.push(Context::ClassSpecifier(node.end_byte()));
        }
      }
      0 if matches!(node.kind(), "alias_declaration") => context.push(Context::AliasDeclaration),
      0 if matches!(node.kind(), "type_definition") => context.push(Context::TypeDefinition(0)),
      1 if matches!(node.kind(), "template_type")
        && matches!(context[0], Context::TypeDefinition(0)) =>
      {
        context[0] = Context::TypeDefinition(node.end_byte());
      }
      1 if matches!(node.kind(), "type_identifier") => match context[0] {
        Context::TypeDefinition(0) => context[0] = Context::TypeDefinition(node.end_byte()),
        Context::ClassSpecifier(_) | Context::ClassIdentifier(_) => {
          context[0] = Context::ClassIdentifier(Class::new(
            syntax_tree.source(&node),
            &syntax_tree.file,
            node.start_position().row + 1,
          ));
        }
        _ => {
          if let Context::TypeDefinition(end_byte) = context[0] {
            if node.end_byte() <= end_byte {
              return;
            }
          }
          context.pop();
          let class = Class::new_alias(
            syntax_tree.source(&node),
            &syntax_tree.file,
            node.start_position().row + 1,
          );
          graph.add_node(&class);
        }
      },
      1.. => {
        if matches!(node.kind(), "field_declaration_list" | "base_class_clause") {
          if context.len() == 1 {
            if let Context::ClassIdentifier(ref class) = context[0] {
              graph.add_node(class);
            }
          }
          context.clear();
        } else if let Context::ClassSpecifier(pos) = context[0] {
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

    syntax_tree.iter().for_each(|node| match context.len() {
      0 if matches!(node.kind(), "struct_specifier" | "class_specifier") => {
        let source = syntax_tree.source(&node);
        if source.find('{').is_some() && source.ends_with('}') {
          context.push(Context::ClassSpecifier(node.end_byte()));
        }
      }
      1 if matches!(node.kind(), "type_identifier") => {
        if let Some(class) = graph.get_node(syntax_tree.source(&node)) {
          context.push(Context::ClassIdentifier(class.clone()));
        } else {
          context.pop();
        }
      }
      2 if matches!(node.kind(), "base_class_clause") => context.push(Context::BaseClassClause(0)),
      3 if matches!(
        node.kind(),
        "type_identifier" | "template_type" | "qualified_identifier"
      ) =>
      {
        if let Context::ClassIdentifier(ref class) = context[1] {
          if let Context::BaseClassClause(pos) = context[2] {
            if node.start_byte() > pos {
              let source = syntax_tree.source(&node);
              let baseclass_name = if let Some(index) = source.find('<') {
                &source[..index]
              } else {
                source
              };
              let unqualified_baseclass_name = if let Some(index) = baseclass_name.rfind(':') {
                &baseclass_name[index + 1..]
              } else {
                baseclass_name
              };
              if let Some(baseclass) = graph.get_node(unqualified_baseclass_name) {
                let baseclass = baseclass.clone();
                graph.add_edge(&baseclass, class);
              } else {
                let baseclass = Class::new_without_loc(baseclass_name);
                graph.add_node(&baseclass);
                graph.add_edge(&baseclass, class);
              }
              context[2] = Context::BaseClassClause(node.end_byte());
            }
          }
        } else {
          context.clear();
        }
      }
      1.. => {
        if node.kind() == "field_declaration_list" {
          context.clear();
        } else if let Context::ClassSpecifier(pos) = context[0] {
          if node.start_byte() + 1 >= pos {
            context.clear();
          }
        }
      }
      _ => (),
    });
  }
}
