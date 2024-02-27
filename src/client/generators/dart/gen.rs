use async_trait::async_trait;
use teo_runtime::config::client::Client;
use crate::client::ctx::Ctx;
use crate::client::generator::Generator;
use std::process::Command;
use askama::Template;
use crate::outline::outline::{Mode, Outline};
use crate::utils::exts::ClientExt;
use crate::utils::file::FileUtil;
use crate::utils::message::green_message;
use crate::utils::filters;

#[derive(Template)]
#[template(path = "client/dart/readme.md.jinja", escape = "none")]
pub(self) struct DartReadMeTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/dart/pubspec.yaml.jinja", escape = "none")]
pub(self) struct DartPubspecTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/dart/teo.dart.jinja", escape = "none")]
pub(self) struct DartMainTemplate<'a> {
    pub(self) outline: &'a Outline,
    pub(self) conf: &'a Client,
}

pub(in crate::client) struct DartGenerator {}

impl DartGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Generator for DartGenerator {

    fn module_directory_in_package(&self, conf: &Client) -> String {
        "lib".to_owned()
    }

    async fn generate_module_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.clear_root_directory().await?;
        Ok(())
    }

    async fn generate_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.ensure_root_directory().await?;
        generator.generate_file(".gitignore", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/client/dart/gitignore"))).await?;
        generator.generate_file("README.md", DartReadMeTemplate { conf: ctx.conf }.render().unwrap()).await?;
        generator.generate_file("pubspec.yaml", DartPubspecTemplate { conf: ctx.conf }.render().unwrap()).await?;
        Ok(())
    }

    async fn update_parent_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        Ok(())
    }

    async fn generate_main(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        let outline = Outline::new(ctx.main_namespace, Mode::Client, ctx.main_namespace);
        generator.generate_file(format!("{}.dart", ctx.conf.inferred_package_name_snake_case()), DartMainTemplate {
            outline: &outline,
            conf: ctx.conf,
        }.render().unwrap()).await?;
        // run commands
        let base = generator.get_base_dir();
        let parent = base.parent().unwrap();
        std::env::set_current_dir(parent).unwrap();
        green_message("run", "`dart pub get`".to_owned());
        Command::new("dart").arg("pub").arg("get").spawn()?.wait()?;
        green_message("run", "`dart pub run build_runner build --delete-conflicting-outputs`".to_owned());
        Command::new("dart").arg("pub").arg("run").arg("build_runner").arg("build").arg("--delete-conflicting-outputs").spawn()?.wait()?;
        Ok(())
    }
}
