use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;
use askama::Template;
use teo_result::Result;
use crate::admin::translations_index_ts::{fetch_translation_entries, TranslationEntry};

#[derive(Template)]
#[template(path = "admin/lib/generated/translations/lang/index.ts.jinja", escape = "none")]
pub(self) struct TranslationsLangIndexTsTemplate {
    pub(self) entries: Vec<TranslationEntry>,
}

pub(crate) async fn generate_translations_lang_index_ts(lang: &'static str, namespace: &Namespace, file_util: &FileUtil) -> Result<()> {
    file_util.ensure_directory_and_generate_file(&format!("src/lib/generated/translations/{}/index.ts", lang), TranslationsLangIndexTsTemplate {
        entries: fetch_translation_entries(namespace, lang)
    }.render().unwrap()).await?;
    Ok(())
}
