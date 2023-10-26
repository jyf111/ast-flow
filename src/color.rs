use colorful::{Color, Colorful};

pub fn color(s: &str, color: Color) -> String {
  match std::env::var_os("TERM") {
    None => return s.to_string(),
    Some(k) => {
      if k == "dumb" {
        return s.to_string();
      }
    }
  }

  if std::env::var_os("NO_COLOR").is_some() {
    s.to_string()
  } else {
    format!("{}", s.color(color))
  }
}
