use clap::Args;
use common::{command::{types::ArgumentType, Operation}, console::CONSOLE, struct_gen};

struct_gen! {
  pub struct Options use Args, std_v2::derive::Command {
    #[arg(short = 'H', long), help]
    pub let help: bool = false;
  }

  impl Operation {
    const NAME: &'static str = "version";

    fn usage(status: i32) {
      CONSOLE.print_usage::<Self>(vec![ArgumentType::Flags]);
      CONSOLE.print_operation_collection(Self::operations());
      std::process::exit(status);
    }

    fn main(self: &Self) -> std::io::Result<()> {
      (self.help).then(|| Self::usage(0));
      CONSOLE.print(crate::get_version());
      Ok(())
    }
  }
}
