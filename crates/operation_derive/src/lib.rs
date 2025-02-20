use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

mod field;
use field::{Field, FieldTypes};

#[proc_macro_derive(Command, attributes(help, flag, long_flag, variadic, operation, subcommand))]
pub fn operation_info_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let name = &input.ident;

  let mut essential = Vec::new();
  let mut operations = Vec::new();
  let mut flags = Vec::new();
  let mut variadics = Vec::new();

  let fields = match input.data {
    Data::Struct(data) => Field::get_fields(&data.fields, None),
    Data::Enum(data) => {
      let mut fields: Vec<Field> = vec![];
      for variant in data.variants.iter() {
        let variant_name = variant.ident.to_string();
        let field_name = variant_name.to_lowercase();
        let mut field = Field::new();
        field.name = field_name;
        field.from_attributes(&variant.attrs);
        fields.push(field);
      }

      fields
    },
    Data::Union(data) => {
      let mut fields: Vec<Field> = vec![];
      for field in data.fields.named.iter() {
        let field_name = field.ident.as_ref().unwrap().to_string().to_lowercase();
        let mut _field = Field::new();
        _field.name = field_name;
        _field.from_attributes(&field.attrs);
        fields.push(_field);
      }
      fields
    }
  };

  for field in fields {
    let name = field.name();
    let about = field.about();

    match field.field_type() {
      FieldTypes::Help => essential.push(quote! { common::command::Command::help_flag() }),
      FieldTypes::Variadic => variadics.push(quote! { common::command::Command::variadic(#name, #about) }),
      FieldTypes::Flag => {
        flags.push(
          if let Some(example) = field.example() {
            let example = example.as_str();
            quote! { common::command::Command::option(#name, #example, #about) }
          } else {
            quote! { common::command::Command::flag(#name, #about) }
          }
        );
      },
      FieldTypes::LongFlag => {
        flags.push(
          if let Some(example) = field.example() {
            let example = example.as_str();
            quote! { common::command::Command::long_option(#name, #example, #about) }
          } else {
            quote! { common::command::Command::long_flag(#name, #about) }
          }
        );
      },
      FieldTypes::Operation => {
        operations.push(
          if let Some(example) = field.example() {
            let example = example.as_str();
            quote! { common::command::Command::operation(#name, #example, #about) }
          } else {
            quote! { common::command::Command::command(#name, #about) }
          }
        );
      },
      FieldTypes::Subcommand => operations.push(quote! { common::command::Command::command(#name, #about) }),
      _ => {}
    }
  }

  TokenStream::from(
    quote! {
      impl #name {
        pub fn operations() -> Vec<Vec<common::command::Command>> {
          vec![
            vec![#(#essential),*],
            vec![#(#operations),*],
            vec![#(#flags),*],
            vec![#(#variadics),*]
          ]
        }
      }
    }
  )
}
