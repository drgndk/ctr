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
    $(/$(/)* $($doc_comments:tt)*)*
    $(#[ $enum_attribute:meta ])*
    $visibility:vis enum $enum_name:ident $(use $($derive_traits:ident),*)? {
      $(
        $(#[ $($variant_attribute:meta),* ])?
        $variant_name:ident $(
          {$(
            let $field_name:ident: $field_type:ty = $field_default:expr;
          )*}
        )?
      ),*$(,)?
    }

    $(
      impl $impl_trait:ident$(<$($impl_generics:ty),*>)? {
        $(
          type $associated_type_name:ident = $associated_type:ty;
        )*
        $(
          const $const_name:ident: $const_type:ty = $const_value:expr;
        )*
        $(
          fn $fn_name:ident$(<$($fn_generic_param:ident$(: $fn_generic_bound:ident)?),*>)?($($fn_arg_name:ident: $fn_arg_type:ty),*) $(-> $fn_return_type:ty)? $fn_body:block
        )*
      }
    )*

    $(
      mod $($_:ident)? {
        $(
          $mod_visibility:vis fn $mod_fn_name:ident$(<$($mod_fn_generic_param:ident$(: $mod_fn_generic_bound:ident)?),*>)?($($mod_fn_arg_name:ident: $mod_fn_arg_type:ty),*) $(-> $mod_fn_return_type:ty)? $mod_fn_body:block
        )*
      }
    )*
  ) => {
    $(/$(/)* $($doc_comments)*)*
    #[derive(Debug)]
    $(#[derive($($derive_traits),*)])?
    $(#[$enum_attribute])*
    #[allow(dead_code)]
    $visibility enum $enum_name {
      $(
        $($(#[$variant_attribute])*)?
        $variant_name $({
          $($field_name: $field_type),*
        })?
      ),*
    }

    impl $enum_name {
      #[allow(unused_variables)]
      #[allow(dead_code)]
      pub fn to_string(self) -> String {
        match self {
          $(
            Self::$variant_name $({ $($field_name),* })? => stringify!($variant_name),
          )*
        }.to_string()
      }

      #[allow(dead_code)]
      pub fn try_from(string: impl Into<String>) -> Option<Self> {
        let string = string.into();
        match string.to_lowercase() {
          $(
            s if s.eq(&stringify!($variant_name).to_lowercase()) => Some(Self::$variant_name $({ $($field_name: $field_default),* })?),
          )*
          _ => None,
        }
      }
    }

    impl From<&str> for $enum_name {
      fn from(string: &str) -> Self {
        if let Some(effect) = Self::try_from(string) {
          return effect;
        }

        use $crate::console::CONSOLE;
        CONSOLE.panic("Could not convert string to enum variant.")
      }
    }

    impl From<&String> for $enum_name {
      fn from(string: &String) -> Self {
        Self::from(string.as_str())
      }
    }

    impl From<String> for $enum_name {
      fn from(string: String) -> Self {
        Self::from(&string)
      }
    }

    $(
      impl $impl_trait$(<$($impl_generics),*>)? for $enum_name {
        $(
          type $associated_type_name = $associated_type;
        )*
        $(
          const $const_name: $const_type = $const_value;
        )*
        $(
          fn $fn_name$(<$($fn_generic_param$(: $fn_generic_bound)?),*>)?($($fn_arg_name: $fn_arg_type),*) $(-> $fn_return_type)? $fn_body
        )*
      }
    )*

    $(
      $(
        impl $enum_name {
          $mod_visibility fn $mod_fn_name$(<$($mod_fn_generic_param$(: $mod_fn_generic_bound)?),*>)?($($mod_fn_arg_name: $mod_fn_arg_type),*) $(-> $mod_fn_return_type)? $mod_fn_body
        }
      )*
    )*
  };
}
