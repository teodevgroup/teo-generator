use itertools::Itertools;
use inflector::Inflector;
use teo_result::{Error, Result};
use teo_parser::r#type::Type;
use crate::outline::outline::Mode;
use crate::utils::enum_reference_lookup::enum_reference_lookup;
use crate::utils::shape_reference_lookup::shape_reference_lookup;

pub(crate) fn lookup(t: &Type) -> Result<String> {
    Ok(match t {
        Type::Undetermined => Err(Error::new("encountered undetermined"))?,
        Type::Ignored => Err(Error::new("encountered ignored"))?,
        Type::Any => "Any".to_owned(),
        Type::Null => "None".to_owned(),
        Type::Bool => "bool".to_owned(),
        Type::Int => "int".to_owned(),
        Type::Int64 => "int".to_owned(),
        Type::Float32 => "float".to_owned(),
        Type::Float => "float".to_owned(),
        Type::Decimal => "Decimal".to_owned(),
        Type::String => "str".to_owned(),
        Type::ObjectId => "ObjectId".to_owned(),
        Type::Date => "date".to_owned(),
        Type::DateTime => "datetime".to_owned(),
        Type::File => "File".to_owned(),
        Type::Regex => "Pattern".to_owned(),
        Type::Model => Err(Error::new("encountered model"))?,
        Type::DataSet => Err(Error::new("encountered dataset"))?,
        Type::Enumerable(inner) => format!("Enumerable<{}>", lookup(inner.as_ref())?),
        Type::Array(inner) => format!("list[{}]", lookup(inner.as_ref())?),
        Type::Dictionary(inner) => format!("dict[str, {}]", lookup(inner.as_ref())?),
        Type::Tuple(t) => format!("tuple[{}]", t.iter().map(|t| lookup(t)).collect::<Result<Vec<String>>>()?.join(", ")),
        Type::Range(_) => "Range".to_owned(),
        Type::Union(types) => types.iter().map(|t| Ok(lookup(t)?)).collect::<Result<Vec<String>>>()?.join(" | "),
        Type::EnumVariant(reference) => reference.string_path().iter().map(|s| s.to_snake_case()).join("."),
        Type::InterfaceObject(reference, types) => if types.is_empty() {
            reference.string_path().join(".")
        } else {
            reference.string_path().join(".") + "[" + &types.iter().map(|t| lookup(t)).collect::<Result<Vec<String>>>()?.join(", ") + "]"
        },
        Type::ModelObject(reference) => reference.string_path().iter().map(|s| s.to_snake_case()).join("."),
        Type::GenericItem(i) => i.to_owned(),
        Type::Optional(inner) => format!("Optional[{}]", lookup(inner.as_ref())?),
        Type::SynthesizedShapeReference(shape_reference) => shape_reference_lookup(shape_reference, ".", Mode::Entity)?,
        Type::SynthesizedEnumReference(enum_reference) => enum_reference_lookup(enum_reference, ".")?,
        _ => Err(Error::new("encountered unhandled type in lookup"))?,
    })
}
