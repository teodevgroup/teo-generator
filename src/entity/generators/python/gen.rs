use std::path::{Path, PathBuf};
use askama::Template;
use async_recursion::async_recursion;
use async_trait::async_trait;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use crate::entity::ctx::Ctx;
use crate::entity::generator::Generator;
use crate::outline::outline::{Mode, Outline};
use crate::utils::file::FileUtil;
use std::str::FromStr;
use inflector::Inflector;

#[derive(Template)]
#[template(path = "entity/python/__init__.py.jinja", escape = "none")]
pub(self) struct PythonModuleTemplate<'a> {
    pub(self) namespace: &'a Namespace,
    pub(self) outline: Outline,
}

impl<'a> PythonModuleTemplate<'a> {

    fn new(namespace: &'a Namespace, main_namespace: &'a Namespace) -> Self {
        Self { namespace, outline: Outline::new(namespace, Mode::Entity, main_namespace), }
    }
}

pub(crate) struct PythonGenerator { }

impl PythonGenerator {

    pub fn new() -> Self {
        Self { }
    }

    async fn generate_module_file(&self, namespace: &Namespace, filename: impl AsRef<Path>, generator: &FileUtil, main_namespace: &Namespace) -> teo_result::Result<()> {
        let template = PythonModuleTemplate::new(namespace, main_namespace);
        generator.generate_file(filename.as_ref(), template.render().unwrap()).await?;
        Ok(())
    }

    #[async_recursion]
    async fn generate_module_for_namespace(&self, namespace: &Namespace, generator: &FileUtil, main_namespace: &Namespace) -> teo_result::Result<()> {
        if namespace.is_main() || !namespace.namespaces.is_empty() {
            // create dir and create mod.rs
            if !namespace.is_main() {
                generator.ensure_directory(namespace.path().iter().map(|s| s.to_snake_case()).join("/")).await?;
            }
            self.generate_module_file(
                namespace,
                PathBuf::from_str(&namespace.path().iter().map(|s| s.to_snake_case()).join("/")).unwrap().join("__init__.pyi"),
                generator,
                main_namespace
            ).await?;
        } else {
            // create file
            self.generate_module_file(
                namespace,
                PathBuf::from_str(&namespace.path().iter().rev().skip(1).rev().map(|s| s.to_snake_case()).collect::<Vec<String>>().join("/")).unwrap().join(namespace.path().last().unwrap().to_snake_case() + ".pyi"),
                generator,
                main_namespace,
            ).await?;
        }
        for namespace in namespace.namespaces.values() {
            self.generate_module_for_namespace(namespace, generator, main_namespace).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Generator for PythonGenerator {

    async fn generate_entity_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        // module files
        self.generate_module_for_namespace(ctx.main_namespace, generator, ctx.main_namespace).await?;
        Ok(())
    }
}


