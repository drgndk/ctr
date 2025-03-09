use common::console::CONSOLE;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, Lit, Meta, NestedMeta, PathArguments, Type};

pub fn from_attributes(attributes: &[syn::Attribute], ident: &Ident) -> Option<TokenStream> {
  for attr in attributes.iter() {
    let mut name = ident.to_string();
    let mut example: Option<String> = None;
    let mut about = String::new();

    let field = attr.path.get_ident();

    let meta = attr.parse_meta();
    match meta {
      Ok(Meta::List(meta_list)) => {
        for nested_meta in meta_list.nested.iter() {
          match nested_meta {
            NestedMeta::Meta(meta) => {
              if let Meta::NameValue(name_value) = meta {
                if let Some(ident) = name_value.path.get_ident().map(|i| i.to_string()) {
                  match ident.as_str() {
                    ident_name if ["name", "example", "about"].contains(&ident_name) => {
                      if let Lit::Str(lit_str) = &name_value.lit {
                        let value = lit_str.value();
                        match ident_name {
                          "name" => name = value,
                          "example" => example = Some(value),
                          _ => about = value,
                        }
                      }
                    },
                    _ => {},
                  }
                }
              }
            },
            NestedMeta::Lit(lit) => {
              if let Lit::Str(lit_str) = lit {
                about = lit_str.value();
              }
            },
          }
        }
      },
      Ok(Meta::Path(path)) if path.is_ident("help") => {
        name = "help".to_string();
      },
      _ => continue,
    }

    match field {
      Some(field) => {
        let field_name = field.to_string();
        if name.is_empty() {
          name = field_name.clone();
        }

        name = name.replace('_', "-");

        match field_name.to_lowercase().as_str() {
          "help" => return Some(quote! { std_v2::command::Command::help_flag() }),
          "flag" => {
            let short: char = name.chars().next().unwrap_or(' ').to_ascii_lowercase();

            return match example {
              Some(example) => Some(quote! { std_v2::command::Command::option(#short, #name, #example, #about) }),
              None => Some(quote! { std_v2::command::Command::flag(#short.to_ascii_uppercase(), #name, #about) }),
            }
          },
          "longflag" => {
            return match example {
              Some(example) => Some(quote! { std_v2::command::Command::long_option(#name, #example, #about) }),
              None => Some(quote! { std_v2::command::Command::long_flag(#name, #about) }),
            }
          },
          "variadic" => return Some(quote! { std_v2::command::Command::variadic(#name, #about) }),
          "operation" => {
            return match example {
              Some(example) => Some(quote! { std_v2::command::Command::operation(#name, #example, #about) }),
              None => Some(quote! { std_v2::command::Command::subcommand(#name, #about) }),
            }
          },
          _ => continue,
        }
      },
      None => continue,
    }
  }

  None
}

pub fn get_fields(fields: &Fields) -> Vec<TokenStream> {
  let mut field_vector = vec![];

  match fields {
    // Structs
    Fields::Named(fields) => {
      for named_field in fields.named.iter() {
        if named_field.attrs.is_empty() {
          continue;
        }

        if let Some(ref ident) = named_field.ident {
          if let Some(field) = from_attributes(&named_field.attrs, &ident) {
            field_vector.push(field);
          }
        }
      }
    },
    // Enums [Experimental]
    Fields::Unnamed(fields) => {
      for (_, unnamed_field) in fields.unnamed.iter().enumerate() {
        if unnamed_field.attrs.is_empty() {
          continue;
        }

        if let Some(ref ident) = unnamed_field.ident {
          if let Some(field) = from_attributes(&unnamed_field.attrs, &ident) {
            field_vector.push(field);
          }
        }
      }
    },
    Fields::Unit => CONSOLE.panic("Unit struct not supported"),
  };

  field_vector
}

pub fn find_commands_enum(input: &DeriveInput) -> Option<Ident> {
  let data_struct = match input.data {
    Data::Struct(ref data_struct) => data_struct,
    _ => return None,
  };

  for field in &data_struct.fields {
    let ident = field.ident.as_ref()?;

    // suboperations always contain `#[command(subcommand)]`
    // if this is missing, it's not a enum containing operations
    if ident != "command" {
      continue;
    }

    let group = match field.ty {
      Type::Group(ref group) => group,
      _ => continue,
    };

    if let Type::Path(ref type_path) = *group.elem {
      let segment = type_path.path.segments.last()?;
      let args = match &segment.arguments {
        PathArguments::AngleBracketed(args) => args,
        _ => continue,
      };

      let inner_type_path = match args.args.first()? {
        GenericArgument::Type(Type::Path(inner_type_path)) => inner_type_path,
        _ => continue,
      };

      let inner_segment = inner_type_path.path.segments.last()?;
      // if the inner segment has a `::operations` method. It's should be the correct
      // enum i'll add more checks in the future. For now this is enough
      if syn::parse_str::<syn::Expr>(format!("{}::operations", inner_segment.ident).as_str()).is_ok() {
        return Some(inner_segment.ident.clone());
      } else {
        CONSOLE.warn(format!("Found {}, but it doesn't have the required `::operations` method.", inner_segment.ident));
      }
    }
  }

  None
}
