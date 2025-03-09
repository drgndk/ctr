use std::path::Path;

use clap::{Args, CommandFactory, Parser};
use std_v2::{
  command::{types::ArgumentType, Operation},
  console::CONSOLE,
  derive::Command,
  env::{consts::SHELL, Shell},
  struct_gen,
};

use crate::Commands;

#[derive(Parser)]
#[command(disable_help_flag = true, disable_help_subcommand = true, subcommand_required = false)]
struct Completions {
  #[command(subcommand)]
  command: Commands,
}

struct_gen! {
  pub struct Options use Args, Command {
    #[arg(short = 'H', long), help]
    pub let help: bool = false;

    #[arg(short, long), flag("Directory where to install completions", example = "~/.config/fish/completions")]
    pub let dir: Option<String> = None;

    #[arg(short, long), flag("Change the shell to generate completions for", example = "bash")]
    pub let shell: Option<Shell> = None;
  }

  impl Operation {
    const NAME: &'static str = "completions";

    fn main(self: &Self) -> std::io::Result<()> {
      (self.help).then(|| Self::usage(0));
      let commands = &mut Completions::command();

      let shell = self.shell.unwrap_or(*SHELL);
      if let Some(ref path) = self.dir {
        match shell.generate_to(commands, path.clone()) {
          Ok(path) => CONSOLE.print(format!("<brightmagenta>Installed completions to {}</brightmagenta>", path.display())),
          Err(err) => CONSOLE.exit(format!("{err}"))
        }
      } else {
        shell.generate(commands, &mut std::io::stdout());
      }

      return Ok(());
    }

    fn validate(self: &Self) -> std::io::Result<()> {
      if let Some(ref file_path) = self.dir {
        let path = Path::new(file_path);
        if !path.exists() {
          CONSOLE.exit(format!("{file_path} does not exist"));
        }

        if !path.is_dir() {
          CONSOLE.exit(format!("{file_path} is not a directory"));
        }

        return Ok(());
      }

      Ok(())
    }
  }
}
