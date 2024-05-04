use askama::Template;
use crate::utils::file::FileUtil;
use teo_result::Result;

#[derive(Template)]
#[template(path = "admin/webpack.config.js.jinja", escape = "none")]
pub(self) struct WebpackConfigJsTemplate {
    pub(self) port: String,
}

pub(crate) async fn generate_webpack_config_js(port: String, file_util: &FileUtil) -> Result<()> {
    let template = WebpackConfigJsTemplate {
        port
    };
    file_util.generate_file_if_not_exist("webpack.config.js", template.render().unwrap()).await?;
    Ok(())
}