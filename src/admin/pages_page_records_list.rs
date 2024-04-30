use askama::Template;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;

#[derive(Template)]
#[template(path = "admin/components/generated/pages/page/RecordsList.tsx.jinja", escape = "none")]
pub(self) struct PagesPageRecordsListTemplate {
    name: String,
    double_open: &'static str,
}

pub(crate) async fn generate_pages_page_records_list_tsx(_namespace: &Namespace, _model: &Model, display_name: &str, path: &str, file_util: &FileUtil) -> teo_result::Result<()> {
    let template = PagesPageRecordsListTemplate {
        name: display_name.to_owned(),
        double_open: "{{",
    };
    file_util.ensure_directory_and_generate_file(
        &format!("/templates/admin/components/generated/pages/{path}/RecordsList.tsx.jinja"),
        template.render().unwrap()
    ).await?;
    Ok(())
}