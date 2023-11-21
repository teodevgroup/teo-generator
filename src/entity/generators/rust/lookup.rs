use teo_result::{Result, Error};
use teo_parser::r#type::Type;
use crate::utils::enum_reference_lookup::enum_reference_lookup;
use crate::utils::shape_reference_lookup::shape_reference_lookup;

pub(in crate::entity) fn lookup(t: &Type) -> Result<String> {
    Ok(match t {
        Type::Undetermined => Err(Error::new("encountered undetermined"))?,
        Type::Ignored => Err(Error::new("encountered ignored"))?,
        Type::Any => "Value".to_owned(),
        Type::Null => "Option<Value>".to_owned(),
        Type::Bool => "bool".to_owned(),
        Type::Int => "i32".to_owned(),
        Type::Int64 => "i64".to_owned(),
        Type::Float32 => "f32".to_owned(),
        Type::Float => "f64".to_owned(),
        Type::Decimal => "BigDecimal".to_owned(),
        Type::String => "String".to_owned(),
        Type::ObjectId => "ObjectId".to_owned(),
        Type::Date => "NaiveDate".to_owned(),
        Type::DateTime => "DateTime<Utc>".to_owned(),
        Type::File => "File".to_owned(),
        Type::Regex => Err(Error::new("encountered regex"))?,
        Type::Model => Err(Error::new("encountered model"))?,
        Type::DataSet => Err(Error::new("encountered dataset"))?,
        Type::Enumerable(_) => "Value".to_owned(),
        Type::Array(inner) => format!("Vec<{}>", lookup(inner.as_ref())?),
        Type::Dictionary(inner) => format!("IndexMap<String, {}>", lookup(inner.as_ref())?),
        Type::Tuple(t) => format!("({})", t.iter().map(|t| lookup(t)).collect::<Result<Vec<String>>>()?.join(", ")),
        Type::Range(_) => "Range".to_owned(),
        Type::Union(_) => "Value".to_owned(),
        Type::EnumVariant(reference) => reference.string_path().join("::"),
        Type::InterfaceObject(_, _) => "Value".to_owned(),
        Type::ModelObject(reference) => reference.string_path().join("::"),
        Type::GenericItem(i) => i.to_owned(),
        Type::Optional(inner) => format!("Option<{}>", lookup(inner.as_ref())?),
        Type::SynthesizedShapeReference(shape_reference) => shape_reference_lookup(shape_reference, "::")?,
        Type::SynthesizedEnumReference(enum_reference) => enum_reference_lookup(enum_reference, "::")?,
        _ => Err(Error::new("encountered unhandled type in lookup"))?,
    })
}