use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
use std::iter;
use std::path;

pub struct FileIterator {
  root: path::PathBuf,
  visited: bool,
  stack: Vec<fs::ReadDir>,
  exclude_globset: GlobSet,
}

impl FileIterator {
  pub fn new(root: &path::PathBuf, exclude_globset: &Vec<String>, no_exclude: bool) -> Self {
    let mut exclude_builder = GlobSetBuilder::new();
    for glob in exclude_globset {
      exclude_builder.add(Glob::new(glob).unwrap());
    }
    if !no_exclude {
      exclude_builder.add(Glob::new("*benchmark*").unwrap());
      exclude_builder.add(Glob::new("*build*").unwrap());
      exclude_builder.add(Glob::new("*contrib*").unwrap());
      exclude_builder.add(Glob::new("*example*").unwrap());
      exclude_builder.add(Glob::new("*test*").unwrap());
      exclude_builder.add(Glob::new("*thirdparty*").unwrap());
      exclude_builder.add(Glob::new("*third[-_]party*").unwrap());
      exclude_builder.add(Glob::new("*deps*").unwrap());
    }

    FileIterator {
      root: root.clone(),
      visited: false,
      stack: if root.is_file() {
        vec![]
      } else {
        vec![fs::read_dir(root).unwrap()]
      },
      exclude_globset: exclude_builder.build().unwrap(),
    }
  }
}

impl iter::Iterator for FileIterator {
  type Item = path::PathBuf;

  fn next(&mut self) -> Option<Self::Item> {
    if self.root.is_file() {
      if !self.visited {
        self.visited = true;
        Some(self.root.clone())
      } else {
        None
      }
    } else {
      loop {
        if let Some(current_dir) = self.stack.last_mut() {
          if let Some(Ok(entry)) = current_dir.next() {
            let entry_path = entry.path();
            if let Ok(relative_path) = entry_path.strip_prefix(&self.root) {
              if entry_path.is_file() && self.exclude_globset.matches(relative_path).is_empty() {
                if let Some(ext) = relative_path.extension() {
                  if let Some(ext) = ext.to_str() {
                    if matches!(ext, "c" | "cc" | "cpp" | "h" | "hh" | "hpp") {
                      return Some(entry_path);
                    }
                  }
                }
              } else if entry_path.is_dir() && !relative_path.starts_with(".") {
                if let Ok(read_dir) = fs::read_dir(entry_path) {
                  self.stack.push(read_dir);
                }
              }
            }
          } else {
            self.stack.pop();
          }
        } else {
          return None;
        }
      }
    }
  }
}
