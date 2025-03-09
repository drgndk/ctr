use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

mod field;
use field::{find_commands_enum, from_attributes, get_fields};

#[proc_macro_derive(Command, attributes(help, flag, longflag, variadic, operation, usage))]
pub fn operation_info_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;
  let cmd_enum_operations = if let Some(cmd_enum) = find_commands_enum(&input) {
    quote! { base.extend(#cmd_enum::operations()); }
  } else {
    quote! {}
  };

  let usage_attributes = input
    .attrs
    .iter()
    .filter_map(|attr| {
      if attr.path.is_ident("usage") {
        Some(attr.parse_args::<TokenStream2>().unwrap())
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  let usage_vector = if !usage_attributes.is_empty() {
    quote! { vec![#(#usage_attributes),*] }
  } else {
    quote! { vec![ Flags ] }
  };

  let fields = {
    let mut fields = match input.data {
      Data::Struct(data) => get_fields(&data.fields),
      Data::Enum(ref data) => data.variants.iter().filter_map(|variant| from_attributes(&variant.attrs, &variant.ident)).collect::<Vec<TokenStream2>>(),
      Data::Union(_) => common::console::CONSOLE.panic("Unit struct not supported"),
    };

    if let Some(field) = from_attributes(&input.attrs, &name) {
      fields.push(field);
    }
    fields
  };

  TokenStream::from(quote! {
    impl #name {

      fn usage(status: i32) {
        use std_v2::{ command::types::ArgumentType::*, console::CONSOLE };
        CONSOLE.print_usage::<Self>(#usage_vector);
        CONSOLE.print_operation_collection(Self::operations());
        std::process::exit(status);
      }

      pub fn operations() -> Vec<Vec<std_v2::command::Command>> {
        let mut base = vec![
          vec![#(#fields),*],
        ];

        #cmd_enum_operations

        base
      }
    }
  })
}
