use clap::Args;
use common::{command::Operation, console::CONSOLE, struct_gen};
use std_v2::derive::Command;

use crate::execute_command;

struct_gen! {
  pub struct Options use Args, Command {
    #[arg(short, long)]
    #[help_flag]
    pub let help: bool = false;
  }

  impl Operation {
    const NAME: &'static str = "help";

    fn usage(status: i32) {
      CONSOLE.print_usage::<Self>(vec![]);
      CONSOLE.print_operation_collection(Self::operations());
      std::process::exit(status);
    }

    fn main(self: &Self) -> std::io::Result<()> {
      (self.help).then(|| Self::usage(0));

      execute_command(&crate::Options {
        command: None,
        help: true,
        version: false
      });

      Ok(())
    }
  }
}
