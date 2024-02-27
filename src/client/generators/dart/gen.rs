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

use std::borrow::Cow;
use tokio::fs;
use crate::client::generators::dart::pubspec::updated_pubspec_yaml_for_existing_project;

fn should_escape(name: &str) -> bool {
    name.starts_with("_") || ["is", "in", "AND", "OR", "NOT"].contains(&name)
}

fn type_is_not_dynamic(t: &str) -> bool {
    t != "dynamic"
}

fn type_is_dynamic(t: &str) -> bool {
    t == "dynamic"
}

fn value_for_data_transformer_dart<'a>(action_name: &str, model_name: &str) -> Cow<'a, str> {
    match action_name {
        "findUnique" | "findFirst" | "create" | "update" | "upsert" | "delete" | "signIn" | "identity" => Cow::Owned(format!("(p0) => {}.fromJson(p0)", model_name)),
        "findMany" | "createMany" | "updateMany" | "deleteMany" => Cow::Owned(format!("(p0) => p0.map<{}>((e) => {}.fromJson(e)).toList() as List<{}>", model_name, model_name, model_name)),
        _ => Cow::Borrowed("(p0) => p0"),
    }
}

fn value_for_meta_transformer_dart(action_name: &str) -> &'static str {
    match action_name {
        "findMany" | "createMany" | "updateMany" | "deleteMany" => "(p0) => PagingInfo.fromJson(p0)",
        "signIn" => "(p0) => TokenInfo.fromJson(p0)",
        _ => "null",
    }
}

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
        if let Some(pubspec_yaml) = generator.find_file_upwards("pubspec.yaml") {
            let yaml_data = std::fs::read_to_string(&pubspec_yaml).expect("Unable to read pubspec.yaml");
            let updated_json_data = updated_pubspec_yaml_for_existing_project(yaml_data);
            fs::write(pubspec_yaml, updated_json_data).await?;
        }
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
