use askama::Template;
use async_trait::async_trait;
use teo_parser::r#type::Type;
use teo_runtime::config::client::Client;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::named::Named;
use crate::client::ctx::Ctx;
use crate::client::generator::Generator;
use crate::outline::outline::{Mode, Outline};
use crate::utils::exts::ClientExt;
use crate::utils::file::FileUtil;
use crate::utils::filters;
use crate::utils::lookup::Lookup;
use crate::client::generators::swift::lookup;

#[derive(Template)]
#[template(path = "client/swift/readme.md.jinja", escape = "none")]
pub(self) struct SwiftReadMeTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/swift/package.swift.jinja", escape = "none")]
pub(self) struct SwiftPackageSwiftTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/swift/namespace.swift.jinja", escape = "none")]
pub(self) struct SwiftNamespaceTemplate<'a> {
    pub(self) main_namespace: &'a Namespace,
    pub(self) namespace: &'a Namespace,
    pub(self) outline: &'a Outline,
    pub(self) conf: &'a Client,
    pub(self) lookup: &'static dyn Lookup,
    pub(crate) render_namespace: &'static dyn Fn(&Namespace, &Client, &Namespace) -> String,
}

#[derive(Template)]
#[template(path = "client/swift/teo.swift.jinja", escape = "none")]
pub(self) struct SwiftMainTemplate<'a> {
    pub(self) namespace: &'a Namespace,
    pub(self) outline: &'a Outline,
    pub(self) conf: &'a Client,
    pub(self) lookup: &'static dyn Lookup,
    pub(crate) render_namespace: &'static dyn Fn(&Namespace, &Client, &Namespace) -> String,
}

unsafe impl Send for SwiftMainTemplate<'_> { }
unsafe impl Sync for SwiftMainTemplate<'_> { }
unsafe impl Send for SwiftNamespaceTemplate<'_> { }
unsafe impl Sync for SwiftNamespaceTemplate<'_> { }

pub(crate) fn render_namespace(namespace: &Namespace, conf: &Client, main_namespace: &Namespace) -> String {
    let content = SwiftNamespaceTemplate {
        conf,
        namespace,
        render_namespace: &render_namespace,
        outline: &Outline::new(namespace, Mode::Client, main_namespace, true),
        lookup: &lookup,
        main_namespace,
    }.render().unwrap();
    if namespace.path.is_empty() {
        content
    } else {
        format!("class {} {{\n", namespace.name()) + &indent::indent_by(4, content.as_str()) + "\n}"
    }
}

pub(in crate::client) struct SwiftGenerator { }

impl SwiftGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Generator for SwiftGenerator {

    fn module_directory_in_package(&self, conf: &Client) -> String {
        return format!("Sources/{}", conf.inferred_package_name_camel_case())
    }

    async fn generate_module_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.clear_root_directory().await?;
        Ok(())
    }

    async fn generate_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.ensure_root_directory().await?;
        generator.generate_file(".gitignore", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/client/swift/gitignore"))).await?;
        generator.generate_file("README.md", SwiftReadMeTemplate { conf: ctx.conf }.render().unwrap()).await?;
        generator.generate_file("Package.swift", SwiftPackageSwiftTemplate { conf: ctx.conf }.render().unwrap()).await?;
        Ok(())
    }

    async fn update_parent_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        todo!()
    }

    async fn generate_main(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        let outline = Outline::new(ctx.main_namespace, Mode::Client, ctx.main_namespace, true);
        generator.generate_file(format!("{}.swift", ctx.conf.inferred_package_name_camel_case()), SwiftMainTemplate {
            lookup: &lookup::lookup,
            outline: &outline,
            conf: ctx.conf,
            namespace: ctx.main_namespace,
            render_namespace: &render_namespace,
        }.render().unwrap()).await?;
        Ok(())
    }
}
