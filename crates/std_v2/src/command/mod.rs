use std::io;

pub trait Operation<O = ()>  {
  // Metadata for internal use
  const NAME: &'static str;
  const PARENT: Option<&'static str> = None;

  fn usage(status: i32) {
    std::process::exit(status);
  }

  fn main(&self) -> io::Result<()>;

  fn validate(&self) -> io::Result<()> {
    Ok(())
  }
}
