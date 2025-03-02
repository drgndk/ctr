use std::{os::unix::process::CommandExt, path::Path, process::Command};

use clap::Args;
use std_v2::{
  command::{Operation, types::ArgumentType},
  console::CONSOLE,
  env::consts::{BINARY_NAME, CONFIG_DIR},
  struct_gen,
};
mod ser;
use ser::*;
use uzers::{get_group_by_name, get_user_by_name};

struct_gen! {
  pub struct Options use Args, std_v2::derive::Command {
    #[arg(short = 'H', long), help]
    pub let help: bool = false;

    #[arg(short = 'S', long), flag("Do not print any command output")]
    pub let silent: bool = false;

    #[arg(short = 'D', long), flag("Executes the given command in the background")]
    pub let daemonize: bool = false;

    #[arg(short = 'I', long), flag(name = "ignore-config", "Ignore the config file")]
    pub let ignore_config: bool = false;

    #[variadic(name = "args", about = "Arguments passed to the binary")]
    let args: Vec<String> = Vec::new();
  }

  impl Operation {
    const NAME: &'static str = "run";

    fn usage(status: i32) {
      CONSOLE.print("<brightblue>binary</brightblue> can also be the name of the config file in <bold>~/.config/ctr/binaries</bold>\n");
      CONSOLE.print_usage::<Self>(vec![ArgumentType::Flags, ArgumentType::Operand { name: "binary".to_owned()}, ArgumentType::Variadic { name: "args".to_owned() }]);
      CONSOLE.print_operation_collection(Self::operations());
      std::process::exit(status);
    }

    fn main(self: &Self) -> std::io::Result<()> {
      (self.help).then(|| Self::usage(0));

      if self.args.is_empty() {
        CONSOLE.exit(format!("No binary specified. Use <magenta>{BINARY_NAME} run --help</magenta> for additional information"));
      }


      let config = self.get_configs();
      self.launch(&LaunchOptions::from(config))
    }
  }

  mod utils {
    fn get_group_id(self: &Self, group: &Option<String>) -> Option<u32> {
      if let Some(group) = group {
        if let Ok(gid) = group.parse::<u32>() {
          return Some(gid);
        }

        if let Some(group) = get_group_by_name(group.as_str()) {
          return Some(group.gid());
        }
      }

      None
    }

    fn get_user_id(self: &Self, user: &Option<String>) -> Option<u32> {
      if let Some(user) = user {
        if let Ok(uid) = user.parse::<u32>() {
          return Some(uid);
        }

        if let Some(user) = get_user_by_name(user.as_str()) {
          return Some(user.uid());
        }
      }

      None
    }
  }

  mod implementation {
    pub fn get_configs(self: &Self) -> LaunchConfig {
      let config_path = &*CONFIG_DIR.join(format!("binaries/{}.toml", self.args()[0]));
      let mut default_config = LaunchConfig::default();

      if self.ignore_config || !config_path.exists() {
        if !self.args.is_empty() && default_config.general.command.is_none() {
          default_config.general.command = Some(self.args.join(" "));
        }

        return default_config;
      }

      let config: LaunchConfig = std_v2::toml::parse_file(config_path).unwrap_or_else(|err| CONSOLE.exit(format!("{err}")));

      default_config.merge(config);
      default_config.general.working_dir = default_config.general.working_dir.map(|e| {
        match Path::new(e.as_str()).canonicalize() {
          Ok(path) => path.to_string_lossy().to_string(),
          _ => e
        }
      });

      default_config
    }

    pub fn launch(self: &Self, options: &LaunchOptions) -> std::io::Result<()>  {
      let args = {
        let mut args = vec![options.shell.to_owned(), "-c".to_owned()];

        if let Some(ref run_as) = options.run_as {
          if run_as.sudo.unwrap_or(false) {
            let sudo_cmd = {
              let mut sudo = vec!["sudo".to_owned()];

              if let Some(user) = run_as.user.as_ref() {
                sudo.extend(vec!["-u".to_owned(), user.to_owned()]);
              }

              if let Some(group) = run_as.group.as_ref() {
                sudo.extend(vec!["-g".to_owned(), group.to_owned()]);
              }

              sudo.push("--".to_owned());

              sudo
            };

            for (i, arg) in sudo_cmd.iter().enumerate() {
              args.insert(i, arg.to_owned());
            }
          }
        }

        args.push(
          if options.daemonize {
            format!("setsid {}", options.command)
          } else {
            options.command.to_owned()
          }
        );
        args
      };

      if let Some(binary) = args.first() {
        let mut command = Command::new(binary);

        if let Some(ref run_as) = options.run_as {
          if !run_as.sudo.unwrap_or(false) {
            if let Some(gid) = self.get_group_id(&run_as.group) {
              command.gid(gid);
            }

            if let Some(uid) = self.get_user_id(&run_as.user) {
              command.uid(uid);
            }
          }
        }

        command.args(&args[1..]);

        if self.silent {
          command.stdout(std::process::Stdio::null());
          command.stderr(std::process::Stdio::null());
        }

        match command.spawn() {
          Ok(_) if self.daemonize => std::process::exit(0),
          Ok(mut child) => {
            match child.wait() {
              Ok(status) => std::process::exit(status.code().unwrap_or(1)),
              Err(err) => CONSOLE.exit(format!("Failed to wait for `{}`: {err}", args.join(" ")))
            }
          },
          Err(err) => CONSOLE.exit(format!("Failed to run `{}`: {err}", args.join(" ")))
        }
      } else {
        CONSOLE.exit("No binary specified")
      }
    }
  }
}
