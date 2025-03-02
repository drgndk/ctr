use common::{console::CONSOLE, enum_gen, struct_gen};
use syn::{Fields, Lit, Meta, NestedMeta};

enum_gen! {
  pub enum FieldTypes use Clone {
    Help,
    Variadic,
    Flag,
    LongFlag,
    Operation,
    Subcommand,
    Invalid
  }
}

struct_gen! {
  pub struct Field {
    let field_type: FieldTypes = FieldTypes::Invalid;
    pub let name: String = String::new();
    let about: String = String::new();
    let example: Option<String> = None;
  }

  mod implementation {
    fn update_field(self: &mut Self, meta: &Meta) -> &mut Self {
      if let Meta::NameValue(name_value) = meta {
        if let Some(ident) = name_value.path.get_ident().map(|i| i.to_string()) {
          match ident.as_str() {
            ident_name if ["name", "example", "about"].contains(&ident_name) => {
              if let Lit::Str(lit_str) = &name_value.lit {
                let value = lit_str.value();
                match ident_name {
                  "name" => self.name = value,
                  "example" => self.example = Some(value),
                  _ => self.about = value
                }
              }
            },
            ident_name => CONSOLE.panic(format!("\"{ident_name}\" is not a valid OperationInfo attribute"))
          }
        }
      }

      self
    }

    pub fn from_attributes(self: &mut Self, attributes: &[syn::Attribute]) -> &mut Self {
      for attr in attributes.iter() {
        let field_name = attr.path.get_ident().map(|i| i.to_string());

        match field_name {
          Some(field_name) => {
            if let Some(field) = FieldTypes::try_from(&field_name) {
              self.field_type = field;
            } else {
              continue;
            }
          },
          None => continue
        }

        let meta = attr.parse_meta();
        if meta.is_err() {
          CONSOLE.panic("Unexpected attribute meta");
        }

        if let Meta::List(meta_list) = meta.unwrap() {
          for nested_meta in meta_list.nested.iter() {
            match nested_meta {
              NestedMeta::Meta(meta) => {
                self.update_field(meta);
              },
              NestedMeta::Lit(lit) => {
                if let Lit::Str(lit_str) = lit {
                  self.about = lit_str.value();
                }
              }
            }
          }
        }
      }

      self
    }

    pub fn get_fields(fields: &Fields, name: Option<String>) -> Vec<Self> {
      let mut field_list = vec![];

      let name = name.as_ref();

      match fields {
        // Structs
        Fields::Named(fields) => {
          for named_field in fields.named.iter() {
            if named_field.attrs.is_empty() {
              continue;
            }

            field_list.push({
              let mut field = Self::default();

              let field_name = named_field.ident.as_ref().unwrap().to_string();
              field.name = name.to_owned().unwrap_or(&field_name).to_lowercase();

              field.from_attributes(&named_field.attrs);
              field
            });
          }
        },
        // Enums [Experimental]
        Fields::Unnamed(fields) => {
          for (index, unnamed_field) in fields.unnamed.iter().enumerate() {
            if unnamed_field.attrs.is_empty() {
              continue;
            }

            field_list.push({
              let mut field = Self::default();
              let field_name = format!("variant field_{index}");
              field.name = name.to_owned().unwrap_or(&field_name).to_lowercase();

              field.from_attributes(&unnamed_field.attrs);
              field
            });
          }
        },
        Fields::Unit => CONSOLE.panic("Unit struct not supported"),
      };

      field_list
    }
  }
}
