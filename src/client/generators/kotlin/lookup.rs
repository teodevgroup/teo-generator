use teo_result::{Error, Result};
use teo_parser::r#type::Type;
use crate::outline::outline::Mode;
use crate::utils::declared_shape_lookup::declared_shape_lookup;
use crate::utils::enum_reference_lookup::enum_reference_lookup;
use crate::utils::shape_reference_lookup::shape_reference_lookup;

pub(in crate::client) fn lookup(t: &Type) -> Result<String> {
    Ok(match t {
        Type::Any => "Any".to_string(),
        Type::ObjectId => "String".to_string(),
        Type::Bool => "Boolean".to_string(),
        Type::Int => "Int".to_string(),
        Type::Int64 => "Long".to_string(),
        Type::Float32 => "Float".to_string(),
        Type::Float => "Double".to_string(),
        Type::Decimal => "BigDecimal".to_string(),
        Type::String => "String".to_string(),
        Type::Date => "LocalDate".to_string(),
        Type::DateTime => "OffsetDateTime".to_string(),
        Type::EnumVariant(reference) => reference.str_path().join("."),
        Type::Optional(inner) => format!("{}?", lookup(inner)?),
        Type::Array(inner) => format!("List<{}>", lookup(inner)?),
        Type::Dictionary(inner) => format!("Map<String, {}>", lookup(inner)?),
        Type::FieldType(_, _) => Err(Error::new("encountered field type"))?,
        Type::FieldName(_) => Err(Error::new("encountered field name"))?,
        Type::GenericItem(i) => i.to_owned(),
        Type::Keyword(_) => Err(Error::new("encountered keyword"))?,
        Type::Null => "Any".to_string(),
        Type::Enumerable(_) => "Any".to_string(),
        Type::SynthesizedShapeReference(shape_reference) => shape_reference_lookup(shape_reference, ".", Mode::Client)?,
        Type::DeclaredSynthesizedShape(reference, model_type) => declared_shape_lookup(reference, model_type.as_ref(), ".")?,
        Type::SynthesizedEnumReference(enum_reference) => enum_reference_lookup(enum_reference, ".")?,
        Type::ModelObject(reference) => reference.string_path().join("."),
        Type::InterfaceObject(reference, types) => if types.is_empty() {
            reference.string_path().join(".")
        } else {
            reference.string_path().join(".") + "<" + &types.iter().map(|t| lookup(t)).collect::<Result<Vec<String>>>()?.join(", ") + ">"
        },
        _ => Err(Error::new("encountered an unsupported type"))?,
    })
}
