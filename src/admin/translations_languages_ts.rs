use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;
use askama::Template;
use itertools::Itertools;
use teo_result::Result;
use teo_runtime::admin::language::Language;

struct LanguageItem {
    name: &'static str,
    display: &'static str,
}

#[derive(Template)]
#[template(path = "admin/lib/generated/translations/languages.ts.jinja", escape = "none")]
pub(self) struct TranslationsLanguageTsTemplate {
    pub(self) joined_languages: String,
    pub(self) languages: Vec<LanguageItem>,
}

pub(crate) async fn generate_translations_languages_ts(languages: &Vec<Language>, file_util: &FileUtil) -> Result<()> {
    file_util.ensure_directory_and_generate_file("src/lib/generated/translations/languages.ts", TranslationsLanguageTsTemplate {
        languages: languages.iter().map(|l| LanguageItem {
            name: l.as_str(),
            display: l.display(),
        }).collect(),
        joined_languages: languages.iter().map(|l| format!("\"{}\"", l.as_str())).join(" | ")
    }.render().unwrap()).await?;
    Ok(())
}
