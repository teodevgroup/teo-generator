use async_trait::async_trait;
use crate::entity::ctx::Ctx;
use crate::entity::generator::Generator;
use crate::utils::file::FileUtil;

pub(crate) struct PythonGenerator {}

impl PythonGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Generator for PythonGenerator {

    async fn generate_entity_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        Ok(())
    }
}


