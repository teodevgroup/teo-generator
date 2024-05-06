use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_parser::r#type::Type;
use teo_runtime::model::field::typed::Typed;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::named::Named;
use crate::utils::file::FileUtil;

struct PageFormField {
    display_name: String,
    name: String,
    secure: bool,
    type_hint: String,
    optional: bool,
}

#[derive(Template)]
#[template(path = "admin/src/components/generated/pages/page/Form.tsx.jinja", escape = "none")]
pub(self) struct PagesPageFormTemplate {
    name: String,
    imports: String, // Admin, AdminCreateInput, AdminUpdateInput
    partial_type_combined: String, // Admin & AdminCreateInput & AdminUpdateInput
    model_dot_path: String, // admin
    fields: Vec<PageFormField>,
    omit_in_default: String,
}

fn type_hint(t: &Type) -> String {
    match t {
        Type::String => "String".to_owned(),
        Type::ObjectId => "String".to_owned(),
        Type::Bool => "Bool".to_owned(),
        Type::Date => "Date".to_owned(),
        Type::DateTime => "DateTime".to_string(),
        Type::Decimal => "Decimal".to_string(),
        Type::Int => "Int".to_string(),
        Type::Int64 => "Int64".to_string(),
        Type::Float => "Float".to_string(),
        Type::Float32 => "Float32".to_string(),
        Type::Array(_) => "Array".to_string(),
        _ => "".to_owned(),
    }
}

pub(crate) async fn generate_pages_page_form_tsx(_namespace: &Namespace, model: &Model, display_name: &str, path: &str, file_util: &FileUtil) -> teo_result::Result<()> {
    let template = PagesPageFormTemplate {
        name: display_name.to_owned(),
        imports: if model.path().len() == 1 {
            let stem = model.path().first().unwrap().to_string();
            format!("{}, {}CreateInput, {}UpdateInput", stem, stem, stem)
        } else {
            model.path().first().unwrap().to_string()
        },
        partial_type_combined: {
            let joined = model.path().join(".");
            format!("{} & {}CreateInput & {}UpdateInput", joined, joined, joined)
        },
        model_dot_path: model.path.iter().map(|s| s.to_camel_case()).join("."),
        fields: {
            let mut result = vec![];
            let model_path = model.path().iter().map(|s| s.to_camel_case()).join(".");
            for field in model.fields() {
                if !field.write.is_no_write() {
                    result.push(PageFormField {
                        display_name: format!("model.{}.{}.name", model_path, field.name()),
                        name: field.name().to_owned(),
                        secure: field.data.get("admin:secureInput").is_some(),
                        type_hint: type_hint(field.r#type().unwrap_optional()),
                        optional: field.r#type.is_optional(),
                    })
                }
            }
            result
        },
        omit_in_default: {
            let mut list: Vec<String> = vec![];
            for field in model.fields() {
                if field.write.is_no_write() {
                    list.push(field.name().to_owned());
                }
            }
            list.iter().map(|item| format!("\"{}\"", item)).join(", ")
        },
    };
    file_util.ensure_directory_and_generate_file(
        &format!("src/components/generated/pages/{path}/Form.tsx"),
        template.render().unwrap()
    ).await?;
    Ok(())
}