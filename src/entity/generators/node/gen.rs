use askama::Template;
use async_trait::async_trait;
use teo_result::Result;
use teo_runtime::namespace::Namespace;
use crate::entity::ctx::Ctx;
use crate::entity::generator::Generator;
use crate::outline::outline::Mode;
use crate::shared::ts::conf::TsConf;
use crate::shared::ts::templates::{render_namespace, TsIndexDTsTemplate};
use crate::utils::file::FileUtil;

#[derive(Template)]
#[template(path = "entity/nodejs/index.js.jinja", escape = "none")]
pub(self) struct TsIndexJsTemplate { }

pub(crate) struct NodeGenerator {}

impl NodeGenerator {

    pub fn new() -> Self {
        Self {}
    }

    pub async fn generate_index_js(&self, generator: &FileUtil) -> Result<()> {
        generator.generate_file("index.js", TsIndexJsTemplate { }.render().unwrap()).await
    }

    pub async fn generate_index_d_ts(&self, main_namespace: &Namespace, generator: &FileUtil) -> Result<()> {
        generator.generate_file("index.d.ts", TsIndexDTsTemplate {
            main_namespace,
            conf: &TsConf::new("teo".to_string(), "Teo".to_string(), false),
            render_namespace: &render_namespace,
            mode: Mode::Entity,
        }.render().unwrap()).await
    }
}

#[async_trait]
impl Generator for NodeGenerator {

    async fn generate_entity_files(&self, ctx: &Ctx, generator: &FileUtil) -> Result<()> {
        // index.js
        self.generate_index_js(generator).await?;
        // index.d.ts
        self.generate_index_d_ts(ctx.main_namespace, generator).await?;
        Ok(())
    }
}


