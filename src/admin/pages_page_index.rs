use askama::Template;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;

#[derive(Template)]
#[template(path = "admin/src/components/generated/pages/page/index.tsx.jinja", escape = "none")]
pub(self) struct PagesPageIndexTemplate {
    name: String
}

pub(crate) async fn generate_pages_page_index_tsx(_namespace: &Namespace, _model: &Model, display_name: &str, path: &str, file_util: &FileUtil) -> teo_result::Result<()> {
    let template = PagesPageIndexTemplate {
        name: display_name.to_owned()
    };
    file_util.ensure_directory_and_generate_file(
        &format!("src/components/generated/pages/{path}/index.tsx"),
        template.render().unwrap()
    ).await?;
    Ok(())
}