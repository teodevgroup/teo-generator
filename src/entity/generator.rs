use async_trait::async_trait;
use teo_result::Result;
use crate::entity::ctx::Ctx;
use crate::utils::file::FileUtil;

#[async_trait]
pub(crate) trait Generator {

    async fn generate_entity_files(&self, ctx: &Ctx, generator: &FileUtil) -> Result<()>;
}
