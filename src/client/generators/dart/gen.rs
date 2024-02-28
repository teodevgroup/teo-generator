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
use async_recursion::async_recursion;
use teo_result::Result;
use regex::Regex;
use teo_runtime::namespace::Namespace;
use tokio::fs;
use std::borrow::Borrow;
use itertools::Itertools;
use crate::client::generators::dart::lookup;
use crate::client::generators::dart::pubspec::updated_pubspec_yaml_for_existing_project;
use crate::utils::lookup::Lookup;

fn import_dots(namespace: &Namespace) -> String {
    if namespace.path.len() <= 1 {
        "".to_owned()
    } else {
        "../".repeat(namespace.path().len() - 1)
    }
}

fn should_escape(name: &str) -> bool {
    name.starts_with("_") || ["is", "in", "AND", "OR", "NOT"].contains(&name)
}

fn type_is_not_dynamic(t: &str) -> bool {
    t != "dynamic"
}

fn type_is_dynamic(t: &str) -> bool {
    t == "dynamic"
}

fn value_for_data_transformer_dart(action_name: &str, model_name: &str) -> String {
    match action_name {
        "findUnique" | "findFirst" | "create" | "update" | "upsert" | "delete" | "signIn" | "identity" => format!("(p0) => {}.fromJson(p0)", model_name),
        "findMany" | "createMany" | "updateMany" | "deleteMany" => format!("(p0) => p0.map<{}>((e) => {}.fromJson(e)).toList() as List<{}>", model_name, model_name, model_name),
        _ => "(p0) => p0".to_owned(),
    }
}

fn from_json_parameters(names: &Vec<String>) -> String {
    names.iter().map(|n| format!(", {} Function(Object? json) fromJson{}", n, n)).join("")
}

fn from_json_arguments(names: &Vec<String>) -> String {
    names.iter().map(|n| format!(", fromJson{}", n)).join("")
}

fn to_json_parameters(names: &Vec<String>) -> String {
    names.iter().map(|n| format!(", Object Function({} value) toJson{}", n, n)).join("")
}

fn to_json_arguments(names: &Vec<String>) -> String {
    names.iter().map(|n| ", anyToJson".to_string()).join("")
    //names.iter().map(|n| format!(", toJson{}", n)).join("")
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
#[template(path = "client/dart/helper.dart.jinja", escape = "none")]
pub(self) struct DartHelperTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/dart/namespace.dart.jinja", escape = "none")]
pub(self) struct DartMainTemplate<'a> {
    pub(self) namespace: &'a Namespace,
    pub(self) outline: &'a Outline,
    pub(self) conf: &'a Client,
    pub(self) should_escape: &'static dyn Fn(&str) -> bool,
    pub(self) type_is_not_dynamic: &'static dyn Fn(&str) -> bool,
    pub(self) type_is_dynamic: &'static dyn Fn(&str) -> bool,
    pub(self) value_for_data_transformer_dart: &'static dyn Fn(&str, &str) -> String,
    pub(self) import_dots: &'static dyn Fn(&Namespace) -> String,
    pub(self) from_json_parameters: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) from_json_arguments: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) to_json_parameters: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) to_json_arguments: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) lookup: &'static dyn Lookup,
}

unsafe impl Send for DartMainTemplate<'_> { }
unsafe impl Sync for DartMainTemplate<'_> { }

pub(in crate::client) struct DartGenerator {}

impl DartGenerator {

    pub fn new() -> Self {
        Self {}
    }

