use async_trait::async_trait;
use teo_runtime::config::client::Client;
use teo_result::Result;
use crate::client::ctx::Ctx;
use crate::utils::file::FileUtil;

#[async_trait]
pub(in crate::client) trait Generator {

    fn module_directory_in_package(&self, conf: &Client) -> String;

    async fn generate_module_files(&self, ctx: &Ctx, generator: &FileUtil) -> Result<()>;

    async fn generate_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> Result<()>;

    async fn update_parent_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> Result<()>;

    async fn generate_main(&self, ctx: &Ctx, generator: &FileUtil) -> Result<()>;
}
