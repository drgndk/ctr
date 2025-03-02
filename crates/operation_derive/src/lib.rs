use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, GenericArgument, Ident, PathArguments, Type, parse_macro_input};
mod field;
use field::{Field, FieldTypes};

#[proc_macro_derive(Command, attributes(help, flag, longflag, variadic, operation, subcommand))]
pub fn operation_info_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let cmd_enum_operations = if let Some(cmd_enum) = find_commands_enum(&input) {
    quote! { base.extend(#cmd_enum::operations()); }
  } else {
    quote! {}
  };

  let name = &input.ident;
  let mut essential = Vec::new();
  let mut operations = Vec::new();
  let mut flags = Vec::new();
  let mut variadics = Vec::new();
  let fields = {
    let mut fields = match input.data {
      Data::Struct(data) => Field::get_fields(&data.fields, None),
      Data::Enum(data) => {
        let mut fields: Vec<Field> = vec![];
        for variant in data.variants.iter() {
          let variant_name = variant.ident.to_string();
          let field_name = variant_name.to_lowercase();
          let mut field = Field::default();
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
          let mut _field = Field::default();
          _field.name = field_name;
          _field.from_attributes(&field.attrs);
          fields.push(_field);
        }
        fields
      },
    };
    let mut field = Field::default();
    field.from_attributes(&input.attrs);
    fields.push(field);
    fields
  };

  for field in fields {
    let name = field.name();
    let about = field.about();
    match field.field_type() {
      FieldTypes::Variadic => variadics.push(quote! { common::command::Command::variadic(#name, #about) }),
      FieldTypes::Help => essential.push(quote! { common::command::Command::help_flag() }),
      FieldTypes::Flag => {
        flags.push(
          if let Some(example) = field.example() {
            let example = example.as_str();
            quote! { common::command::Command::option(#name, #example, #about) }
          } else {
            quote! { common::command::Command::flag(#name, #about) }
          },
        );
      },
      FieldTypes::LongFlag => {
        flags.push(
          if let Some(example) = field.example() {
            let example = example.as_str();
            quote! { common::command::Command::long_option(#name, #example, #about) }
          } else {
            quote! { common::command::Command::long_flag(#name, #about) }
          },
        );
      },
      FieldTypes::Operation => {
        operations.push(
          if let Some(example) = field.example() {
            let example = example.as_str();
            quote! { common::command::Command::operation(#name, #example, #about) }
          } else {
            quote! { common::command::Command::command(#name, #about) }
          },
        );
      },
      FieldTypes::Subcommand => operations.push(quote! { common::command::Command::command(#name, #about) }),
      _ => {},
    }
  }

  TokenStream::from(quote! {
    impl #name {
      pub fn operations() -> Vec<Vec<common::command::Command>> {
        let mut base = vec![
          vec![#(#essential),*],
          vec![#(#operations),*],
          vec![#(#flags),*],
          vec![#(#variadics),*]
        ];

        #cmd_enum_operations

        base
      }
    }
  })
}

fn find_commands_enum(input: &DeriveInput) -> Option<Ident> {
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
      }
    }
  }

  None
}
