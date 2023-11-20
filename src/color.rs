use colorful::{Color, Colorful};

pub fn color(s: &str, color: Color) -> String {
  match std::env::var_os("TERM") {
    None => return String::from(s),
    Some(k) => {
      if k == "dumb" {
        return String::from(s);
      }
    }
  }

  if std::env::var_os("NO_COLOR").is_some() {
    String::from(s)
  } else {
    format!("{}", s.color(color))
  }
}
