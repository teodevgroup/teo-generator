use askama::Template;
use async_trait::async_trait;
use teo_runtime::config::client::Client;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::named::Named;
use crate::client::ctx::Ctx;
use crate::client::generator::Generator;
use crate::client::generators::ts;
use crate::client::generators::ts::package_json::{generate_package_json, update_package_json};
use crate::outline::outline::{Mode, Outline};
use crate::utils::file::FileUtil;
use crate::utils::lookup::Lookup;
use crate::utils::filters;
use crate::utils::exts::ClientExt;
use indent;
use teo_parser::r#type::Type;
use crate::client::generators::ts::lookup;

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
}

#[derive(Template)]
#[template(path = "client/ts/index.d.ts.jinja", escape = "none")]
pub(self) struct TsIndexDTsTemplate<'a> {
    pub(self) main_namespace: &'a Namespace,
    pub(self) conf: &'a Client,
    pub(self) render_namespace: &'static dyn Fn(&Namespace) -> String,
}

unsafe impl Send for TsIndexDTsTemplate<'_> { }
unsafe impl Sync for TsIndexDTsTemplate<'_> { }

#[derive(Template)]
#[template(path = "client/ts/namespace.partial.jinja", escape = "none")]
pub(self) struct TsNamespaceTemplate<'a> {
    pub(self) namespace: &'a Namespace,
    pub(self) render_namespace: &'static dyn Fn(&Namespace) -> String,
    pub(self) outline: &'a Outline,
    pub(self) lookup: &'static dyn Lookup,
    pub(self) get_payload_suffix: &'static dyn Fn(&Type) -> &'static str,
    pub(self) ts_extends: &'static dyn Fn(&Vec<Type>) -> String,
}

unsafe impl Send for TsNamespaceTemplate<'_> { }
unsafe impl Sync for TsNamespaceTemplate<'_> { }

fn ts_extends(extends: &Vec<Type>) -> String {
    if extends.is_empty() {
        "".to_owned()
    } else {
        extends.iter().map(|extend| lookup(extend).unwrap() + " & ").collect::<Vec<String>>().join("")
    }
}

fn get_payload_suffix(t: &Type) -> &'static str {
    if t.is_array() {
        "[]"
    } else if t.is_optional() {
        "?"
    } else {
        ""
    }
}

pub(self) fn render_namespace(namespace: &Namespace) -> String {
    let content = TsNamespaceTemplate {
        namespace,
        render_namespace: &render_namespace,
        outline: &Outline::new(namespace, Mode::Client),
        lookup: &ts::lookup,
        get_payload_suffix: &get_payload_suffix,
        ts_extends: &ts_extends,
    }.render().unwrap();
    if namespace.path.is_empty() {
        content
    } else {
        format!("export namespace {} {{\n", namespace.name()) + &indent::indent_by(4, content.as_str()) + "\n}"
    }
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
        generator.generate_file("README.md", TsReadMeTemplate { conf: ctx.conf }.render().unwrap()).await?;
        if generator.generate_file_if_not_exist("package.json", generate_package_json(generator.get_base_dir())).await? {
            // if exist, update package.json with a minor version
            let json_data = std::fs::read_to_string(generator.get_file_path("package.json"))
                .expect("Unable to read package.json");
            generator.generate_file("package.json", update_package_json(json_data)).await?;
        }
        Ok(())
    }

    async fn generate_main(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.generate_file("index.d.ts", TsIndexDTsTemplate {
            main_namespace: ctx.main_namespace,
            conf: ctx.conf,
            render_namespace: &render_namespace,
        }.render().unwrap()).await?;
        generator.generate_file("index.js", TsIndexJsTemplate {
            main_namespace: ctx.main_namespace,
            conf: ctx.conf,
        }.render().unwrap()).await?;
        Ok(())
    }
}


