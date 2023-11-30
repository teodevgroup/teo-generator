mod generators;
mod ctx;
mod generator;

use teo_result::Result;
use teo_runtime::config::entity::{Entity, Runtime};
use teo_runtime::namespace::Namespace;
use generators::*;
use crate::entity::ctx::Ctx;
use crate::entity::generator::Generator;
use crate::outline::outline::Outline;
use crate::utils::file::FileUtil;

pub async fn generate(main_namespace: &Namespace, entity: &Entity) -> Result<()> {
    match entity.provider {
        Runtime::Rust => {
            let ctx = Ctx::new(entity, main_namespace);
            let generator = rust::gen::RustGenerator::new();
            gen(generator, &ctx).await
        }
        Runtime::Node => {
            let ctx = Ctx::new(entity, main_namespace);
            let generator = node::gen::NodeGenerator::new();
            gen(generator, &ctx).await
        }
        Runtime::Python => {
            let ctx = Ctx::new(entity, main_namespace);
            let generator = python::gen::PythonGenerator::new();
            gen(generator, &ctx).await
        }
    }
}

async fn gen<T: Generator>(entity_generator: T, ctx: &Ctx<'_>) -> Result<()> {
    let dest = &ctx.conf.dest;
    let dest_dir = std::env::current_dir()?.join(dest);
    let generator = FileUtil::new(&dest_dir);
    generator.ensure_root_directory().await?;
    entity_generator.generate_entity_files(ctx, &generator).await?;
    Ok(())
}
