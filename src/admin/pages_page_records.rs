use askama::Template;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;

#[derive(Template)]
#[template(path = "admin/components/generated/pages/page/Records.tsx.jinja", escape = "none")]
pub(self) struct PagesPageRecordsTemplate {
    name: String
}

pub(crate) async fn generate_pages_page_records_tsx(_namespace: &Namespace, _model: &Model, display_name: &str, path: &str, file_util: &FileUtil) -> teo_result::Result<()> {
    let template = PagesPageRecordsTemplate {
        name: display_name.to_owned()
    };
    file_util.ensure_directory_and_generate_file(
        &format!("src/components/generated/pages/{path}/Records.tsx.jinja"),
        template.render().unwrap()
    ).await?;
    Ok(())
}