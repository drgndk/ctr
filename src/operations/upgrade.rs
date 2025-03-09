use std::{
  path::Path,
  process::{Command, Stdio},
};

use clap::{builder::PossibleValue, Args, ValueEnum};
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use std_v2::{
  command::Operation,
  console::CONSOLE,
  env::consts::{BINARY_NAME, INSTALL_DIR, REPO_DIR},
  struct_gen,
};

use super::check_conflicts;
use crate::{get_version, VERSION};

#[derive(Deserialize)]
struct CargoPackage {
  version: String,
  repository: String,
}

#[derive(Deserialize)]
struct CargoToml {
  package: CargoPackage,
}

#[derive(Deserialize)]
pub struct CommitResponse {
  commit: CommitInfo,
}

#[derive(Deserialize)]
pub struct CommitInfo {
  message: String,
}

static GITHUB_COMMIT_URL: &str = "https://api.github.com/repos/drgndk/ctr/commits";

#[derive(Debug, Clone, Copy)]
pub enum UpdateChannel {
  Stable,
  Unstable,
}

impl ValueEnum for UpdateChannel {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::Stable, Self::Unstable]
  }

  fn to_possible_value(&self) -> Option<PossibleValue> {
    Some(match self {
      Self::Stable => PossibleValue::new("stable"),
      Self::Unstable => PossibleValue::new("unstable"),
    })
  }
}

impl UpdateChannel {
  fn to_str(&self) -> &'static str {
    match self {
      Self::Stable => "stable",
      Self::Unstable => "unstable",
    }
  }
}

