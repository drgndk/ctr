use crate::{enum_gen, string::StringV2};
enum_gen! {
  pub enum CommandType use Clone {
    Flag {
      let short: char = ' ';
      let name: String = String::new();
    },
    LongFlag {
      let name: String = String::new();
    },
    Operation {
      let name: String = String::new();
    },
    Variadic {
      let name: String = String::new();
    },
    Other {
      let name: String = String::new();
    }
  }

  mod implementation {
    pub fn name(self: &Self) -> String {
      match self {
        Self::Flag { name, .. } => name.clone(),
        Self::LongFlag { name, .. } => name.clone(),
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
    Operand {
      let name: String = String::new();
    },
    Variadic {
      let name: String = String::new();
    },
  }

  mod implementation {
    pub fn to_str_v2(self: &Self) -> StringV2 {
      match self {
        Self::Operand { name } => StringV2::from(name).push_effect("brightblue"),
        Self::Variadic { name } => StringV2::from(format!("...{}", name)).push_effect("brightgreen"),
        Self::Flags => StringV2::from("[...flags]").push_effect("brightmagenta")
      }
    }
  }
}
