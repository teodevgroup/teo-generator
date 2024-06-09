use teo_result::{Error, Result};
use teo_parser::r#type::Type;
use crate::outline::outline::Mode;
use crate::utils::declared_shape_lookup::declared_shape_lookup;
use crate::utils::enum_reference_lookup::enum_reference_lookup;
use crate::utils::shape_reference_lookup::shape_reference_lookup;

pub(in crate::client) fn lookup(t: &Type) -> Result<String> {
    lookup_inner(t, false)
}

fn lookup_inner(t: &Type, contextual: bool) -> Result<String> {
    Ok(match t {
        Type::Any => if contextual { "AnyCodable".to_string() } else { "AnyCodable".to_string() },
        Type::Union(detailed) => {
            if detailed.len() == 2 {
                if detailed.get(0).unwrap().is_null() {
                    format!("NullOr<{}>", lookup(detailed.get(1).unwrap())?)
                } else if detailed.get(1).unwrap().is_null() {
                    format!("NullOr<{}>", lookup(detailed.get(0).unwrap())?)
                } else {
                    "AnyCodable".to_string()
                }
            } else {
                "AnyCodable".to_string()
            }
        },
        Type::ObjectId => "String".to_string(),
        Type::Bool => "Bool".to_string(),
        Type::Int => "Int32".to_string(),
        Type::Int64 => "Int64".to_string(),
        Type::Float32 => "Float32".to_string(),
        Type::Float => "Double".to_string(),
        Type::Decimal => "Decimal".to_string(),
        Type::String => "String".to_string(),
        Type::Date => "String".to_string(),
        Type::DateTime => "Date".to_string(),
        Type::EnumVariant(reference) => reference.str_path().join("."),
        Type::Optional(inner) => format!("{}?", lookup(inner)?),
        Type::Array(inner) => format!("Array<{}>", lookup_inner(inner, true)?),
        Type::Dictionary(inner) => format!("Dictionary<String, {}>", lookup_inner(inner, true)?),
        Type::FieldType(_, _) => Err(Error::new("encountered field type"))?,
        Type::FieldName(_) => Err(Error::new("encountered field name"))?,
        Type::GenericItem(i) => i.to_owned(),
        Type::Keyword(_) => Err(Error::new("encountered keyword"))?,
        Type::Null => "AnyCodable".to_string(),
        Type::Enumerable(inner) => lookup(&Type::Array(inner.clone()))?,
        Type::SynthesizedShapeReference(shape_reference) => shape_reference_lookup(shape_reference, ".", Mode::Client)?,
        Type::DeclaredSynthesizedShape(reference, model_type) => declared_shape_lookup(reference, model_type.as_ref(), ".")?,
        Type::SynthesizedEnumReference(enum_reference) => enum_reference_lookup(enum_reference, ".")?,
        Type::ModelObject(reference) => reference.string_path().join("."),
        Type::InterfaceObject(reference, types) => if types.is_empty() {
            reference.string_path().join(".")
        } else {
            reference.string_path().join(".") + "<" + &types.iter().map(|t| lookup_inner(t, true)).collect::<Result<Vec<String>>>()?.join(", ") + ">"
        },
        _ => Err(Error::new("encountered an unsupported type"))?,
    })
}