use common::{console::CONSOLE, enum_gen, struct_gen};
use syn::{ Fields, Lit, Meta, NestedMeta };

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
            ident_name if vec!["name", "example", "about"].contains(&ident_name) => {
              if let Lit::Str(lit_str) = &name_value.lit {
                let value = lit_str.value();
                match ident_name {
                  "name" => self.name = value,
                  "example" => self.example = Some(value),
                  _ => self.about = value
                }
              }
            },
            ident_name => {
              CONSOLE.panic(format!("\"{ident_name}\" is not a valid OperationInfo attribute"));
            }
          }
        }
      }

      self
    }

    pub fn from_attributes(self: &mut Self, attributes: &Vec<syn::Attribute>) -> &mut Self {
      let mut attributes = attributes.iter();
      while let Some(attr) = attributes.next() {
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
        if let Err(_) = meta {
          CONSOLE.panic("Unexpected attribute meta");
        }

        if let Meta::List(meta_list) = meta.unwrap() {
          for nested_meta in meta_list.nested.iter() {
            match nested_meta {
              NestedMeta::Meta(meta) => {
                self.update_field(meta);
              },
              NestedMeta::Lit(lit) => {
                match lit {
                  Lit::Str(lit_str) => {
                    self.about = lit_str.value();
                  },
                  _ => {}
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
          for field in fields.named.iter() {
            if field.attrs.is_empty() {
              continue;
            }

            field_list.push({
              let mut _field = Self::new();

              let field_name = field.ident.as_ref().unwrap().to_string();
              _field.name = name.to_owned().unwrap_or(&field_name).to_lowercase();

              _field.from_attributes(&field.attrs);
              _field
            });
          }
        },
        // Enums [Experimental]
        Fields::Unnamed(fields) => {
          for (index, field) in fields.unnamed.iter().enumerate() {
            if field.attrs.is_empty() {
              continue;
            }

            field_list.push({
              let mut _field = Self::new();
              let field_name = format!("variant field_{index}");
              _field.name = name.to_owned().unwrap_or(&field_name).to_lowercase();

              _field.from_attributes(&field.attrs);
              _field
            });
          }
        },
        Fields::Unit => CONSOLE.panic("Unit struct not supported"),
      };

      field_list
    }
  }
}
