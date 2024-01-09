use std::borrow::Cow;

pub(in crate::gen) fn should_escape_dart(&self) -> bool {
    self.name.starts_with("_") || ["is", "AND", "OR", "NOT"].contains(&self.name)
}

pub(in crate::gen) fn type_is_not_dynamic_dart(&self) -> bool {
    self.field_type.as_ref() != "dynamic"
}

pub(in crate::gen) fn type_is_dynamic_dart(&self) -> bool {
    self.field_type.as_ref() == "dynamic"
}

pub(in crate::gen) fn value_for_data_transformer_dart(&self, action_name: &str) -> Cow<'a, str> {
    match action_name {
        "findUnique" | "findFirst" | "create" | "update" | "upsert" | "delete" | "signIn" | "identity" => Cow::Owned(format!("(p0) => {}.fromJson(p0)", self.model_name.as_ref())),
        "findMany" | "createMany" | "updateMany" | "deleteMany" => Cow::Owned(format!("(p0) => p0.map<{}>((e) => {}.fromJson(e)).toList() as List<{}>", self.model_name.as_ref(), self.model_name.as_ref(), self.model_name.as_ref())),
        _ => Cow::Borrowed("(p0) => p0"),
    }
}

pub(in crate::gen) fn value_for_meta_transformer_dart(&self, action_name: &str) -> &'static str {
    match action_name {
        "findMany" | "createMany" | "updateMany" | "deleteMany" => "(p0) => PagingInfo.fromJson(p0)",
        "signIn" => "(p0) => TokenInfo.fromJson(p0)",
        _ => "null",
    }
}