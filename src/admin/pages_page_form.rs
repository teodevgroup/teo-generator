use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::named::Named;
use crate::utils::file::FileUtil;

struct PageFormField {
    display_name: String,
    name: String,
    secure: bool,
}

#[derive(Template)]
#[template(path = "admin/components/generated/pages/page/Form.tsx.jinja", escape = "none")]
pub(self) struct PagesPageFormTemplate {
    name: String,
    imports: String, // Admin, AdminCreateInput, AdminUpdateInput
    partial_type_combined: String, // Admin & AdminCreateInput & AdminUpdateInput
    model_dot_path: String, // admin
    fields: Vec<PageFormField>,
    omit_in_default: String,
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
                    })
                }
            }
            result
        },
        omit_in_default: {
            let mut list: Vec<String> = vec![];
            for field in model.fields() {
                if !field.write.is_no_write() {
                    list.push(field.name().to_owned());
                }
            }
            for property in model.properties() {
                list.push(property.name().to_owned())
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