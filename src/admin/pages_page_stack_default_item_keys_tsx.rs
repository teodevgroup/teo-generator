use askama::Template;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;

#[derive(Template)]
#[template(path = "admin/components/generated/pages/PageStackDefaultItemKeys.tsx.jinja", escape = "none")]
pub(self) struct PagesPageStackDefaultItemKeysTemplate {
    pub(self) keys: String,
}

pub(crate) async fn generate_pages_stack_default_item_keys_tsx(namespace: &Namespace, file_util: &FileUtil) -> teo_result::Result<()> {
    let template = PagesPageStackDefaultItemKeysTemplate {
        keys: namespace.collect_models(|m| m.data.get("admin:ignore").is_none()).iter().map(|m| {
            let base = m.path().join(".");
            format!("\"{}\" | \"{}Form\"", base, base)
        }).join(" | ")
    };
    file_util.ensure_directory_and_generate_file("src/components/generated/pages/PageStackDefaultItemKeys.tsx", template.render().unwrap()).await?;
    Ok(())
}