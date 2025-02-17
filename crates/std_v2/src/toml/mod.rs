use serde::{de::{DeserializeOwned, Error}, Serialize};
use std::{fs, path::Path};

pub use toml::*;

pub fn parse<V: DeserializeOwned>(contents: impl Into<String>) -> Result<V, de::Error> {
  match toml::from_str(&contents.into()) {
    Ok(parsed) => Ok(parsed),
    Err(e) => Err(e)
  }
}

pub fn parse_file<V: DeserializeOwned>(path: impl AsRef<Path>) -> Result<V, de::Error> {
  match fs::read_to_string(path) {
    Ok(contents) => parse(&contents),
    Err(e) => Err(de::Error::custom(&format!("{e}")))
  }
}

pub fn stringify<V: Serialize>(value: &V) -> String {
  toml::to_string(value).unwrap()
}
