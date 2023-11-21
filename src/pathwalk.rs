use anyhow::{Context, Result};
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
  pub fn new(root: &path::PathBuf, exclude_globset: &[String], no_exclude: bool) -> Result<Self> {
    let mut exclude_builder = GlobSetBuilder::new();
    let mut exclude_globset = exclude_globset.to_owned();
    if !no_exclude {
      exclude_globset.push("*benchmark*".to_string());
      exclude_globset.push("*build*".to_string());
      exclude_globset.push("*contrib*".to_string());
      exclude_globset.push("*example*".to_string());
      exclude_globset.push("*test*".to_string());
      exclude_globset.push("*thirdparty*".to_string());
      exclude_globset.push("*third[-_]party*".to_string());
      exclude_globset.push("*deps*".to_string());
    }
    for pattern in exclude_globset {
      match Glob::new(&pattern) {
        Err(e) => eprintln!("[Warning] {}", e),
        Ok(glob) => {
          exclude_builder.add(glob);
        }
      };
    }

    Ok(FileIterator {
      root: root.clone(),
      visited: false,
      stack: if root.is_file() {
        vec![]
      } else {
        vec![fs::read_dir(root)
          .with_context(|| format!("Failed to read directory: \"{}\"!", root.display()))?]
      },
      exclude_globset: exclude_builder
        .build()
        .context("Failed to build GlobSet!")?,
    })
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
