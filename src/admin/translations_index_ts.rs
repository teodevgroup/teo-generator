use askama::Template;
use inflector::Inflector;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use teo_result::Result;
use teo_runtime::traits::documentable::Documentable;
use teo_runtime::traits::named::Named;
use crate::utils::file::FileUtil;
use snailquote::escape;

pub(super) struct TranslationEntry {
    pub(super) key: String,
    pub(super) value: String,
}

fn wrap(value: impl AsRef<str>) -> String {
    "\"".to_owned() + escape(value.as_ref()).to_string().as_str() + "\""
}

pub(super) fn fetch_translation_entries(namespace: &Namespace, lang: &'static str) -> Vec<TranslationEntry> {
    let mut result = vec![];
    let models = namespace.collect_models(|m| m.data.get("admin:ignore").is_none());
    for model in models {
        let model_path = model.path().iter().map(|s| s.to_camel_case()).join(".");
        result.push(TranslationEntry {
            key: format!("model.{}.name", model_path),
            value: wrap(&model.title())
        });
        result.push(TranslationEntry {
            key: format!("model.{}.desc", model_path),
            value: wrap(&model.desc())
        });
        for field in model.fields() {
            result.push(TranslationEntry {
                key: format!("model.{}.{}.name", model_path, field.name()),
                value: wrap(&field.title()),
            });
            result.push(TranslationEntry {
                key: format!("model.{}.{}.desc", model_path, field.name()),
                value: wrap(&field.desc()),
            });
        }
        for relation in model.relations() {
            result.push(TranslationEntry {
                key: format!("model.{}.{}.name", model_path, relation.name()),
                value: wrap(&relation.title()),
            });
            result.push(TranslationEntry {
                key: format!("model.{}.{}.desc", model_path, relation.name()),
                value: wrap(&relation.desc()),
            });
        }
        for property in model.properties() {
            result.push(TranslationEntry {
                key: format!("model.{}.{}.name", model_path, property.name()),
                value: wrap(&property.title()),
            });
            result.push(TranslationEntry {
                key: format!("model.{}.{}.desc", model_path, property.name()),
                value: wrap(&property.desc()),
            });
        }
    }
    result
}

#[derive(Template)]
#[template(path = "admin/lib/generated/translations/index.ts.jinja", escape = "none")]
pub(self) struct TranslationsIndexTsTemplate {
    pub(self) entries: Vec<TranslationEntry>,
}

pub(crate) async fn generate_translations_index_ts(namespace: &Namespace, file_util: &FileUtil) -> Result<()> {
    file_util.ensure_directory_and_generate_file("src/lib/generated/translations/index.ts", TranslationsIndexTsTemplate {
        entries: fetch_translation_entries(namespace, "enUs")
    }.render().unwrap()).await?;
    Ok(())
}