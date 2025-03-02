use std::{fs, path::Path};

use serde::{
  Serialize,
  de::{DeserializeOwned, Error},
};

pub use toml::*;
pub fn parse<V: DeserializeOwned>(contents: impl Into<String>) -> Result<V, de::Error> {
  toml::from_str(&contents.into())
}

pub fn parse_file<V: DeserializeOwned>(path: impl AsRef<Path>) -> Result<V, de::Error> {
  match fs::read_to_string(path) {
    Ok(contents) => parse(&contents),
    Err(e) => Err(de::Error::custom(format!("{e}"))),
  }
}

pub fn stringify<V: Serialize>(value: &V) -> String {
  toml::to_string(value).unwrap()
}
