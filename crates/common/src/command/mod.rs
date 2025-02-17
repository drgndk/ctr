use crate::struct_gen;

pub mod types;
use types::CommandType;

pub trait Operation<O = ()>  {
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

  mod custom_default_costructors {
    #[doc = "generate a Command struct for the `-h/--help` flag"]
    pub fn help_flag() -> Self {
      Self {
        example: None,
        about: "Prints this message.".to_owned(),
        command_type: CommandType::Flag {
          short: 'h',
          name: "help".to_owned(),
        }
      }
    }

    #[doc = "generate a Command struct for the `-v/--version` flag"]
    pub fn version_flag() -> Self {
      Self {
        example: None,
        about: "Prints the current version of compass.".to_owned(),
        command_type: CommandType::Flag {
          short: 'v',
          name: "version".to_owned(),
        }
      }
    }
  }

  mod argument_constructor {
    pub fn option(name: impl Into<String>, example: impl Into<String>, about: impl Into<String>) -> Self {
      let name = name.into().to_lowercase();
      if name.is_empty() {
        panic!("`long` cannot be empty.");
      }

      Self {
        example: Some(example.into()),
        about: about.into(),
        command_type: CommandType::Flag {
          short: name.chars().next().unwrap_or_else(|| {
            panic!("`short` cannot be empty.");
          }),
          name,
        }
      }
    }

    pub fn long_option(name: impl Into<String>, example: impl Into<String>, about: impl Into<String>) -> Self {
      let name = name.into().to_lowercase();
      if name.is_empty() {
        panic!("`long` cannot be empty.");
      }

      Self {
        example: Some(example.into()),
        about: about.into(),
        command_type: CommandType::LongFlag {
          name,
        }
      }
    }

    pub fn flag(name: impl Into<String>, about: impl Into<String>) -> Self {
      let name = name.into().to_lowercase();
      if name.is_empty() {
        panic!("`long` cannot be empty.");
      }

      Self {
        example: None,
        about: about.into(),
        command_type: CommandType::Flag {
          short: name.chars().next().unwrap_or_else(|| {
            panic!("`short` cannot be empty.");
          }),
          name,
        }
      }
    }

    pub fn long_flag(name: impl Into<String>, about: impl Into<String>) -> Self {
      let name = name.into().to_lowercase();
      if name.is_empty() {
        panic!("`long` cannot be empty.");
      }

      Self {
        example: None,
        about: about.into(),
        command_type: CommandType::LongFlag {
          name,
        }
      }
    }
  }

  mod operation_constructor {
    pub fn operation(name: impl Into<String>, example: impl Into<String>, about: impl Into<String>) -> Self {
      let name = name.into().to_lowercase();
      if name.is_empty() {
        panic!("`name` cannot be empty.");
      }

      Self {
        example: Some(example.into()),
        about: about.into(),
        command_type: CommandType::Operation {
          name,
        }
      }
    }

    pub fn command(name: impl Into<String>, about: impl Into<String>) -> Self {
      let name = name.into().to_lowercase();
      if name.is_empty() {
        panic!("`name` cannot be empty.");
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
