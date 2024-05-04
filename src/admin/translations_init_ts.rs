use teo_runtime::namespace::Namespace;
use crate::utils::file::FileUtil;
use askama::Template;
use teo_result::Result;
use teo_runtime::admin::language::Language;

#[derive(Template)]
#[template(path = "admin/src/lib/generated/translations/init.ts.jinja", escape = "none")]
pub(self) struct TranslationsInitTsTemplate {
    pub(self) languages: Vec<Language>,
}

pub(crate) async fn generate_translations_init_ts(languages: &Vec<Language>, file_util: &FileUtil) -> Result<()> {
    file_util.ensure_directory_and_generate_file("src/lib/generated/translations/init.ts", TranslationsInitTsTemplate {
        languages: languages.clone()
    }.render().unwrap()).await?;
    Ok(())
}
