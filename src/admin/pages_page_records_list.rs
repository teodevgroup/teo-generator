use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::model::field::typed::Typed;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::named::Named;
use crate::utils::file::FileUtil;

pub(self) struct RecordsListField {
    title_in_header: String, // Id, Email in i18n form
    fetch_value: String, // item.id, item.email
    enum_name: Option<String>, // the enum name in the enum definitions
}

#[derive(Template)]
#[template(path = "admin/src/components/generated/pages/page/RecordsList.tsx.jinja", escape = "none")]
pub(self) struct PagesPageRecordsListTemplate {
    name: String,
    double_open: &'static str,
    single_open: &'static str,
    model_dot_path: String,
    fields: Vec<RecordsListField>,
    primary_fields: String,
}

pub(crate) async fn generate_pages_page_records_list_tsx(_namespace: &Namespace, model: &Model, display_name: &str, path: &str, file_util: &FileUtil) -> teo_result::Result<()> {
    let template = PagesPageRecordsListTemplate {
        name: display_name.to_owned(),
        double_open: "{{",
        single_open: "{",
        model_dot_path: model.path().iter().map(|s| s.to_camel_case()).join("."),
        fields: {
            let model_path = model.path().iter().map(|s| s.to_camel_case()).join(".");
            let mut result = vec![];
            for field in model.fields().values() {
                if !field.read().is_no_read() && !field.foreign_key() {
                    result.push(RecordsListField {
                        title_in_header: format!("model.{}.{}.name", model_path, field.name()),
                        fetch_value: format!("item.{}", field.name()),
                        enum_name: if let Some(e) = field.r#type().unwrap_optional().unwrap_array().unwrap_optional().as_enum_variant() {
                            Some(e.str_path().join("."))
                        } else {
                            None
                        },
                    });
                }
            }
            for property in model.properties().values() {
                if property.getter().is_some() {
                    result.push(RecordsListField {
                        title_in_header: format!("model.{}.{}.name", model_path, property.name()),
                        fetch_value: format!("item.{}", property.name()),
                        enum_name: if let Some(e) = property.r#type().unwrap_optional().unwrap_array().unwrap_optional().as_enum_variant() {
                            Some(e.str_path().join("."))
                        } else {
                            None
                        },
                    });
                }
            }
            result
        },
        primary_fields: model.primary_index().unwrap().items().iter().map(|i| format!("\"{}\"", i.field)).join(", ")
    };
    file_util.ensure_directory_and_generate_file(
        &format!("src/components/generated/pages/{path}/RecordsList.tsx"),
        template.render().unwrap()
    ).await?;
    Ok(())
}