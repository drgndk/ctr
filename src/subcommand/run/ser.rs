use std::{collections::HashMap, sync::LazyLock};

use common::struct_gen;
use serde::Deserialize;
use std_v2::{toml::Value, HOME};

type EnvironmentMap = HashMap<String, Value>;

static USER_STR: LazyLock<String> = LazyLock::new(|| HOME.to_string_lossy().to_string());
static SHELL: LazyLock<String> = LazyLock::new(|| std::env::var("SHELL").unwrap_or("bash".to_owned()));

struct_gen! {
  pub struct LaunchConfigRunAs use Deserialize, Clone {
    pub let sudo: Option<bool> = Some(false);
    pub let user: Option<String> = None;
    pub let group: Option<String> = None;
  }
}

struct_gen! {
  pub struct LaunchConfigGeneral use Deserialize {
    pub let preserve_env: Option<bool> = Some(true);
    pub let deamonize: Option<bool> = Some(false);
    pub let working_dir: Option<String> = Some(USER_STR.to_owned());
    pub let command: Option<String> = None;
    pub let shell: Option<String> = None;
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
        ($parent:ident { $($field:ident),*$(,)? }) => {
          $(
            if other.$parent.$field.is_some() {
              self.$parent.$field = other.$parent.$field;
            }
          )*
        };
        (Option<$parent:ident> { $($field:ident),*$(,)? }) => {
          $(
            if let Some(ref mut self_parent) = self.$parent {
              if let Some(other_field) = other.$parent.as_ref().and_then(|p| p.$field.clone()) {
                self_parent.$field = Some(other_field);
              }
            } else {
              self.$parent = other.$parent.clone();
            }
          )*
        };
      }

      merge!(general { preserve_env, deamonize, working_dir, command, shell });
      merge!(Option<run_as> { user, group, sudo });

      if let Some(env) = other.environment {
        self.environment.get_or_insert_with(EnvironmentMap::new).extend(env);
      }
    }
  }
}

struct_gen! {
  pub struct LaunchOptions {
    pub let preserve_env: bool = true;
    pub let environment: EnvironmentMap = EnvironmentMap::new();
    pub let current_dir: String = USER_STR.to_owned();
    pub let daemonize: bool = false;
    pub let command: String = String::new();
    pub let run_as: Option<LaunchConfigRunAs> = None;
    pub let shell: String = SHELL.to_owned();
  }

  impl From<LaunchConfig> {
    fn from(config: LaunchConfig) -> Self {
      Self {
        run_as: config.run_as,
        preserve_env: config.general.preserve_env.unwrap_or(true),
        environment: config.environment.unwrap_or_default(),
        current_dir: config.general.working_dir.unwrap_or(USER_STR.to_owned()),
        command: config.general.command.unwrap_or_default(),
        daemonize: config.general.deamonize.unwrap_or(false),
        shell: config.general.shell.unwrap_or(SHELL.to_owned()),
      }
    }
  }
}
