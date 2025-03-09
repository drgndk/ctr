use clap::Args;
use std_v2::{
  command::Operation,
  console::CONSOLE,
  struct_gen,
};

struct_gen! {
  pub struct Options use Args, std_v2::derive::Command {
    #[arg(short = 'H', long), help]
    let help: bool = false;

    #[arg(short = 'A', long), flag("Show all variables")]
    let all: bool = false;
  }

  impl Operation {
    const NAME: &'static str = "env";

    fn main(self: &Self) -> std::io::Result<()> {
      (self.help).then(|| Self::usage(0));

      let mut vars = std::env::vars().collect::<Vec<_>>();

      vars.sort_by(|a, b| a.0.cmp(&b.0));

      for (key, value) in vars {
        if key.starts_with('_') && !self.all {
          continue;
        }

        CONSOLE.print(format!("{}<black>=</black>{value}", key))
      }

      Ok(())
    }
  }
}
