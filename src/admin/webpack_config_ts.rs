use askama::Template;
use crate::utils::file::FileUtil;
use teo_result::Result;

#[derive(Template)]
#[template(path = "admin/webpack.config.ts.jinja", escape = "none")]
pub(self) struct WebpackConfigTsTemplate {
    pub(self) port: String,
}

pub(crate) async fn generate_webpack_config_ts(port: String, file_util: &FileUtil) -> Result<()> {
    let template = WebpackConfigTsTemplate {
        port
    };
    file_util.generate_file_if_not_exist("webpack.config.ts", template.render().unwrap()).await?;
    Ok(())
}