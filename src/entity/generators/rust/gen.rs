use async_trait::async_trait;
use async_recursion::async_recursion;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use askama::Template;
use teo_parser::r#type::Type;
use teo_runtime::config::entity::Entity;
use teo_runtime::namespace::Namespace;
use teo_result::Result;
use teo_runtime::model::field::typed::Typed;
use teo_runtime::traits::documentable::Documentable;
use teo_runtime::traits::named::Named;
use teo_runtime::model::field::is_optional::IsOptional;
use std::str::FromStr;
use maplit::btreeset;
use tokio::fs;
use toml_edit::{Document, value};
use crate::entity::ctx::Ctx;
use crate::entity::generator::Generator;
use crate::entity::generators::rust;
use crate::utils::file::FileUtil;
use crate::utils::filters;
use crate::utils::lookup::Lookup;

fn format_model_path(path: Vec<&str>) -> String {
    "vec![".to_owned() + &path.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<String>>().join(", ") + "]"
}

fn generics_declaration(names: Vec<&str>) -> String {
    if names.is_empty() {
        "".to_owned()
    } else {
        "<".to_owned() + &names.join(", ") + ">"
    }
}

#[derive(Template)]
#[template(path = "entity/rust/mod.rs.jinja", escape = "none")]
pub(self) struct RustMainModTemplate<'a> {
    pub(self) namespace: &'a Namespace,
    pub(self) has_date: bool,
    pub(self) has_datetime: bool,
    pub(self) has_decimal: bool,
    pub(self) has_object_id: bool,
    pub(self) lookup: &'static dyn Lookup,
    pub(self) format_model_path: &'static dyn Fn(Vec<&str>) -> String,
    pub(self) generics_declaration: &'static dyn Fn(Vec<&str>) -> String,
}

unsafe impl Send for RustMainModTemplate<'_> { }
unsafe impl Sync for RustMainModTemplate<'_> { }

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
            lookup: &rust::lookup,
            format_model_path: &format_model_path,
            generics_declaration: &generics_declaration,
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

    async fn generate_module_file(&self, namespace: &Namespace, ctx: &Ctx<'_>, filename: impl AsRef<Path>, generator: &FileUtil) -> Result<()> {
        let template = RustMainModTemplate::new(namespace);
        generator.generate_file(filename.as_ref(), template.render().unwrap()).await?;
        Ok(())
    }

    #[async_recursion]
    async fn generate_module_for_namespace(&self, namespace: &Namespace, ctx: &Ctx<'_>, generator: &FileUtil) -> Result<()> {
        if namespace.is_std() {
            return Ok(());
        }
        if namespace.is_main() || !namespace.namespaces.is_empty() {
            // create dir and create mod.rs
            if !namespace.is_main() {
                generator.ensure_directory(namespace.path().join("/")).await?;
            }
            self.generate_module_file(
                namespace,
                ctx,
                PathBuf::from_str(&namespace.path().join("/")).unwrap().join("mod.rs"),
                generator
            ).await?;
        } else {
            // create file
            self.generate_module_file(
                namespace,
                ctx,
                PathBuf::from_str(&namespace.path().iter().rev().skip(1).rev().map(|s| *s).collect::<Vec<&str>>().join("/")).unwrap().join(namespace.path().last().unwrap().to_string() + ".rs"),
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
        // module files
        self.generate_module_for_namespace(ctx.main_namespace, ctx, generator).await?;
        // helpers
        generator.ensure_directory("helpers").await?;
        generator.generate_file("helpers/mod.rs", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/entity/rust/helpers/mod.rs.jinja"))).await?;
        generator.generate_file("helpers/enum.rs", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/entity/rust/helpers/enum.rs.jinja"))).await?;
        generator.generate_file("helpers/interface.rs", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/entity/rust/helpers/interface.rs.jinja"))).await?;
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


