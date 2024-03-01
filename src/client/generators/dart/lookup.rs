use teo_result::{Error, Result};
use teo_parser::r#type::Type;
use crate::outline::outline::Mode;
use crate::utils::enum_reference_lookup::enum_reference_lookup;
use crate::utils::shape_reference_lookup::shape_reference_lookup;

pub(in crate::client) fn lookup(t: &Type) -> Result<String> {
    Ok(match t {
        Type::Undetermined => Err(Error::new("encountered undetermined"))?,
        Type::Ignored => Err(Error::new("encountered ignored"))?,
        Type::Any | Type::Union(_) | Type::Enumerable(_) => "dynamic".to_owned(),
        Type::Optional(t) => {
            let result = lookup(t)? + "?";
            if result.ends_with("?") {
                result
            } else {
                result + "?"
            }
        },
        Type::FieldType(_, _) => Err(Error::new("encountered field type"))?,
        Type::FieldName(_) => Err(Error::new("encountered field name"))?,
        Type::GenericItem(i) => i.to_owned(),
        Type::Keyword(_) => Err(Error::new("encountered keyword"))?,
        Type::Null => "dynamic".to_owned(),
        Type::Bool => "bool".to_owned(),
        Type::Int | Type::Int64 => "int".to_owned(),
        Type::Float32 | Type::Float => "double".to_owned(),
        Type::Decimal => "Decimal".to_owned(),
        Type::String | Type::ObjectId | Type::Date => "String".to_owned(),
        Type::DateTime => "DateTime".to_owned(),
        Type::File => "TeoFile".to_owned(),
        Type::Regex => Err(Error::new("encountered regex"))?,
        Type::Array(inner) => format!("List<{}>", lookup(inner)?),
        Type::Dictionary(inner) => format!("Map<String, {}>", lookup(inner)?),
        Type::Tuple(_) => Err(Error::new("encountered tuple"))?,
        Type::Range(_) => Err(Error::new("encountered range"))?,
        Type::SynthesizedShapeReference(r) => dart_path_replace_fix(shape_reference_lookup(r, ".", Mode::Client)?),
        Type::EnumVariant(reference) => dart_path_join(reference.string_path()),
        Type::SynthesizedEnumReference(r) => dart_path_replace_fix(enum_reference_lookup(r, ".")?),
        Type::ModelObject(reference) => dart_path_join(reference.string_path()),
        Type::InterfaceObject(reference, types) => if types.is_empty() {
            dart_path_join(reference.string_path())
        } else {
            dart_path_join(reference.string_path()) + "<" + &types.iter().map(|t| lookup(t)).collect::<Result<Vec<String>>>()?.join(", ") + ">"
        },
        _ => Err(Error::new("encountered an unsupported type"))?,
    })
}

fn dart_path_join(items: &Vec<String>) -> String {
    let mut result = "".to_owned();
    for (index, item) in items.iter().enumerate() {
        result.push_str(item.as_str());
        if index == items.len() - 1 {
            // do nothing
        } else if index == items.len() - 2 {
            result.push('.');
        } else {
            result.push('_');
        }
    }
    result
}

fn dart_path_replace_fix(original: String) -> String {
    let components: Vec<String> = original.split(".").map(|s| s.to_string()).collect();
    dart_path_join(&components)
}