use common::{command::{types::ArgumentType, Operation}, console::CONSOLE, string::StringV2, NAME};
use std_v2::{derive::Command, struct_gen};
use clap::{error::{ContextKind, ErrorKind}, Parser, Subcommand};

mod subcommand;

use subcommand::help::Options as HelpCommand;
use subcommand::version::Options as VersionCommand;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Subcommand, Command)]
#[non_exhaustive]
pub enum Commands {
  #[subcommand("Show this message")]
  Help(HelpCommand),
  #[subcommand("Show the current version of compass")]
  Version(VersionCommand),
  #[command(external_subcommand)]
  External(Vec<String>),
}

pub fn execute_command<Command: Operation>(command: &Command) {
  match command.validate() {
    Ok(_) => {
      if let Err(e) = command.main() {
        CONSOLE.panic(format!("{e}"));
      }
    },
    Err(e) => CONSOLE.panic(format!("{e}"))
  }
}

struct_gen! {
  #[command(name = "compass", disable_help_flag = true, disable_help_subcommand = true, subcommand_required = false)]
  pub struct Options use Parser, Command {
    #[command(subcommand)]
    let command: Option<Commands> = None;

    #[arg(short, long), help_flag]
    let help: bool = false;

    #[arg(short, long), version_flag]
    let version: bool = false;
  }

  impl Operation {
    const NAME: &'static str = "ctr";

    fn usage(status: i32) {
      let version = {
        #[allow(unused_mut)]
        let mut ver = VERSION.to_string();

        #[cfg(debug_assertions)]
        ver.push_str("-dev");

        ver
      };
      CONSOLE.print(format!(
        "{desc} <brightblack>(v{version})</brightblack>\n",
        desc = env!("CARGO_PKG_DESCRIPTION").replace("ctr", "<magenta>ctr</magenta>")
      ));

      CONSOLE.print_usage::<Options>(vec![
        ArgumentType::Flags,
        ArgumentType::Operand {
          name: "operation".to_owned()
        }
      ]);

      CONSOLE.print_operation_collection({
        let mut operations = Options::operations();
        operations.extend(Commands::operations());

        operations
      });

      std::process::exit(status);
    }

    fn main(self: &Self) -> std::io::Result<()> {
      self.help.then(|| Self::usage(0));
      self.version.then(|| execute_command(&VersionCommand {
        help: false
      }));

      if self.command.is_none() {
        let version = {
          #[allow(unused_mut)]
          let mut ver = VERSION.to_string();

          #[cfg(debug_assertions)]
          ver.push_str("-dev");

          ver
        };

        CONSOLE.print(format!(
          "{desc} <brightblack>(v{version})</brightblack>\n",
          desc = env!("CARGO_PKG_DESCRIPTION").replace("ctr", "<magenta>ctr</magenta>")
        ));
        CONSOLE.print(format!("Use <magenta>{} help</magenta> for additional information", NAME.to_lowercase()));

        std::process::exit(0);
      }

      if let Some(ref command) = self.command {
        match command {
          Commands::Help(options) => execute_command(options),
          Commands::Version(options) => execute_command(options),
          Commands::External(args) => {
            let arg0_default = String::new();
            let arg0 = args.first().unwrap_or(&arg0_default);

            let commands = Commands::operations().iter()
              .flat_map(|e| e.iter().map(|op| op.command_type().name().to_string()))
              .collect::<Vec<String>>();

            CONSOLE.exit(
              if let Some(suggestion) = StringV2::from(arg0).nearest(commands) {
                format!("Did you mean <magenta>{suggestion}</magenta>?")
              } else {
                format!("Unknown operation: {arg0}")
              }
            );
          }
        }
      }

      Ok(())
    }
  }
}

pub type ClapError = clap::error::Error<clap::error::DefaultFormatter>;
pub fn handle_error(e: ClapError) -> ! {
  match e.kind() {
    ErrorKind::UnknownArgument => {
      if let Some(argument) = e.get(ContextKind::InvalidArg) {
        let arg = argument.to_string();

        CONSOLE.exit(format!("The {} '{}' is not recognized.", {
          if arg.starts_with("-") {
            "argument"
          } else {
            "operation"
          }
        }, arg));
      }
    },
    ErrorKind::ArgumentConflict => {
      if let Some(argument) = e.get(ContextKind::InvalidArg) {
        let arg = argument.to_string();
        if arg.starts_with("-") {
          CONSOLE.exit(format!("The argument '{arg}' is used more than once."));
        }
      }
    },
    ErrorKind::InvalidValue => {
      if let Some(argument) = e.get(ContextKind::InvalidArg) {
        let arg = argument.to_string();

        if arg.starts_with("-") {
          if let Some(invalid_value) = e.get(ContextKind::InvalidValue) {
            let value = invalid_value.to_string();

            if value.is_empty() {
              if let Some(argument_name) = arg.split_whitespace().next() {
                CONSOLE.exit(format!("The argument '{argument_name}' requires a value."));
              } else {
                std::process::exit(1);
              }
            }

            CONSOLE.exit(format!("The value '{value}' is not valid for the argument '{arg}'."));
          }
        }
      }
    },
    _ => CONSOLE.exit(format!("{e}"))
  }

  CONSOLE.exit("Unkown operation behavior");
}

fn run(argv: Vec<String>) {
  match Options::try_parse_from(argv.iter()) {
    Ok(cli) => execute_command::<Options>(&cli),
    Err(e) => {
      handle_error(e);
    }
  }
}

fn main() {
  run(std::env::args().collect::<Vec<String>>());
}
