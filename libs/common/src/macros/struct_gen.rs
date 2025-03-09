/// Generates a struct with associated functions and traits.
///
/// # Example
/// ```rs
/// struct_gen! {
///   pub struct MyStruct<T: Default = i32> use Clone {
///     pub let name: String = String::from("default name");
///     let age: u32 = 0;
///   }
///
///   impl MyTrait {
///     type Output = String;
///
///     const DEFAULT_AGE: u32 = 18;
///
///     fn greet(self: &Self) -> String {
///       let Self { name, age } = self;
///       format!("Hello, my name is {name} and I am {age} years old.")
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
///   let instance = MyStruct::new();
///   println!("{}", instance.greet());
///   println!("{}", MyStruct::helper());
/// }
/// ```
#[macro_export]
macro_rules! struct_gen {
  (
    $(/$(/)* $($doc_comments:tt)*)*
    $(#[ $struct_attribute:meta ])*
    $struct_visibility:vis$(($struct_crate_visibility:vis))? struct $struct_name:ident$(< $($generic_param:ident$(: $generic_constraint:ident)?$(= $generic_default:tt)?),* >)? $(use $($($derive_trait:ident)::*),*)? {
      $(
        $(#[$($field_attribute:meta),*])*
        $field_visibility:vis$(($field_super_visibility:vis))? let $(&$field_mutability:ident)? $field_name:ident: $field_type:ty = $field_default:expr;
      )*
    }

    $(
      $(#[ $impl_attribute:meta ])*
      impl $($impl_path:ident)::*$(<$($impl_generic:ty),*>)? {
        $(type $associated_type_name:ident = $associated_type:ty;)*
        $(const $const_name:ident: $const_type:ty = $const_value:expr;)*
        $(
          $(#[ $fn_attribute:meta ])*
          fn $fn_name:ident$(<$($fn_generic_param:ident$(: $fn_generic_constraint:ident)?),*>)?($($fn_arg_name:ident: $fn_arg_type:ty),*) $(-> $fn_return_type:ty)? $fn_body:block
        )*
      }
    )*

    $(
      $(#[ $mod_attribute:meta ])*
      mod $($_:ident)? {
        $(
          $(/$(/)* $($mod_doc_comments:tt)*)*
          $(#[ $mod_fn_attribute:meta ])*
          $mod_fn_visibility:vis fn $mod_fn_name:ident$(<$($mod_fn_generic_param:ident$(: $mod_fn_generic_constraint:ident)?),*>)?($($mod_fn_arg_name:ident: $mod_fn_arg_type:ty),*) $(-> $mod_fn_return_type:ty)? $mod_fn_body:block
        )*
      }
    )*
  ) => {
    $(/$(/)* $($doc_comments)*)*
    #[derive(Debug)]
    $(#[derive($($($derive_trait)::*),*)])?
    $(#[$struct_attribute])*
    $struct_visibility$(($struct_crate_visibility))?  struct $struct_name$(<$($generic_param$(: $generic_constraint)?$(= $generic_default)?),*>)? {
      $(
        $($(#[$field_attribute])*)?
        $field_visibility$(($field_super_visibility))? $field_name: $field_type
      ),*
    }

    impl Default for $struct_name {
      fn default() -> Self {
        Self {
          $($field_name: $field_default),*
        }
      }
    }

    impl$(<$($generic_param$(: $generic_constraint)?),*>)? $struct_name$(<$($generic_param),*>)? {
      #[allow(dead_code)]
      pub fn new($($field_name: $field_type),*) -> Self {
        Self {
          $( $field_name ),*
        }
      }

      $(
        #[allow(dead_code)]
        pub fn $field_name(&self) -> &$field_type {
          &self.$field_name
        }

        $(
          paste::paste! {
            #[allow(dead_code)]
            $field_visibility fn [<$field_name _mut>](&$field_mutability self) -> &mut $field_type {
              &mut self.$field_name
            }
          }
        )?
      )*

      $(
        $(
          $(/$(/)* $($mod_doc_comments)*)*
          $(#[ $mod_fn_attribute ])*
          #[allow(dead_code)]
          $mod_fn_visibility fn $mod_fn_name$(<$($mod_fn_generic_param$(: $mod_fn_generic_constraint)?),*>)?($($mod_fn_arg_name: $mod_fn_arg_type),*) $(-> $mod_fn_return_type)? $mod_fn_body
        )*
      )*
    }
    $(
      $(#[ $impl_attribute ])*
      impl $($impl_path)::*$(<$($impl_generic),*>)? for $struct_name {
        $(
          type $associated_type_name = $associated_type;
        )*
        $(
          const $const_name: $const_type = $const_value;
        )*
        $(
          $(#[ $fn_attribute ])*
          fn $fn_name$(<$($fn_generic_param$(: $fn_generic_constraint)?),*>)?($($fn_arg_name: $fn_arg_type),*) $(-> $fn_return_type)? $fn_body
        )*
      }
    )*
  };
}
