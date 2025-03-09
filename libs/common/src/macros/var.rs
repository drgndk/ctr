/// Retrieves the value of the environment variable named `$name` as an
/// `Option<String>`.
///
/// # Examples
///
/// ```
/// let value = option_var!(MY_ENV_VAR);
/// ```
///
/// If the environment variable is found, it returns `Some(value)`.
/// If the environment variable is not found, it returns `None`.
#[macro_export]
macro_rules! option_var {
  ($name:ident) => {
    std::env::var(stringify!($name)).ok()
  };
}

/// Retrieves the value of the environment variable named `$name`.
///
/// # Examples
///
/// ```
/// let value = var!(MY_ENV_VAR);
/// ```
///
/// If the environment variable is not found, it will cause a panic with a
/// message indicating the missing variable.
///
/// ```
/// let value = var!(MY_ENV_VAR, "default_value");
/// ```
///
/// If the environment variable is not found, it will return the provided
/// fallback value.
#[macro_export]
macro_rules! var {
  ($name:ident) => {
   option_var!($name).unwrap_or_else(|| CONSOLE.panic!("Environment variable {} not found", $name))
  };

  ($name:ident, $fallback:expr) => {
    option_var!($name).unwrap_or($fallback)
  };
}

#[macro_export]
macro_rules! lazy_var {
  ($pub:vis $name:ident<$type:ty> $fn_block:block) => {
    $pub static $name: LazyLock<$type> = LazyLock::new(|| $fn_block);
  };
}
