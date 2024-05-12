use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;
use teo_result::Result;
use teo_runtime::traits::named::Named;

pub(super) struct ModelForPreferences {
    pub(super) key_name: String,
    pub(super) var_name: String,
    pub(super) fields: Vec<String>,
}

pub(super) struct AccountModel {
    pub(super) pascalcase_name: String,
    pub(super) camelcase_name: String,
    pub(super) secure_fields: String,
}

#[derive(Template)]
#[template(path = "admin/src/lib/generated/preferences.ts.jinja", escape = "none")]
pub(self) struct PreferencesTsTemplate {
    pub(self) account_models: Vec<AccountModel>,
    pub(self) models: Vec<ModelForPreferences>,
}

fn fetch_template_data(namespace: &Namespace) -> PreferencesTsTemplate {
    let models = namespace.collect_models(|m| m.data.get("admin:ignore").is_none());
    let account_models = namespace.collect_models(|m| m.data.get("admin:administrator").is_some());
    PreferencesTsTemplate {
        account_models: account_models.iter().map(|m| AccountModel {
            pascalcase_name: m.path().iter().join(""),
            camelcase_name: m.path().iter().join("").to_camel_case(),
            secure_fields: m.fields().iter().filter(|f| f.data.get("admin:secureInput").is_some()).map(|f| format!("\"{}\"", f.name())).join(", ")
        }).collect(),
        models: models.iter().map(|m| ModelForPreferences {
            key_name: m.path().join("."),
            var_name: m.path().iter().map(|m| m.to_pascal_case()).join(""),
            fields: m.fields().iter().filter(|f| !f.write.is_no_write()).map(|f| f.name().to_string()).collect(),
        }).collect(),
    }
}

pub(crate) async fn generate_preferences_ts(namespace: &Namespace, file_util: &FileUtil) -> Result<()> {
    let template = fetch_template_data(namespace);
    file_util.ensure_directory_and_generate_file("src/lib/generated/preferences.ts", template.render().unwrap()).await?;
    Ok(())
}