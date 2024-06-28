use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::admin::language::Language;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::named::Named;
use crate::admin::preferences_ts::ModelForPreferences;
use crate::utils::file::FileUtil;

pub(self) struct NavItem {
    pub(self) path: String,
    pub(self) name: String,
}

pub(self) struct SignInModel {
    pub(self) camelcase_name: String,
    pub(self) default_id_key: String,
    pub(self) default_checker_key: String,
}

#[derive(Template)]
#[template(path = "admin/src/lib/generated/defaultPreferences.ts.jinja", escape = "none")]
pub(self) struct DefaultPreferencesTsTemplate {
    pub(self) default_lang: String,
    pub(self) nav_items: Vec<NavItem>,
    pub(self) default_sign_in_model: String,
    pub(self) sign_in_models: Vec<SignInModel>,
    pub(self) models: Vec<ModelForPreferences>,
}

pub(crate) async fn generate_default_preferences_ts(namespace: &Namespace, languages: &Vec<Language>, file_util: &FileUtil) -> teo_result::Result<()> {
    let models = namespace.collect_models(|m| m.data().get("admin:ignore").is_none());
    let sign_in_models = namespace.collect_models(|m| m.data().get("admin:administrator").is_some());
    let template = DefaultPreferencesTsTemplate {
        default_lang: languages.first().unwrap().as_str().to_string(),
        nav_items: namespace.collect_models(|m| m.data().get("admin:ignore").is_none()).iter().map(|m| NavItem {
            path: m.path().join("/"),
            name: format!("model.{}.name", m.path().iter().map(|s| s.to_camel_case()).join(".")),
        }).collect(),
        default_sign_in_model: sign_in_models.first().map_or("".to_owned(), |m| m.path().join(".")),
        sign_in_models: sign_in_models.iter().map(|m| SignInModel {
            camelcase_name: m.path().iter().map(|s| s.to_camel_case()).join("."),
            default_id_key: m.fields().values().find(|f| f.data().get("identity:id").is_some()).map_or("".to_owned(), |f| f.name().to_string()),
            default_checker_key: m.fields().values().find(|f| f.data().get("identity:checker").is_some()).map_or("".to_owned(), |f| f.name().to_string()),
        }).collect(),
        models: models.iter().map(|m| ModelForPreferences {
            key_name: m.path().join("."),
            var_name: m.path().iter().map(|m| m.to_pascal_case()).join(""),
            fields: m.fields().values().filter(|f| !f.write().is_no_write() && !f.foreign_key()).map(|f| f.name().to_string()).collect(),
        }).collect(),
    };
    file_util.ensure_directory_and_generate_file("src/lib/generated/defaultPreferences.ts", template.render().unwrap()).await?;
    Ok(())
}