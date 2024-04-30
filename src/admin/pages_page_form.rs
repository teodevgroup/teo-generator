use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;

#[derive(Template)]
#[template(path = "admin/components/generated/pages/page/Form.tsx.jinja", escape = "none")]
pub(self) struct PagesPageFormTemplate {
    name: String,
    imports: String, // Admin, AdminCreateInput, AdminUpdateInput
    partial_type_combined: String, // Admin & AdminCreateInput & AdminUpdateInput
    model_dot_path: String, // admin
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
    };
    file_util.ensure_directory_and_generate_file(
        &format!("/templates/admin/components/generated/pages/{path}/Form.tsx.jinja"),
        template.render().unwrap()
    ).await?;
    Ok(())
}