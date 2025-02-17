use common::console::CONSOLE;

pub mod help;
pub mod version;

#[allow(dead_code)]
pub fn check_conflicts(args: Vec<(&str, bool)>) {
  let mut set = vec![];
  for (name, is_set) in args {
    is_set.then(|| set.push(name));
  }

  if set.len() > 1 {
    let mut conflicts = String::new();
    for (index, name) in set.iter().enumerate() {
      conflicts.push_str(&format!("<blue>--{name}</blue>"));
      if index.ne(&set.len().saturating_sub(1)) {
        conflicts.push_str(if index.eq(&set.len().saturating_sub(2)) {
          " and "
        } else {
          ", "
        });
      }
    }

    CONSOLE.panic(format!("Cannot use {conflicts} at the same time"));
  }
}
