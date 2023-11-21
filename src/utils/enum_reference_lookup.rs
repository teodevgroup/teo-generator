use inflector::Inflector;
use teo_parser::r#type::synthesized_enum_reference::{SynthesizedEnumReference, SynthesizedEnumReferenceKind};
use teo_result::Result;

pub(crate) fn enum_reference_lookup(
    enum_reference: &SynthesizedEnumReference,
    path_separator: &str,
) -> Result<String> {
    Ok(match enum_reference.kind {
        SynthesizedEnumReferenceKind::ModelScalarFields => format!("{}ScalarFields", enum_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedEnumReferenceKind::ModelSerializableScalarFields => format!("{}SerializableScalarFields", enum_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedEnumReferenceKind::ModelRelations => format!("{}Relations", enum_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedEnumReferenceKind::ModelDirectRelations => format!("{}DirectRelations", enum_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedEnumReferenceKind::ModelIndirectRelations => format!("{}IndirectRelations", enum_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
    })
}