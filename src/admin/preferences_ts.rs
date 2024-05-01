use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;
use teo_result::Result;
use teo_runtime::traits::named::Named;

pub(super) struct AccountModel {
    pub(super) pascalcase_name: String,
    pub(super) camelcase_name: String,
    pub(super) secure_fields: String,
}

#[derive(Template)]
#[template(path = "admin/lib/generated/preferences.ts.jinja", escape = "none")]
pub(self) struct PreferencesTsTemplate {
    pub(self) account_models: Vec<AccountModel>,
}

fn fetch_template_data(namespace: &Namespace) -> PreferencesTsTemplate {
    let models = namespace.collect_models(|m| m.data.get("admin:administrator").is_some());
    PreferencesTsTemplate {
        account_models: models.iter().map(|m| AccountModel {
            pascalcase_name: m.path().iter().join(""),
            camelcase_name: m.path().iter().join("").to_camel_case(),
            secure_fields: m.fields().iter().filter(|f| f.data.get("admin:secureInput").is_some()).map(|f| format!("\"{}\"", f.name())).join(", ")
        }).collect()
    }
}

pub(crate) async fn generate_preferences_ts(namespace: &Namespace, file_util: &FileUtil) -> Result<()> {
    let template = fetch_template_data(namespace);
    file_util.ensure_directory_and_generate_file("src/lib/generated/preferences.ts", template.render().unwrap()).await?;
    Ok(())
}