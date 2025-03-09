use clap::Args;
use std_v2::{command::Operation, struct_gen};

struct_gen! {
  pub struct Options use Args, std_v2::derive::Command {
    #[arg(short = 'H', long), help]
    let help: bool = false;
  }

  impl Operation {
    const NAME: &'static str = "help";

    fn main(self: &Self) -> std::io::Result<()> {
      (self.help).then(|| Self::usage(0));
      crate::Options::usage(0);
      Ok(())
    }
  }
}
