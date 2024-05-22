use std::env;
use crate::utils::filters;
use std::process::Command;
use askama::Template;
use async_trait::async_trait;
use teo_parser::r#type::Type;
use teo_runtime::config::client::Client;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::named::Named;
use crate::client::ctx::Ctx;
use crate::client::generator::Generator;
use crate::client::generators::kotlin::lookup;
use crate::outline::outline::{Mode, Outline};
use crate::utils::exts::ClientExt;
use crate::utils::file::FileUtil;
use crate::utils::lookup::Lookup;
use crate::utils::message::green_message;

fn package_name_from_ctx_conf(ctx: &Ctx) -> String {
    let mut slice: &str = ctx.conf.dest.as_str();
    for prefix in ["src/main/java", "src\\main\\java"] {
        if let Some(index) = slice.rfind(prefix) {
            slice = &slice[(index + 1 + prefix.len())..]
        }
    }
    slice.replace("/", ".").replace("\\", ".")
}

fn maybe_any_prefix(t: &Type) -> &'static str {
    let lookup_result = lookup(t).unwrap();
    return if lookup_result.matches("^Any\\??$").count() > 0 {
        "@Serializable(with=AnySerializer::class) "
    } else {
        ""
    }
}

fn maybe_underscore(name: &str) -> String {
    if name.starts_with("_") {
        format!("@SerialName(\"{}\") ", name)
    } else {
        "".to_owned()
    }
}

#[derive(Template)]
#[template(path = "client/kotlin/readme.md.jinja", escape = "none")]
pub(self) struct KotlinReadMeTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/kotlin/build.gradle.kts.jinja", escape = "none")]
pub(self) struct KotlinBuildGradleTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/kotlin/settings.gradle.kts.jinja", escape = "none")]
pub(self) struct KotlinSettingsGradleTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/kotlin/namespace.kt.jinja", escape = "none")]
pub(self) struct KotlinNamespaceTemplate<'a> {
    pub(self) main_namespace: &'a Namespace,
    pub(self) namespace: &'a Namespace,
    pub(self) outline: &'a Outline,
    pub(self) conf: &'a Client,
    pub(self) lookup: &'static dyn Lookup,
    pub(crate) render_namespace: &'static dyn Fn(&Namespace, &Client, &Namespace) -> String,
    pub(self) maybe_any_prefix: &'static dyn Fn(&Type) -> &'static str,
    pub(self) maybe_underscore: &'static dyn Fn(&str) -> String,
}

#[derive(Template)]
#[template(path = "client/kotlin/teo.kt.jinja", escape = "none")]
pub(self) struct KotlinMainTemplate<'a> {
    pub(self) package_name: String,
    pub(self) namespace: &'a Namespace,
    pub(self) outline: &'a Outline,
    pub(self) conf: &'a Client,
    pub(self) lookup: &'static dyn Lookup,
    pub(crate) render_namespace: &'static dyn Fn(&Namespace, &Client, &Namespace) -> String,
}

unsafe impl Send for KotlinMainTemplate<'_> { }
unsafe impl Sync for KotlinMainTemplate<'_> { }
unsafe impl Send for KotlinNamespaceTemplate<'_> { }
unsafe impl Sync for KotlinNamespaceTemplate<'_> { }

pub(crate) fn render_namespace(namespace: &Namespace, conf: &Client, main_namespace: &Namespace) -> String {
    let content = KotlinNamespaceTemplate {
        conf,
        namespace,
        render_namespace: &render_namespace,
        outline: &Outline::new(namespace, Mode::Client, main_namespace),
        lookup: &lookup,
        main_namespace,
        maybe_any_prefix: &maybe_any_prefix,
        maybe_underscore: &maybe_underscore,
    }.render().unwrap();
    if namespace.path.is_empty() {
        content
    } else {
        format!("class {} {{\n", namespace.name()) + &indent::indent_by(4, content.as_str()) + "\n}"
    }
}

pub(in crate::client) struct KotlinGenerator { }

impl KotlinGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Generator for KotlinGenerator {

    fn module_directory_in_package(&self, conf: &Client) -> String {
        "src/main/kotlin".to_owned()
    }

    async fn generate_module_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.clear_root_directory().await?;
        Ok(())
    }

    async fn generate_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.ensure_root_directory().await?;
        let base = generator.get_base_dir();
        let mut has_project = false;
        for file in ["build.gradle", "build.gradle.kts"] {
            let proj_file = base.join(file);
            if proj_file.exists() { has_project = true; break; }
        }
        if !has_project {
            let saved_cwd = env::current_dir().unwrap();
            env::set_current_dir(base).unwrap();
            green_message("run", format!("`gradle init --type basic --dsl kotlin --project-name {}`", ctx.conf.inferred_package_name_camel_case()));
            let exit_status = Command::new("gradle").arg("init").arg("--type").arg("basic").arg("--dsl").arg("kotlin").arg("--project-name").arg(ctx.conf.inferred_package_name_camel_case()).spawn()?.wait()?;
            if exit_status.success() {
                env::set_current_dir(saved_cwd).unwrap();
                generator.generate_file(".gitignore", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/client/kotlin/gitignore"))).await?;
                generator.generate_file("README.md", KotlinReadMeTemplate { conf: ctx.conf }.render().unwrap()).await?;
                generator.generate_file("build.gradle.kts", KotlinBuildGradleTemplate { conf: ctx.conf }.render().unwrap()).await?;
                generator.generate_file("settings.gradle.kts", KotlinSettingsGradleTemplate { conf: ctx.conf }.render().unwrap()).await?;
            }
        }
        Ok(())
    }

    async fn update_parent_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        Ok(())
    }

    async fn generate_main(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        let outline = Outline::new(ctx.main_namespace, Mode::Client, ctx.main_namespace);
        generator.generate_file(format!("{}.kt", ctx.conf.inferred_package_name_camel_case()), KotlinMainTemplate {
            package_name: package_name_from_ctx_conf(ctx),
            lookup: &lookup::lookup,
            outline: &outline,
            conf: ctx.conf,
            namespace: ctx.main_namespace,
            render_namespace: &render_namespace,
        }.render().unwrap()).await?;
        Ok(())
    }
}
