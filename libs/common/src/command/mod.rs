use crate::{console::CONSOLE, struct_gen};
pub mod types;
use types::CommandType;

pub trait Operation<O = ()> {
  const NAME: &'static str;
  const PARENT: Option<&'static str> = None;

  fn usage(status: i32) {
    std::process::exit(status);
  }

  fn main(&self) -> std::io::Result<()>;

  fn validate(&self) -> std::io::Result<()> {
    Ok(())
  }
}

struct_gen! {
  pub struct Command use Clone {
    let example: Option<String> = None;
    let about: String = "No description provided.".to_owned();
    let command_type: CommandType = CommandType::Other {
      name: String::new()
    };
  }

  mod preset_costructors {
    pub fn help_flag() -> Self {
      Self::flag('H', "help", "Prints this message.")
    }
  }

  mod argument_constructor {
    pub fn option(short: impl Into<char>, long: impl Into<String>, example: impl Into<String>, about: impl Into<String>) -> Self {
      let long = long.into().to_lowercase();
      if long.is_empty() {
        CONSOLE.panic("`long` cannot be empty.");
      }

      Self {
        example: Some(example.into()),
        about: about.into(),
        command_type: CommandType::Flag {
          short: short.into(),
          long,
        }
      }
    }

    pub fn long_option(long: impl Into<String>, example: impl Into<String>, about: impl Into<String>) -> Self {
      let long = long.into().to_lowercase();
      if long.is_empty() {
        CONSOLE.panic("`long` cannot be empty.");
      }

      Self {
        example: Some(example.into()),
        about: about.into(),
        command_type: CommandType::LongFlag {
          long,
        }
      }
    }

    pub fn flag(short: impl Into<char>, long: impl Into<String>, about: impl Into<String>) -> Self {
      let long = long.into();
      if long.is_empty() {
        CONSOLE.panic("`long` cannot be empty.");
      }

      Self {
        example: None,
        about: about.into(),
        command_type: CommandType::Flag {
          short: short.into().to_ascii_uppercase(),
          long: long.to_lowercase(),
        }
      }
    }

    pub fn long_flag(long: impl Into<String>, about: impl Into<String>) -> Self {
      let long = long.into().to_lowercase();
      if long.is_empty() {
        CONSOLE.panic("`long` cannot be empty.");
      }

      Self {
        example: None,
        about: about.into(),
        command_type: CommandType::LongFlag {
          long,
        }
      }
    }
  }

  mod operation_constructor {
    pub fn operation(name: impl Into<String>, example: impl Into<String>, about: impl Into<String>) -> Self {
      let name = name.into().to_lowercase();
      if name.is_empty() {
        CONSOLE.panic("`name` cannot be empty.");
      }

      Self {
        example: Some(example.into()),
        about: about.into(),
        command_type: CommandType::Operation {
          name,
        }
      }
    }

    pub fn subcommand(name: impl Into<String>, about: impl Into<String>) -> Self {
      let name = name.into().to_lowercase();
      if name.is_empty() {
        CONSOLE.panic("`name` cannot be empty.");
      }

      Self {
        example: None,
        about: about.into(),
        command_type: CommandType::Operation {
          name,
        }
      }
    }

    pub fn variadic(name: impl Into<String>, about: impl Into<String>) -> Self {
      Self {
        example: None,
        about: about.into(),
        command_type: CommandType::Variadic {
          name: format!("...{}", name.into()),
        }
      }
    }
  }
}
