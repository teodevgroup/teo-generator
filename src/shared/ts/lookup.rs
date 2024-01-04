use teo_parser::r#type::synthesized_shape_reference::{SynthesizedShapeReference, SynthesizedShapeReferenceKind};
use teo_result::{Error, Result};
use teo_parser::r#type::Type;
use crate::outline::outline::Mode;
use crate::utils::enum_reference_lookup::enum_reference_lookup;
use crate::utils::shape_reference_lookup::shape_reference_lookup;

pub(crate) fn lookup(t: &Type, ts_result_mode: bool) -> Result<String> {
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
        Type::Enumerable(inner) => format!("Enumerable<{}>", lookup(inner.as_ref(), ts_result_mode)?),
        Type::Array(inner) => if ts_result_mode {
            if let Some(shape_reference) = inner.as_synthesized_shape_reference() {
                ts_result_shape_reference_lookup(shape_reference)?
            } else if inner.is_union() {
                format!("({})[]", lookup(inner.as_ref(), ts_result_mode)?)
            } else {
                format!("{}[]", lookup(inner.as_ref(), ts_result_mode)?)
            }
        } else {
            if inner.is_union() {
                format!("({})[]", lookup(inner.as_ref(), ts_result_mode)?)
            } else {
                format!("{}[]", lookup(inner.as_ref(), ts_result_mode)?)
            }
        },
        Type::Dictionary(inner) => format!("{{[key: string]: {}}}", lookup(inner.as_ref(), ts_result_mode)?),
        Type::Tuple(t) => format!("[{}]", t.iter().map(|t| lookup(t, ts_result_mode)).collect::<Result<Vec<String>>>()?.join(", ")),
        Type::Range(_) => "Range".to_owned(),
        Type::Union(types) => types.iter().map(|t| Ok(lookup(t, ts_result_mode)?)).collect::<Result<Vec<String>>>()?.join(" | "),
        Type::EnumVariant(reference) => reference.string_path().join("."),
        Type::InterfaceObject(reference, types) => if types.is_empty() {
            reference.string_path().join(".")
        } else {
            reference.string_path().join(".") + "<" + &types.iter().map(|t| lookup(t, ts_result_mode)).collect::<Result<Vec<String>>>()?.join(", ") + ">"
        },
        Type::ModelObject(reference) => reference.string_path().join("."),
        Type::GenericItem(i) => i.to_owned(),
        Type::Optional(inner) => if ts_result_mode {
            if let Some(shape_reference) = inner.as_synthesized_shape_reference() {
                ts_result_shape_reference_lookup(shape_reference)?
            } else {
                format!("{}?", lookup(inner.as_ref(), ts_result_mode)?)
            }
        } else {
            format!("{}?", lookup(inner.as_ref(), ts_result_mode)?)
        },
        Type::SynthesizedShapeReference(shape_reference) => if ts_result_mode {
            ts_result_shape_reference_lookup(shape_reference)?
        } else {
            shape_reference_lookup(shape_reference, ".", Mode::Client)?
        },
        Type::SynthesizedEnumReference(enum_reference) => enum_reference_lookup(enum_reference, ".")?,
        _ => Err(Error::new("encountered unhandled type in lookup"))?,
    })
}

pub(crate) fn ts_result_shape_reference_lookup(shape_reference: &SynthesizedShapeReference) -> Result<String> {
    Ok(match shape_reference.kind {
        SynthesizedShapeReferenceKind::Result => {
            let base = shape_reference.owner.as_model_object().unwrap().string_path().join(".");
            format!("CheckSelectInclude<T, {base}, {base}GetPayload<T>>")
        },
        SynthesizedShapeReferenceKind::CountAggregateResult => format!("{}CountAggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(".")),
        SynthesizedShapeReferenceKind::SumAggregateResult => format!("{}SumAggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(".")),
        SynthesizedShapeReferenceKind::AvgAggregateResult => format!("{}AvgAggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(".")),
        SynthesizedShapeReferenceKind::MinAggregateResult => format!("{}MinAggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(".")),
        SynthesizedShapeReferenceKind::MaxAggregateResult => format!("{}MaxAggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(".")),
        SynthesizedShapeReferenceKind::AggregateResult => format!("{}AggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(".")),
        SynthesizedShapeReferenceKind::GroupByResult => format!("{}GroupByResult", shape_reference.owner.as_model_object().unwrap().string_path().join(".")),
        _ => shape_reference_lookup(shape_reference, ".", Mode::Client)?,
    })
}