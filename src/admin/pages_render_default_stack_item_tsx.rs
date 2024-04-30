use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;

struct StackItemModel {
    class_name: String,
    path: String,
}

#[derive(Template)]
#[template(path = "admin/components/generated/pages/renderDefaultStackItem.tsx.jinja", escape = "none")]
pub(self) struct PagesRenderDefaultStackItemTemplate {
    pub(self) models: Vec<StackItemModel>,
}

pub(crate) async fn generate_pages_render_default_stack_item_tsx(namespace: &Namespace, file_util: &FileUtil) -> teo_result::Result<()> {
    let template = PagesRenderDefaultStackItemTemplate {
        models: namespace.collect_models(|m| m.data.get("admin:ignore").is_none()).iter().map(|m| {
            StackItemModel {
                class_name: m.path().iter().map(|s| s.to_pascal_case()).join(""),
                path: m.path().join("/"),
            }
        }).collect()
    };
    file_util.ensure_directory_and_generate_file("src/components/generated/pages/renderDefaultStackItem.tsx", template.render().unwrap()).await?;
    Ok(())
}