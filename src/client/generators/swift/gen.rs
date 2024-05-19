use askama::Template;
use async_trait::async_trait;
use teo_runtime::config::client::Client;
use teo_runtime::namespace::Namespace;
use tokio::fs;

use crate::client::ctx::Ctx;
use crate::utils::exts::ClientExt;
use crate::client::generator::Generator;
use crate::client::swift::package_swift::updated_package_swift_for_existing_project;

use crate::outline::outline::Outline;
use crate::utils::file::FileUtil;


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
#[template(path = "client/swift/package.swift.jinja", escape = "none")]
pub(self) struct SwiftMainTemplate<'a> {
    pub(self) namespace: &'a Namespace,
    pub(self) outline: &'a Outline,
    pub(self) conf: &'a Client,
}


pub(in crate::client) struct SwiftGenerator {}

impl SwiftGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Generator for SwiftGenerator {

    fn module_directory_in_package(&self, conf: &Client) -> String {
        return format!("Sources/{}", conf.inferred_package_name())
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
        if let Some(package_swift) = generator.find_file_upwards("Package.swift") {
            let package_swift_data = std::fs::read_to_string(&package_swift).expect("Unable to read Package.swift");
            let updated_data = updated_package_swift_for_existing_project(package_swift_data);
            fs::write(package_swift, updated_data).await?;
        }
        Ok(())
    }

    async fn generate_main(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        // generator.generate_file("Teo.swift", SwiftMainTemplate {
        //     namespace: ctx.main_namespace,
        //     outline: &ctx.outline,
        //     conf: ctx.conf,
        // }.render().unwrap()).await?;
        // Ok(())
        todo!()
    }
}