struct_gen! {
  pub struct Options use Args, std_v2::derive::Command {
    #[arg(short = 'H', long), help]
    let help: bool = false;

    #[arg(short = 'C', long), flag("Check for updates without updating")]
    let check: bool = false;

    // This is currently very aggressive. Might change the behavior in the future.
    #[arg(short = 'F', long), flag("Forces a upgrade, a hard reset of the local repository as well as a rebuild")]
    let force: bool = false;

    #[arg(short, long), flag("Change the channel which gets build", example = "stable")]
    let channel: Option<UpdateChannel> = Some(UpdateChannel::Stable);

    #[arg(short = 'V', long), flag("Print verbose")]
    let verbose: bool = false;
  }

  impl Operation {
    const NAME: &'static str = "upgrade";

    fn main(self: &Self) -> std::io::Result<()> {
      (self.help).then(|| Self::usage(0));

      match self.fetch_versions() {
        Ok((latest, current)) => {
          if latest > current || self.force {
            if self.force {
              self.download_update();
            } else {

              let message = self.update_message();

              CONSOLE.print(format!("<brightmagenta>A new version of <italic>{BINARY_NAME}</italic> is available: <bold><white>{latest}</white></bold></brightmagenta>"));
              println!();
              CONSOLE.print(message);

              println!();
              if !self.check {
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
    fn exec_git_cmd(cmd: &str, verbose: bool) -> Result<String, Box<dyn std::error::Error>> {
      let arguments = cmd.split_whitespace();
      let mut output = Command::new("git");

      output
        .current_dir(&*REPO_DIR)
        .args(["-C", REPO_DIR.display().to_string().as_str()])
        .args(arguments);

      if !verbose {
        output.stdin(Stdio::null());
        output.stdout(Stdio::null());
        output.stderr(Stdio::null());
      }

      let output = output.output().unwrap_or_else(|err| CONSOLE.exit(format!("Failed to execute `git {cmd}`: {err}")));

      if output.status.success() {
        if verbose {
          let output = String::from_utf8(output.stdout).unwrap_or_else(|err| CONSOLE.exit(format!("Failed to parse git {cmd} command output: {err}")));
          let output = output.trim_end().to_string();

          return Ok(output);
        }

        return Ok(String::new());
      }

      Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, String::from_utf8(output.stderr).unwrap_or_else(|err| CONSOLE.exit(format!("Failed to parse git command error: {err}"))))))
    }

    fn fetch(self: &Self, url: impl Into<String>) -> String {
      let url = url.into();
      let response = Client::new().get(&url).header("User-Agent", format!("drgndk@{BINARY_NAME}-{}", get_version())).send()
        .unwrap_or_else(|err| CONSOLE.exit(format!("Failed to fetch '{url}': {err}")));

      match response.text() {
        Ok(response) => response,
        Err(err) => CONSOLE.exit(format!("Failed to fetch '{url}': {err}")),
      }
    }

    fn update_message(self: &Self) -> String {
      let mut latest_message = "No commit message available".to_string();

      let response = self.fetch(GITHUB_COMMIT_URL);

      let commits: Vec<CommitResponse> = serde_json::from_str(&response).unwrap_or_else(|err| CONSOLE.exit(format!("Failed to parse GitHub API response: {err}")));
      latest_message = commits.first().map(|c| &c.commit.message).unwrap_or(&latest_message).clone();

      return latest_message.trim_end().to_string()
    }

    fn fetch_versions(self: &Self) -> Result<(semver::Version, semver::Version), Box<dyn std::error::Error>> {
      // Get the metadata from the remote repository
      let url = "https://drgndk.github.io/metadata.toml";

      let response = Client::new().get(url).send()
        .unwrap_or_else(|err| CONSOLE.exit(format!("Failed to fetch the remote Cargo.toml: {err}")));

      let cargo_toml_content = response.text()
        .unwrap_or_else(|err| CONSOLE.exit(format!("Failed to read the remote Cargo.toml: {err}")));

      let cargo_toml: CargoToml = std_v2::toml::parse::<CargoToml>(&cargo_toml_content)
        .unwrap_or_else(|err| CONSOLE.exit(format!("Failed to parse the remote Cargo.toml: {err}")));

      let mut repo_origin = String::new();
      if let Ok(repo_url) = Self::exec_git_cmd("remote get-url origin", true) {
        repo_origin = repo_url;
      };

      if !repo_origin.eq(&cargo_toml.package.repository) {
        if !repo_origin.is_empty() {
          CONSOLE.print(format!("<green>=></green> Found new repository origin: <bold>{}</bold>", cargo_toml.package.repository));
        }

        if let Err(err) = Self::exec_git_cmd(&format!("remote add origin {repo_origin}"), false){
          CONSOLE.exit(format!("Failed to set repository origin: {err}"));
        }

        if repo_origin.is_empty() {
          CONSOLE.print("<green>=></green> Added repository origin.".to_string());
        } else {
          CONSOLE.print("<green>=></green> Updated repository origin.".to_string());
        }
      }

      if let Ok(latest_version) = Version::parse(&cargo_toml.package.version) {
        let current_version = Version::parse(VERSION).unwrap_or_else(|_| CONSOLE.exit("Failed to parse the current version"));

        Ok((latest_version, current_version))
      } else {
        CONSOLE.exit("Failed to parse the latest version");
      }
    }

    fn download_update(self: &Self) -> () {
      let repo_path = &*REPO_DIR;

      let channel = match self.channel {
        Some(UpdateChannel::Unstable) => UpdateChannel::Unstable,
        _ => UpdateChannel::Stable
      };

      let target_path = match channel {
        UpdateChannel::Unstable => "debug",
        _ => "release"
      };

      if !repo_path.join(".git").exists() {
        CONSOLE.print("<green>=></green> Setting up the remote origin, for future updates");
        Self::exec_git_cmd("remote add origin https://github.com/drgndk/ctr.git", self.verbose);
        Self::exec_git_cmd("branch -M main", self.verbose);
      }

      if self.force {
        CONSOLE.print("<green>=></green> Resetting local repository [Forced]");
        Self::exec_git_cmd("reset --hard", self.verbose);
      }

      CONSOLE.print("<green>=></green> Pulling latest changes");
      Self::exec_git_cmd("pull", self.verbose);



     if self.force {
      CONSOLE.print("<green>=></green> Cleaning the project [Forced]");
      let mut cargo_args = vec!["clean"];

      if !self.verbose {
        cargo_args.push("-q");
      }

      if let Err(err) = Command::new("cargo").args(cargo_args).current_dir(repo_path).status() {
        CONSOLE.exit(format!("Failed to clean the project: {err}"));
      }
     }

      CONSOLE.print("<green>=></green> Building binary");
      let metadata_path = REPO_DIR.join("Cargo.toml").display().to_string();
      let mut cargo_args = vec!["b", "--manifest-path", metadata_path.as_str(), "-F", "stable"];

      if let UpdateChannel::Stable = channel {
        cargo_args.insert(1, "-r");
      }

      if !self.verbose {
        cargo_args.push("-q");
      }

      if let Err(err) = Command::new("cargo").args(cargo_args).current_dir(repo_path).status() {
        if err.kind() == std::io::ErrorKind::NotFound {
          CONSOLE.exit("<bold>Whoops! It seems like you don't have cargo installed.</bold> How strange...");
        }

        // if there is another error, cargo will print it
        std::process::exit(1);
      } else {
        let old_binary_path = INSTALL_DIR.join(format!("bin/{BINARY_NAME}"));

        if old_binary_path.exists() {
          if let Err(err) = std::fs::remove_file(&old_binary_path) {
            CONSOLE.exit(format!("Failed to remove the old binary: {}: {err}", old_binary_path.display()));
          }
        }

        let new_binary_path = repo_path.join(format!("target/{target_path}/{BINARY_NAME}"));
        if let Err(err) = std::fs::copy(&new_binary_path, old_binary_path) {
          CONSOLE.exit(format!("Failed to copy the new binary: {}: {err}", new_binary_path.display()));
        }
      }

      CONSOLE.print(format!("\n<brightmagenta><bold>Successfully</bold> updated <white>{BINARY_NAME}</white> to the latest version</brightmagenta>"));
    }
  }
}
