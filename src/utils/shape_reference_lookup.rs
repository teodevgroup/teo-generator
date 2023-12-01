use inflector::Inflector;
use teo_parser::r#type::synthesized_shape_reference::{SynthesizedShapeReference, SynthesizedShapeReferenceKind};
use teo_result::Result;
use crate::outline::outline::Mode;

pub(crate) fn shape_reference_lookup(
    shape_reference: &SynthesizedShapeReference,
    path_separator: &str,
    mode: Mode,
) -> Result<String> {
    Ok(match shape_reference.kind {
        SynthesizedShapeReferenceKind::Args => format!("{}Args", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::FindManyArgs => format!("{}FindManyArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::FindFirstArgs => format!("{}FindFirstArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::FindUniqueArgs => format!("{}FindUniqueArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::CreateArgs => format!("{}CreateArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::UpdateArgs => format!("{}UpdateArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::UpsertArgs => format!("{}UpsertArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::CopyArgs => format!("{}CopyArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::DeleteArgs => format!("{}DeleteArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::CreateManyArgs => format!("{}CreateManyArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::UpdateManyArgs => format!("{}UpdateManyArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::CopyManyArgs => format!("{}CopyManyArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::DeleteManyArgs => format!("{}DeleteManyArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::CountArgs => format!("{}CountArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::AggregateArgs => format!("{}AggregateArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::GroupByArgs => format!("{}GroupByArgs", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::RelationFilter => format!("{}RelationFilter", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::ListRelationFilter => format!("{}ListRelationFilter", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::WhereInput => format!("{}WhereInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::WhereUniqueInput => format!("{}WhereUniqueInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::ScalarWhereWithAggregatesInput => format!("{}ScalarWhereWithAggregatesInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::CountAggregateInputType => format!("{}CountAggregateInputType", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::SumAggregateInputType => format!("{}SumAggregateInputType", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::AvgAggregateInputType => format!("{}AvgAggregateInputType", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::MaxAggregateInputType => format!("{}MaxAggregateInputType", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::MinAggregateInputType => format!("{}MinAggregateInputType", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::CreateInput => format!("{}CreateInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::CreateInputWithout => format!("{}CreateWithout{}Input", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator), shape_reference.without.as_ref().unwrap().to_pascal_case()),
        SynthesizedShapeReferenceKind::CreateNestedOneInput => format!("{}CreateNestedOneInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::CreateNestedOneInputWithout => format!("{}CreateNestedOneWithout{}Input", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator), shape_reference.without.as_ref().unwrap().to_pascal_case()),
        SynthesizedShapeReferenceKind::CreateNestedManyInput => format!("{}CreateNestedManyInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::CreateNestedManyInputWithout => format!("{}CreateNestedManyWithout{}Input", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator), shape_reference.without.as_ref().unwrap().to_pascal_case()),
        SynthesizedShapeReferenceKind::UpdateInput => format!("{}UpdateInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::UpdateInputWithout => format!("{}UpdateWithout{}Input", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator), shape_reference.without.as_ref().unwrap().to_pascal_case()),
        SynthesizedShapeReferenceKind::UpdateNestedOneInput => format!("{}UpdateNestedOneInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::UpdateNestedOneInputWithout => format!("{}UpdateNestedOneWithout{}Input", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator), shape_reference.without.as_ref().unwrap().to_pascal_case()),
        SynthesizedShapeReferenceKind::UpdateNestedManyInput => format!("{}UpdateNestedManyInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::UpdateNestedManyInputWithout => format!("{}UpdateNestedManyWithout{}Input", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator), shape_reference.without.as_ref().unwrap().to_pascal_case()),
        SynthesizedShapeReferenceKind::ConnectOrCreateInput => format!("{}ConnectOrCreateInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::ConnectOrCreateInputWithout => format!("{}ConnectOrCreateWithout{}Input", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator), shape_reference.without.as_ref().unwrap().to_pascal_case()),
        SynthesizedShapeReferenceKind::UpdateWithWhereUniqueInput => format!("{}UpdateWithWhereUniqueInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::UpdateWithWhereUniqueInputWithout => format!("{}UpdateWithWhereUniqueWithout{}Input", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator), shape_reference.without.as_ref().unwrap().to_pascal_case()),
        SynthesizedShapeReferenceKind::UpsertWithWhereUniqueInput => format!("{}UpsertWithWhereUniqueInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::UpsertWithWhereUniqueInputWithout => format!("{}UpsertWithWhereUniqueWithout{}Input", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator), shape_reference.without.as_ref().unwrap().to_pascal_case()),
        SynthesizedShapeReferenceKind::UpdateManyWithWhereInput => format!("{}UpdateManyWithWhereInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::UpdateManyWithWhereInputWithout => format!("{}UpdateManyWithWhereWithout{}Input", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator), shape_reference.without.as_ref().unwrap().to_pascal_case()),
        SynthesizedShapeReferenceKind::Select => format!("{}Select", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::Include => format!("{}Include", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::OrderByInput => format!("{}OrderByInput", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::Result => if mode == Mode::Client {
            shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)
        } else {
            format!("{}Result", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator))
        },
        SynthesizedShapeReferenceKind::CountAggregateResult => format!("{}CountAggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::SumAggregateResult => format!("{}SumAggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::AvgAggregateResult => format!("{}AvgAggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::MinAggregateResult => format!("{}MinAggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::MaxAggregateResult => format!("{}MaxAggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::AggregateResult => format!("{}AggregateResult", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
        SynthesizedShapeReferenceKind::GroupByResult => format!("{}GroupByResult", shape_reference.owner.as_model_object().unwrap().string_path().join(path_separator)),
    })
}