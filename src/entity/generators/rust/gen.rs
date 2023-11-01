use async_trait::async_trait;
use async_recursion::async_recursion;
use std::collections::BTreeSet;
use askama::Template;
use teo_parser::r#type::Type;
use teo_runtime::config::entity::Entity;
use teo_runtime::namespace::Namespace;
use teo_result::Result;
use teo_runtime::model::field::typed::Typed;
use teo_runtime::traits::documentable::Documentable;
use teo_runtime::traits::named::Named;
use maplit::btreeset;
use tokio::fs;
use toml_edit::{Document, value};
use crate::entity::ctx::Ctx;
use crate::entity::generator::Generator;
use crate::utils::file::FileUtil;

#[derive(Template)]
#[template(path = "entity/rust/mod.rs.jinja", escape = "none")]
pub(self) struct RustMainModTemplate<'a> {
    pub(self) namespace: &'a Namespace,
    pub(self) has_date: bool,
    pub(self) has_datetime: bool,
    pub(self) has_decimal: bool,
    pub(self) has_object_id: bool,
}

impl<'a> RustMainModTemplate<'a> {

    fn new(namespace: &'a Namespace) -> Self {
        let mut has_date = false;
        let mut has_datetime = false;
        let mut has_decimal = false;
        let mut has_object_id = false;
        namespace.models.values().for_each(|c| c.fields.values().for_each(|f| {
            if f.r#type().test(&Type::Date) {
                has_date = true;
            } else if f.r#type().test(&Type::DateTime) {
                has_datetime = true;
            } else if f.r#type().test(&Type::Decimal) {
                has_decimal = true;
            } else if f.r#type().test(&Type::ObjectId) {
                has_object_id = true;
            }
        }));
        Self {
            namespace,
            has_date,
            has_datetime,
            has_decimal,
            has_object_id,
        }
    }
}

pub(in crate::entity) struct RustGenerator { }

impl RustGenerator {

    pub fn new() -> Self {
        Self {}
    }

    async fn find_and_update_cargo_toml(&self, package_requirements: &BTreeSet<&str>, generator: &FileUtil) -> Result<()> {
        let cargo_toml = match generator.find_file_upwards("Cargo.toml") {
            Some(path) => path,
            None => return Ok(()),
        };
        let toml = fs::read_to_string(&cargo_toml).await.unwrap();
        let mut doc = toml.parse::<Document>().expect("`Cargo.toml' has invalid content");
        let deps = doc.get_mut("dependencies").unwrap();
        if package_requirements.contains(&"chrono") {
            if deps.get("chrono").is_none() {
                deps["chrono"]["version"] = value("0.4.23");
            }
        }
        if package_requirements.contains(&"bson") {
            if deps.get("bson").is_none() {
                deps["bson"]["version"] = value("2.3.0");
            }
        }
        if package_requirements.contains(&"bigdecimal") {
            if deps.get("bigdecimal").is_none() {
                deps["bigdecimal"]["version"] = value("0.3.0");
            }
        }
        fs::write(cargo_toml, doc.to_string()).await?;
        Ok(())
    }

    async fn generate_module_file(&self, namespace: &Namespace, ctx: &Ctx<'_>, filename: impl AsRef<str>, generator: &FileUtil) -> Result<()> {
        let template = RustMainModTemplate::new(namespace);
        generator.generate_file(filename.as_ref(), template.render().unwrap()).await?;
        Ok(())
    }

    #[async_recursion]
    async fn generate_module_for_namespace(&self, namespace: &Namespace, ctx: &Ctx<'_>, generator: &FileUtil) -> Result<()> {
        if namespace.is_main() || !namespace.namespaces.is_empty() {
            // create dir and create mod.rs
            if !namespace.is_main() {
                generator.ensure_directory(namespace.path().join("/")).await?;
            }
            self.generate_module_file(
                namespace,
                ctx,
                namespace.path().join("/") + "/mod.rs",
                generator
            ).await?;
        } else {
            // create file
            self.generate_module_file(
                namespace,
                ctx,
                namespace.path().iter().rev().skip(1).rev().map(|s| *s).collect::<Vec<&str>>().join("/") + "/" + *namespace.path().last().unwrap(),
                generator
            ).await?;
        }
        for namespace in namespace.namespaces.values() {
            self.generate_module_for_namespace(namespace, ctx, generator).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Generator for RustGenerator {

    async fn generate_entity_files(&self, ctx: &Ctx, generator: &FileUtil) -> Result<()> {
        self.generate_module_for_namespace(ctx.main_namespace, ctx, generator).await?;
        // Modify files
        // let mut package_requirements = btreeset![];
        // if template.has_date || template.has_datetime {
        //     package_requirements.insert("chrono");
        // }
        // if template.has_decimal {
        //     package_requirements.insert("bigdecimal");
        // }
        // if template.has_object_id {
        //     package_requirements.insert("bson");
        // }
        // if !package_requirements.is_empty() {
        //     self.find_and_update_cargo_toml(&package_requirements, generator).await;
        // }
        Ok(())
    }
}


