/// Generates an enum with associated functions and traits.
///
/// # Example
/// ```rs
/// enum_gen! {
///   pub enum MyEnum use Clone {
///     VariantOne {
///       let name: String = String::from("default name");
///       let age: u32 = 0;
///     },
///     VariantTwo,
///   }
///
///   impl MyTrait {
///     type Output = String;
///
///     const DEFAULT_AGE: u32 = 18;
///
///     fn describe(self: &Self) -> String {
///       match self {
///         MyEnum::VariantOne { name, age } => format!("VariantOne: name = {name}, age = {age}"),
///         MyEnum::VariantTwo => String::from("VariantTwo"),
///       }
///     }
///   }
///
///   mod utils {
///     pub fn helper() -> String {
///       String::from("This is a helper function.")
///     }
///   }
/// }
///
/// fn main() {
///   let instance = MyEnum::from("VariantOne");
///   println!("{}", instance.describe());
///   println!("{}", MyEnum::helper());
/// }
/// ```
#[macro_export]
macro_rules! enum_gen {
  (
    $(#[ $impl_attribute:meta ])*
    $pub_struct:vis enum $struct:ident $(use $($structs:ident),*)? {
      $(
        $(#[ $($named_attribute:meta),* ])?
        $named_name:ident $(
          {$(
            let $field_name:ident: $field_type:ty = $field_default:expr;
          )*}
        )?
      ),*$(,)?
    }

    $(
      impl $mod_ident:ident$(<$($fn_type:ty),*>)? {
        $(
          type $type_name:ident = $type_type:ty;
        )*
        $(
          const $const_name:ident: $const_type:ty = $const_value:expr;
        )*
        $(
          fn $fn_name:ident$(<$($fn_generic:ident$(: $fn_generic_type:ty)?),*>)?($($fn_argument:ident: $arg_type:ty),*) $(-> $fn_return:ty)? $fn_block:block
        )*
      }
    )*

    $(
      mod $($_:ident)? {
        $(
          $mod_pub:vis fn $mod_fn_name:ident$(<$($mod_fn_generic:ident$(: $mod_fn_generic_type:ty)?),*>)?($($mod_fn_argument:ident: $mod_arg_type:ty),*) $(-> $mod_fn_return:ty)? $mod_fn_block:block
        )*
      }
    )*
  ) => {
    #[derive(Debug)]
    $(#[derive($($structs),*)])?
    $(#[$impl_attribute])*
    #[allow(dead_code)]
    $pub_struct enum $struct {
      $(
        $($(#[$named_attribute])*)?
        $named_name $({
          $($field_name: $field_type),*
        })?
      ),*
    }

    impl $struct {
      #[allow(unused_variables)]
      #[allow(dead_code)]
      pub fn to_string(self) -> String {
        match self {
          $(
            Self::$named_name $({ $($field_name),* })? => stringify!($named_name),
          )*
        }.to_string()
      }

      #[allow(dead_code)]
      pub fn try_from(string: impl Into<String>) -> Option<Self> {
        let string = string.into();
        match string.to_lowercase() {
          $(
            s if s.eq(&stringify!($named_name).to_lowercase()) => Some(Self::$named_name $({ $($field_name: $field_default),* })?),
          )*
          _ => None,
        }
      }
    }

    impl From<&str> for $struct {
      fn from(string: &str) -> Self {
        if let Some(effect) = Self::try_from(string) {
          return effect;
        }

        use $crate::console::CONSOLE;
        CONSOLE.panic("Could not convert string to enum variant.")
      }
    }

    impl From<&String> for $struct {
      fn from(string: &String) -> Self {
        Self::from(string.as_str())
      }
    }

    impl From<String> for $struct {
      fn from(string: String) -> Self {
        Self::from(&string)
      }
    }


    $(
      impl $mod_ident$(<$($fn_type),*>)? for $struct {
        $(
          type $type_name = $type_type;
        )*
        $(
          const $const_name: $const_type = $const_value;
        )*
        $(
          fn $fn_name$(<$($fn_generic$(: $fn_generic_type)?),*>)?($($fn_argument: $arg_type),*) $(-> $fn_return)? $fn_block
        )*
      }
    )*

    $(
      $(
        impl $struct {
          $mod_pub fn $mod_fn_name$(<$($mod_fn_generic$(: $mod_fn_generic_type)?),*>)?($($mod_fn_argument: $mod_arg_type),*) $(-> $mod_fn_return)? $mod_fn_block
        }
      )*
    )*
  };
}
