use askama::Template;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;

#[derive(Template)]
#[template(path = "admin/components/generated/pages/page/Form.tsx.jinja", escape = "none")]
pub(self) struct PagesPageFormTemplate {
    name: String
}

pub(crate) async fn generate_pages_page_form_tsx(_namespace: &Namespace, _model: &Model, display_name: &str, path: &str, file_util: &FileUtil) -> teo_result::Result<()> {
    let template = PagesPageFormTemplate {
        name: display_name.to_owned()
    };
    file_util.ensure_directory_and_generate_file(
        &format!("/templates/admin/components/generated/pages/{path}/Form.tsx.jinja"),
        template.render().unwrap()
    ).await?;
    Ok(())
}