use teo_result::{Error, Result};
use teo_parser::r#type::Type;
use crate::utils::enum_reference_lookup::enum_reference_lookup;
use crate::utils::shape_reference_lookup::shape_reference_lookup;

pub(in crate::client) fn lookup(t: &Type) -> Result<String> {
    Ok(match t {
        Type::Undetermined => Err(Error::new("encountered undetermined"))?,
        Type::Ignored => Err(Error::new("encountered ignored"))?,
        Type::Any => "any".to_owned(),
        Type::Null => "null".to_owned(),
        Type::Bool => "boolean".to_owned(),
        Type::Int => "number".to_owned(),
        Type::Int64 => "number".to_owned(),
        Type::Float32 => "number".to_owned(),
        Type::Float => "number".to_owned(),
        Type::Decimal => "Decimal".to_owned(),
        Type::String => "string".to_owned(),
        Type::ObjectId => "ObjectId".to_owned(),
        Type::Date => "DateOnly".to_owned(),
        Type::DateTime => "Date".to_owned(),
        Type::File => "File".to_owned(),
        Type::Regex => Err(Error::new("encountered regex"))?,
        Type::Model => Err(Error::new("encountered model"))?,
        Type::DataSet => Err(Error::new("encountered dataset"))?,
        Type::Enumerable(inner) => format!("Enumerable<{}>", lookup(inner.as_ref())?),
        Type::Array(inner) => if inner.is_union() {
            format!("({})[]", lookup(inner.as_ref())?)
        } else {
            format!("{}[]", lookup(inner.as_ref())?)
        },
        Type::Dictionary(inner) => format!("{{[key: string]: {}}}", lookup(inner.as_ref())?),
        Type::Tuple(t) => format!("[{}]", t.iter().map(|t| lookup(t)).collect::<Result<Vec<String>>>()?.join(", ")),
        Type::Range(_) => "Range".to_owned(),
        Type::Union(types) => types.iter().map(|t| Ok(lookup(t)?)).collect::<Result<Vec<String>>>()?.join(" | "),
        Type::EnumVariant(reference) => reference.string_path().join("."),
        Type::InterfaceObject(reference, types) => if types.is_empty() {
            reference.string_path().join(".")
        } else {
            reference.string_path().join(".") + "<" + &types.iter().map(|t| lookup(t)).collect::<Result<Vec<String>>>()?.join(", ") + ">"
        },
        Type::ModelObject(reference) => reference.string_path().join("."),
        Type::GenericItem(i) => i.to_owned(),
        Type::Optional(inner) => format!("{}?", lookup(inner.as_ref())?),
        Type::SynthesizedShapeReference(shape_reference) => shape_reference_lookup(shape_reference, ".")?,
        Type::SynthesizedEnumReference(enum_reference) => enum_reference_lookup(enum_reference, ".")?,
        _ => Err(Error::new("encountered unhandled type in lookup"))?,
    })
}