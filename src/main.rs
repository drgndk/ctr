use std::process::Command;

use clap::{
  error::{ContextKind, ContextValue, ErrorKind},
  Parser, Subcommand,
};
use std_v2::{
  command::Operation,
  console::CONSOLE,
  derive::Command,
  env::consts::{BINARY_NAME, IS_DEBUG, REPO_DIR, USER_CONFIG_DIR},
  string::StringV2,
  struct_gen,
};
mod operations;
use operations::{
  completions::Options as CompletionsCommand, env::Options as EnvCommand, help::Options as HelpCommand, info::Options as InfoCommand, notify::Options as NotifyCommand, run::Options as RunCommand, upgrade::Options as UpgradeCommand,
  version::Options as VersionCommand,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_version() -> String {
  let mut ver = VERSION.to_string();

  // Debug builds should have a "-dev",
  // and release builds should have a "+<commit hash>" suffix
  if IS_DEBUG {
    ver.push_str("-dev");
  } else if let Ok(output) = Command::new("git").current_dir(&*REPO_DIR).args(["-C", REPO_DIR.display().to_string().as_str(), "rev-parse", "--short", "HEAD"]).output() {
    if output.status.success() {
      if let Ok(commit_hash) = String::from_utf8(output.stdout) {
        ver.push_str(&format!("+{}", commit_hash.trim()));
      } else {
        // if for some reason the commit hash cannot be retrieved,
        // use "release" as a fallback
        ver.push_str("+release");
      }
    }
  }
  ver
}

#[derive(Debug, Subcommand, Command)]
#[non_exhaustive]
pub enum Commands {
  #[operation("Show this message")]
  Help(HelpCommand),
  #[operation("Show the current version")]
  Version(VersionCommand),
  #[operation("Upgrade to the latest version")]
  Upgrade(UpgradeCommand),
  #[operation("Run a command")]
  Run(RunCommand),
  #[operation("Show information about the system")]
  Info(InfoCommand),
  #[operation("Generate shell completions")]
  Completions(CompletionsCommand),
  #[operation("Show or modify environment variables")]
  Env(EnvCommand),
  #[operation("Send a desktop notification")]
  Notify(NotifyCommand),

  #[command(external_subcommand)]
  External(Vec<String>),
}

impl Operation for Commands {
  const NAME: &'static str = BINARY_NAME;

  fn main(&self) -> std::io::Result<()> {
    Ok(())
  }
}

pub fn execute_command<Command: Operation>(command: &Command) {
  match command.validate() {
    Ok(_) => {
      if let Err(e) = command.main() {
        CONSOLE.exit(format!("{e}"));
      }
      std::process::exit(0);
    },
    Err(e) => CONSOLE.exit(format!("{e}")),
  }
}

pub fn print_slogan() {
  CONSOLE.print(format!(
    "{desc} <brightblack>(v{version})</brightblack>\n",
    version = get_version(),
    desc = env!("CARGO_PKG_DESCRIPTION").replace(BINARY_NAME, &format!("<magenta>{BINARY_NAME}</magenta>"))
  ));
}

struct_gen! {
  #[command(name = BINARY_NAME, disable_help_flag = true, disable_help_subcommand = true, subcommand_required = false)]
  #[usage(Flags, Operand { name: "operation".to_string() })]
  pub struct Options use Parser, Command {
    #[command(subcommand)]
    let command: Option<Commands> = None;

    #[arg(short = 'H', long), help]
    let help: bool = false;

    #[arg(short = 'V', long), flag("Show the current version")]
    let version: bool = false;
  }

  impl Operation {
    const NAME: &'static str = BINARY_NAME;

    fn main(self: &Self) -> std::io::Result<()> {
      self.help.then(|| execute_command(&HelpCommand::default()));
      self.version.then(|| execute_command(&VersionCommand::default()));

      match self.command {
        Some(ref command) => {
          match command {
            Commands::Help(options) => execute_command(options),
            Commands::Version(options) => execute_command(options),
            Commands::Upgrade(options) => execute_command(options),
            Commands::Run(options) => execute_command(options),
            Commands::Info(options) => execute_command(options),
            Commands::Completions(options) => execute_command(options),
            Commands::Notify(options) => execute_command(options),
            Commands::Env(options) => execute_command(options),

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

          Ok(())
        }
        _ => {
          print_slogan();
          CONSOLE.print(format!("Use <magenta>{} help</magenta> for additional information", BINARY_NAME.to_lowercase()));

          std::process::exit(0);
        }
      }
    }
  }
}

pub type ClapError = clap::error::Error<clap::error::DefaultFormatter>;
pub fn handle_error(e: ClapError) -> ! {
  match e.kind() {
    ErrorKind::UnknownArgument => {
      if let Some(argument) = e.get(ContextKind::InvalidArg) {
        let arg = argument.to_string();
        CONSOLE.exit(format!(
          "The {} {:?} is not recognized.",
          {
            if arg.starts_with("-") {
              "argument"
            } else {
              "operation"
            }
          },
          arg
        ));
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

        let arg_name = match arg.split_whitespace().next() {
          Some(argument_name) => argument_name.to_string(),
          _ => arg.clone(),
        };

        if arg.starts_with("-") {
          if let Some(invalid_value) = e.get(ContextKind::InvalidValue) {
            let value = invalid_value.to_string();
            if value.is_empty() {
              if arg.ne(&arg_name) {
                CONSOLE.exit(format!("The argument <brightmagenta>{arg_name}</brightmagenta> requires a value."));
              } else {
                std::process::exit(1);
              }
            }

            CONSOLE.error(format!("The value is not valid for the argument <brightmagenta>{arg_name}</brightmagenta>."));
            if let Some(valid_values) = e.get(ContextKind::ValidValue) {
              match valid_values {
                ContextValue::Strings(values) => {
                  CONSOLE.exit(format!("Valid values are: {}", values.join(", ")));
                },
                _ => std::process::exit(1),
              }
            } else {
              std::process::exit(1);
            }
          }
        }
      }
    },
    _ => CONSOLE.exit(format!("{e}")),
  }

  CONSOLE.exit("Unkown operation behavior");
}

fn run(argv: Vec<String>) {
  match Options::try_parse_from(argv.iter()) {
    Ok(cli) => execute_command::<Options>(&cli),
    Err(e) => handle_error(e),
  }
}

fn main() {
  (!USER_CONFIG_DIR.exists()).then(|| std::fs::create_dir_all(&*USER_CONFIG_DIR).unwrap());
  run(std::env::args().collect::<Vec<String>>());
}
