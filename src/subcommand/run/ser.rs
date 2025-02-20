use std::{collections::HashMap, sync::LazyLock};

use common::struct_gen;
use serde::Deserialize;
use std_v2::{toml::Value, HOME};

type EnvironmentMap = HashMap<String, Value>;

static USER_STR: LazyLock<String> = LazyLock::new(|| HOME.to_string_lossy().to_string());

struct_gen! {
  pub struct LaunchConfigRunAs use Deserialize {
    pub let user: Option<String> = None;
    pub let group: Option<String> = None;
  }
}

struct_gen! {
  pub struct LaunchConfigGeneral use Deserialize {
    pub let preserve_env: Option<bool> = Some(true);
    pub let escalate_privileges: Option<bool> = Some(false);
    pub let deamonize: Option<bool> = Some(false);
    pub let working_dir: Option<String> = Some(USER_STR.to_owned());
    pub let command: Option<String> = None;
  }
}

struct_gen! {
  pub struct LaunchConfig use Deserialize {
    pub let general: LaunchConfigGeneral = LaunchConfigGeneral::new();
    pub let run_as: Option<LaunchConfigRunAs> = None;
    pub let environment: Option<EnvironmentMap> = Some(EnvironmentMap::new());
  }

  mod implementation {
    pub fn merge(self: &mut Self, other: Self) {
      macro_rules! merge {
        ($field:ident) => {
          if other.general.$field.is_some() {
            self.general.$field = other.general.$field;
          }
        };
      }

      merge!(preserve_env);
      merge!(escalate_privileges);
      merge!(deamonize);
      merge!(working_dir);
      merge!(command);

      if let Some(env) = other.environment {
        self.environment.get_or_insert_with(EnvironmentMap::new).extend(env);
      }
    }
  }
}


struct_gen! {
  pub struct LaunchOptions {
    pub let preserve_env: bool = true;
    pub let escalate_privileges: bool = false;
    pub let environment: EnvironmentMap = EnvironmentMap::new();
    pub let current_dir: String = USER_STR.to_owned();
    pub let daemonize: bool = false;
    pub let command: String = String::new();
  }

  impl From<LaunchConfig> {
    fn from(config: LaunchConfig) -> Self {
      Self {
        preserve_env: config.general.preserve_env.unwrap_or(true),
        escalate_privileges: config.general.escalate_privileges.unwrap_or(false),
        environment: config.environment.unwrap_or_default(),
        current_dir: config.general.working_dir.unwrap_or(USER_STR.to_owned()),
        command: config.general.command.unwrap_or_default(),
        daemonize: config.general.deamonize.unwrap_or(false),
      }
    }
  }
}
