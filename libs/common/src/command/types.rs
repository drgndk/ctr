use crate::{enum_gen, string::StringV2};
enum_gen! {
  pub enum CommandType use Clone {
    Flag(short: char, long: String),
    LongFlag(long: String),
    Operation(name: String),
    Variadic(name: String),
    Other(name: String),
  }

  mod implementation {
    pub fn name(self: &Self) -> String {
      match self {
        Self::Flag { long, .. } => long.clone(),
        Self::LongFlag { long } => long.clone(),
        Self::Operation { name, .. } => name.clone(),
        Self::Variadic { name, .. } => name.clone(),
        Self::Other { name, .. } => name.clone()
      }
    }

    pub fn to_color(self: &Self) -> &str {
      match self {
        Self::Flag { .. } => "magenta",
        Self::LongFlag { .. } => "magenta",
        Self::Operation { .. } => "blue",
        Self::Variadic { .. } => "green",
        Self::Other { .. } => "reset"
      }
    }
  }
}

enum_gen! {
  pub enum ArgumentType {
    Flags,
    Operand(name: String),
    Variadic(name: String),
  }

  mod implementation {
    pub fn to_str_v2(self: &Self) -> StringV2 {
      match self {
        Self::Operand { name } => StringV2::from(name).push_effect("brightblue"),
        Self::Variadic { name } => StringV2::from(format!("[...{}]", name)).push_effect("brightgreen"),
        Self::Flags => StringV2::from("...flags").push_effect("brightmagenta")
      }
    }
  }
}