    #[async_recursion]
    async fn generate_module_for_namespace(&self, namespace: &Namespace, generator: &FileUtil, main_namespace: &Namespace, conf: &Client) -> Result<()> {
        let outline = Outline::new(namespace, Mode::Client, main_namespace);
        generator.generate_file(if namespace.path().is_empty() {
            format!("{}.dart", conf.inferred_package_name_snake_case())
        } else {
            format!("{}.dart", namespace.path().join("/"))
        }, DartMainTemplate {
            namespace,
            outline: &outline,
            conf,
            should_escape: &should_escape,
            type_is_not_dynamic: &type_is_not_dynamic,
            type_is_dynamic: &type_is_dynamic,
            value_for_data_transformer_dart: &value_for_data_transformer_dart,
            import_dots: &import_dots,
            from_json_parameters: &from_json_parameters,
            from_json_arguments: &from_json_arguments,
            to_json_parameters: &to_json_parameters,
            to_json_arguments: &to_json_arguments,
            lookup: &lookup,
        }.render().unwrap()).await?;
        for child in namespace.namespaces.values() {
            self.generate_module_for_namespace(child, generator, main_namespace, conf).await?;
        }
        Ok(())
    }

    async fn generate_helper(&self, generator: &FileUtil, conf: &Client) -> Result<()> {
        generator.generate_file("_helper.dart", DartHelperTemplate { conf }.render().unwrap()).await?;
        Ok(())
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
        if generator.generate_file_if_not_exist("pubspec.yaml", DartPubspecTemplate { conf: ctx.conf }.render().unwrap()).await? {
            // if exists, update pubspec.yaml with a minor version
            let yaml_data = std::fs::read_to_string(generator.get_file_path("pubspec.yaml"))
                .expect("Unable to read pubspec.yaml");
            generator.generate_file("pubspec.yaml", update_pubspec_yaml_version(yaml_data)).await?;
        }
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

    async fn generate_main(&self, ctx: &Ctx, generator: &FileUtil) -> Result<()> {
        // module files
        self.generate_module_for_namespace(ctx.main_namespace, generator, ctx.main_namespace, ctx.conf).await?;
        self.generate_helper(generator, ctx.conf).await?;
        // run commands
        println!("debug error?: see base dir: {:?}", generator.get_base_dir());
        if let Some(pubspec_yaml) = generator.find_file_upwards("pubspec.yaml") {
            println!("debug error?: see pubspec yaml dir: {:?}", pubspec_yaml);
            let project_root = pubspec_yaml.parent().unwrap();
            std::env::set_current_dir(project_root).unwrap();
            green_message("run", "`dart pub get`".to_owned());
            Command::new("dart").arg("pub").arg("get").spawn()?.wait()?;
            green_message("run", "`dart run build_runner build --delete-conflicting-outputs`".to_owned());
            Command::new("dart").arg("run").arg("build_runner").arg("build").arg("--delete-conflicting-outputs").spawn()?.wait()?;
        }
        Ok(())
    }
}

fn update_pubspec_yaml_version(mut content: String) -> String {
    let regex = Regex::new("version\\s*:\\s*([0-9\\.\\+]+)").unwrap();
    if let Some(captures) = regex.captures(content.as_str()) {
        if let Some(capture) = captures.get(1) {
            let current_version = capture.as_str();
            content.replace_range(capture.range(), next_minor_version(current_version).as_str());
            content
        } else {
            content
        }
    } else {
        content
    }
}

fn next_minor_version(current: &str) -> String {
    let regex = Regex::new("([0-9\\.]+)(\\+[0-9]+)").unwrap();
    if let Some(caps) = regex.captures(current) {
        if let Some(version_number) = caps.get(1) {
            let version_number_str = version_number.as_str();
            let parts = version_number_str.split(".");
            let last = parts.clone().last().unwrap();
            match last.parse::<u32>() {
                Ok(num) => {
                    let new_last = format!("{}", num + 1);
                    let vec_parts: Vec<&str> = parts.into_iter().collect();
                    let new_version = vec_parts.split_last().unwrap().1.join(".") + "." + &new_last;
                    new_version + "+1"
                },
                Err(_) => current.to_owned(),
            }
        } else {
            current.to_owned()
        }
    } else {
        current.to_owned()
    }
}