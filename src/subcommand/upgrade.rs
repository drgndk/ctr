use std::{
  path::Path,
  process::{Command, ExitStatus},
};

use clap::Args;
use common::{
  command::{Operation, types::ArgumentType},
  console::CONSOLE,
  env::consts::{BINARY_NAME, INSTALL_DIR, REPO_DIR},
  struct_gen,
};
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;

use super::check_conflicts;
use crate::VERSION;

#[derive(Deserialize)]
struct CargoPackage {
  version: String,
}

#[derive(Deserialize)]
struct CargoToml {
  package: CargoPackage,
}

struct_gen! {
  pub struct Options use Args, std_v2::derive::Command {
    #[arg(short = 'H', long), help]
    pub let help: bool = false;

    #[arg(short = 'C', long), flag("Check for updates without updating")]
    pub let check: bool = false;

    #[arg(short = 'F', long), flag("Force the upgrade, even if the current version is the latest")]
    pub let force: bool = false;
  }

  impl Operation {
    const NAME: &'static str = "upgrade";

    fn usage(status: i32) {
      CONSOLE.print_usage::<Self>(vec![ArgumentType::Flags]);
      CONSOLE.print_operation_collection(Self::operations());
      std::process::exit(status);
    }

    fn main(self: &Self) -> std::io::Result<()> {
      (self.help).then(|| Self::usage(0));

      match self.fetch_versions() {
        Ok((latest, current)) => {
          if latest > current || self.force {
            if self.force {
              self.download_update();
            } else {
              CONSOLE.print(format!("<brightmagenta>A new version of <italic>{BINARY_NAME}</italic> is available: <bold><white>{latest}</white></bold></brightmagenta>"));
              if !self.check {
                println!();
                self.download_update();
              }
            }
          } else {
            CONSOLE.print("<brightmagenta>Nice!</brightmagenta> You are already using the latest version");

            // for script compatibility
            if self.check {
              std::process::exit(1);
            }
          }
        },
        Err(err) => CONSOLE.error(format!("Unable to fetch the latest version: {err}"))
      }

      Ok(())
    }

    fn validate(self: &Self) -> std::io::Result<()> {
      check_conflicts(vec![
        ("force", self.force),
        ("check", self.check)
      ]);

      Ok(())
    }
  }

  mod implementation {
    fn fetch_versions(self: &Self) -> Result<(semver::Version, semver::Version), Box<dyn std::error::Error>> {
      let url = "https://raw.githubusercontent.com/drgndk/ctr/main/Cargo.toml";
      let response = Client::new().get(url).send()
        .unwrap_or_else(|err| CONSOLE.exit(format!("Failed to fetch the remote Cargo.toml: {err}")));

      let cargo_toml_content = response.text()
        .unwrap_or_else(|err| CONSOLE.exit(format!("Failed to read the remote Cargo.toml: {err}")));

      let cargo_toml: CargoToml = std_v2::toml::parse::<CargoToml>(&cargo_toml_content)
        .unwrap_or_else(|err| CONSOLE.exit(format!("Failed to parse the remote Cargo.toml: {err}")));

      if let Ok(latest_version) = Version::parse(&cargo_toml.package.version) {
        let current_version = Version::parse(VERSION).unwrap_or_else(|_| CONSOLE.exit("Failed to parse the current version"));

        Ok((latest_version, current_version))
      } else {
        CONSOLE.exit("Failed to parse the latest version");
      }
    }

    fn download_update(self: &Self) -> () {
      let repo_path = &*REPO_DIR;

      let git_cmd = |args: Vec<&'static str>| -> ExitStatus {
        let mut cmd = Command::new("git");
        cmd.current_dir(repo_path).args(["-C", repo_path.display().to_string().as_str()]).args(&args);

        match cmd.status() {
          Ok(status) => {
            if !status.success() {
              CONSOLE.exit(format!("Failed to execute the git {} command: Status: {status}", args.join(" ")));
            }

            status
          },
          Err(err) => CONSOLE.exit(format!("Failed to execute the git {} command: {err}", args.join(" ")))
        }
      };

      if !repo_path.join(".git").exists() {
        git_cmd(vec!["remote", "add", "origin", "https://github.com/drgndk/ctr.git"]);
        git_cmd(vec!["branch", "-M", "main"]);
      }

      // Don't overwrite the current changes in debug builds
      git_cmd(vec!["reset", "--hard"]);
      git_cmd(vec!["pull"]);

      if let Err(err) = Command::new("cargo").args(["b", "-r", "--manifest-path", REPO_DIR.join("Cargo.toml").display().to_string().as_str()]).current_dir(repo_path).status() {
        if err.kind() == std::io::ErrorKind::NotFound {
          CONSOLE.exit("<bold>Whoops! It seems like you don't have cargo installed.</bold> How strange...");
        }

        // if there is another error, cargo will print it
        std::process::exit(1);
      } else {
        let old_binary_path = INSTALL_DIR.join(format!("bin/{BINARY_NAME}"));

        if old_binary_path.exists() {
          if let Err(err) = std::fs::remove_file(&old_binary_path) {
            CONSOLE.exit(format!("Failed to remove the old binary: {err}"));
          }
        }

        let new_binary_path = repo_path.join(format!("target/release/{BINARY_NAME}"));
        if let Err(err) = std::fs::copy(&new_binary_path, old_binary_path) {
          CONSOLE.exit(format!("Failed to copy the new binary: {err}"));
        }
      }

      let bin_path = &*INSTALL_DIR.join(format!("bin/{BINARY_NAME}"));
      if !Path::new(&bin_path).exists() {
        let new_binary_path = repo_path.join(format!("target/release/{BINARY_NAME}"));
        if std::fs::rename(&new_binary_path, bin_path).is_err() {
          println!();
          CONSOLE.warn(format!("<brightyellow><white><bold>{BINARY_NAME}</bold></white> was successfully built using cargo, but no symlink was created inside <white>{}</white></brightyellow>", bin_path.display()));
          CONSOLE.warn("<brightyellow>to do that, you can run the following command: </brightyellow>");
          CONSOLE.print(format!("<bold>sudo</bold> ln -s {new_binary_path:?} {bin_path:?}"));
        }
      }

      CONSOLE.print(format!("\n<brightmagenta><bold>Successfully</bold> updated <white>{BINARY_NAME}</white> to the latest version</brightmagenta>"));
    }
  }
}
