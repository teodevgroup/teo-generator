use async_trait::async_trait;
use teo_runtime::config::client::Client;
use crate::client::ctx::Ctx;
use crate::client::generator::Generator;

use crate::utils::file::FileUtil;

pub(in crate::client) struct CSharpGenerator {}

impl CSharpGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Generator for CSharpGenerator {

    fn module_directory_in_package(&self, conf: &Client) -> String {
        todo!()
    }

    async fn generate_module_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        todo!()
    }

    async fn generate_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        todo!()
    }

    async fn generate_main(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        todo!()
    }
}


