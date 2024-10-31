use askama::Template;
use async_trait::async_trait;
use teo_runtime::config::client::Client;
use teo_runtime::namespace::Namespace;
use crate::client::ctx::Ctx;
use crate::client::generator::Generator;
use crate::client::generators::ts::package_json::generate_package_json;
use crate::client::ts::package_json::updated_package_json_for_existing_project;
use crate::utils::file::FileUtil;
use crate::utils::filters;
use crate::utils::exts::ClientExt;
use indent;
use teo_runtime::handler::Handler;
use teo_runtime::request::Method;
use tokio::fs;
use crate::outline::outline::Mode;
use crate::shared::ts::conf::TsConf;
use crate::shared::ts::templates::{render_namespace, TsIndexDTsTemplate};
use crate::utils::update_package_json_version::update_package_json_version;

#[derive(Template)]
#[template(path = "client/ts/readme.md.jinja", escape = "none")]
pub(self) struct TsReadMeTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/ts/index.js.jinja", escape = "none")]
pub(self) struct TsIndexJsTemplate<'a> {
    pub(self) main_namespace: &'a Namespace,
    pub(self) conf: &'a Client,
    pub(self) group_delegate_map: &'static dyn Fn(&Namespace) -> String,
    pub(self) custom_handler_map: &'static dyn Fn(&Namespace) -> String,
}

unsafe impl Send for TsIndexJsTemplate<'_> { }
unsafe impl Sync for TsIndexJsTemplate<'_> { }

fn group_delegate_map(main_namespace: &Namespace) -> String {
    let mut entries = vec![];
    collect_namespace_paths(main_namespace, &mut entries);
    if entries.is_empty() { "[]".to_string() } else { "[\n".to_owned() + &entries.join(",\n") + "\n]" }
}

fn collect_namespace_paths(namespace: &Namespace, entries: &mut Vec<String>) {
    if !namespace.path().is_empty() {
        entries.push("    ".to_owned() + "\"" + &namespace.path().join(".") + "\"");
    }
    for model in namespace.models().values() {
        entries.push("    ".to_owned() + "\"" + &model.path().join(".") + "\"");
    }
    for handler_group in namespace.handler_groups().values() {
        entries.push("    ".to_owned() + "\"" + &handler_group.path().join(".") + "\"");
    }
    for namespace in namespace.namespaces().values() {
        collect_namespace_paths(namespace, entries);
    }
}

fn custom_handler_map(namespace: &Namespace) -> String {
    let mut entries = vec![];
    collect_namespace_custom_handlers(namespace, &mut entries);
    if entries.is_empty() { "{}".to_string() } else {
        "{\n".to_owned() + &entries.join(",\n") + "\n}"
    }
}

fn collect_namespace_custom_handlers(namespace: &Namespace, entries: &mut Vec<String>) {
    for handler in namespace.handlers().values() {
        add_handler_custom_entry(handler, entries)
    }
    for handler_group in namespace.model_handler_groups().values() {
        for handler in handler_group.handlers().values() {
            add_handler_custom_entry(handler, entries)
        }
    }
    for handler_group in namespace.handler_groups().values() {
        for handler in handler_group.handlers().values() {
            if handler.method() != Method::POST || handler.url().is_some() {
                add_handler_custom_entry(handler, entries)
            }
        }
    }
    for namespace in namespace.namespaces().values() {
        collect_namespace_custom_handlers(namespace, entries);
    }
}

fn add_handler_custom_entry(handler: &Handler, entries: &mut Vec<String>) {
    let custom_map = handler.has_custom_url_args();
    let method_name = handler.method().as_str();
    let url = if let Some(url) = handler.url() {
        url.clone()
    } else {
        handler.path().join("/")
    };
    entries.push("    \"".to_owned() + &handler.path().join(".") + "\":" + "{ method: \"" + method_name + "\", " + "path: \"" + url.as_str() + "\", pathArguments: " + &custom_map.to_string() + " }");

}

pub(in crate::client) struct TSGenerator {}

impl TSGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Generator for TSGenerator {

    fn module_directory_in_package(&self, conf: &Client) -> String {
        "src".to_owned()
    }

    async fn generate_module_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.clear_root_directory().await
    }

    async fn generate_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.ensure_root_directory().await?;
        generator.generate_file(".gitignore", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/client/ts/gitignore"))).await?;
        generator.generate_file("tsconfig.json", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/client/ts/tsconfig.json"))).await?;
        generator.generate_file("README.md", TsReadMeTemplate { conf: ctx.conf }.render().unwrap()).await?;
        if generator.generate_file_if_not_exist("package.json", generate_package_json(generator.get_base_dir())).await? {
            // if exists, update package.json with a minor version
            let json_data = std::fs::read_to_string(generator.get_file_path("package.json"))
                .expect("Unable to read package.json");
            generator.generate_file("package.json", update_package_json_version(json_data)).await?;
        }
        Ok(())
    }

    async fn update_parent_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        if let Some(package_json) = generator.find_file_upwards("package.json") {
            let json_data = std::fs::read_to_string(&package_json).expect("Unable to read package.json");
            let updated_json_data = updated_package_json_for_existing_project(json_data);
            fs::write(package_json, updated_json_data).await?;
        }
        Ok(())
    }

    async fn generate_main(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.generate_file("index.d.ts", TsIndexDTsTemplate {
            main_namespace: ctx.main_namespace,
            conf: &TsConf::new(ctx.conf.object_name.clone(), ctx.conf.class_name(), true),
            render_namespace: &render_namespace,
            mode: Mode::Client,
        }.render().unwrap()).await?;
        generator.generate_file("index.js", TsIndexJsTemplate {
            main_namespace: ctx.main_namespace,
            conf: ctx.conf,
            group_delegate_map: &group_delegate_map,
            custom_handler_map: &custom_handler_map,
        }.render().unwrap()).await?;
        Ok(())
    }
}
