use std::borrow::Borrow;
use async_trait::async_trait;
use async_recursion::async_recursion;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use askama::Template;
use teo_parser::r#type::Type;
use teo_runtime::namespace::Namespace;
use teo_result::Result;
use teo_runtime::model::field::typed::Typed;
use teo_runtime::traits::documentable::Documentable;
use teo_runtime::traits::named::Named;
use teo_runtime::model::field::is_optional::IsOptional;
use std::str::FromStr;
use maplit::btreeset;
use teo_parser::r#type::reference::Reference;
use teo_parser::r#type::synthesized_enum_reference::SynthesizedEnumReference;
use teo_parser::r#type::synthesized_shape_reference::SynthesizedShapeReference;
use tokio::fs;
use toml_edit::{Document, value};
use crate::entity::ctx::Ctx;
use crate::entity::generator::Generator;
use crate::entity::generators::rust;
use crate::entity::outline::outline::{Mode, Outline};
use crate::utils::file::FileUtil;
use crate::utils::filters;
use crate::utils::lookup::Lookup;

fn format_model_path(path: Vec<&str>) -> String {
    "vec![".to_owned() + &path.iter().map(|p| format!("\"{}\"", p)).collect::<Vec<String>>().join(", ") + "]"
}

fn super_keywords(path: Vec<&str>) -> String {
    if path.is_empty() {
        "".to_owned()
    } else {
        path.iter().map(|_| "super").collect::<Vec<&str>>().join("::") + "::"
    }
}

fn fix_stdlib(name: &str) -> &str {
    if name == "std" {
        "stdlib"
    } else {
        name
    }
}

fn fix_path_inner(components: &Vec<String>, namespace: &Namespace) -> Vec<String> {
    let mut results = vec![];
    let namespace_path = namespace.path();
    let mut left = namespace_path.len();
    for (index, component) in components.iter().enumerate() {
        if let Some(ns_component) = namespace_path.get(index) {
            if component == *ns_component {
                left -= 1;
            } else {
                results.push(fix_stdlib(component).to_owned());
            }
        } else {
            results.push(fix_stdlib(component).to_owned());
        }
    }
    for _ in 0..left {
        results.insert(0, "super".to_owned());
    }
    results
}

fn fix_path_enum_reference(enum_reference: &SynthesizedEnumReference, namespace: &Namespace) -> SynthesizedEnumReference {
    SynthesizedEnumReference {
        kind: enum_reference.kind,
        owner: Box::new(fix_path(enum_reference.owner.as_ref(), namespace)),
    }
}

fn fix_path_shape_reference(shape_reference: &SynthesizedShapeReference, namespace: &Namespace) -> SynthesizedShapeReference {
    SynthesizedShapeReference {
        kind: shape_reference.kind,
        owner: Box::new(fix_path(shape_reference.owner.as_ref(), namespace)),
        without: shape_reference.without.clone(),
    }
}

fn fix_path(t: &Type, namespace: &Namespace) -> Type {
    match t {
        Type::Undetermined => t.clone(),
        Type::Ignored => t.clone(),
        Type::Any => t.clone(),
        Type::Null => t.clone(),
        Type::Bool => t.clone(),
        Type::Int => t.clone(),
        Type::Int64 => t.clone(),
        Type::Float32 => t.clone(),
        Type::Float => t.clone(),
        Type::Decimal => t.clone(),
        Type::String => t.clone(),
        Type::ObjectId => t.clone(),
        Type::Date => t.clone(),
        Type::DateTime => t.clone(),
        Type::File => t.clone(),
        Type::Regex => t.clone(),
        Type::Model => t.clone(),
        Type::DataSet => t.clone(),
        Type::Enumerable(inner) => Type::Enumerable(Box::new(fix_path(inner.as_ref(), namespace))),
        Type::Array(inner) => Type::Array(Box::new(fix_path(inner.as_ref(), namespace))),
        Type::Dictionary(inner) => Type::Dictionary(Box::new(fix_path(inner.as_ref(), namespace))),
        Type::Tuple(types) => Type::Tuple(types.iter().map(|t| fix_path(t, namespace)).collect()),
        Type::Range(inner) => Type::Range(Box::new(fix_path(inner.as_ref(), namespace))),
        Type::Union(types) => Type::Union(types.iter().map(|t| fix_path(t, namespace)).collect()),
        Type::EnumVariant(reference) => Type::EnumVariant(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace))),
        Type::InterfaceObject(reference, types) => Type::InterfaceObject(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace)), types.iter().map(|t| fix_path(t, namespace)).collect()),
        Type::ModelObject(reference) => Type::ModelObject(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace))),
        Type::StructObject(reference, types) => Type::StructObject(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace)), types.iter().map(|t| fix_path(t, namespace)).collect()),
        Type::GenericItem(name) => Type::GenericItem(name.clone()),
        Type::Keyword(keyword) => Type::Keyword(keyword.clone()),
        Type::Optional(inner) => Type::Optional(Box::new(fix_path(inner.as_ref(), namespace))),
        Type::SynthesizedShapeReference(shape_reference) => Type::SynthesizedShapeReference(fix_path_shape_reference(shape_reference, namespace)),
        Type::SynthesizedEnumReference(enum_reference) => Type::SynthesizedEnumReference(fix_path_enum_reference(enum_reference, namespace)),
        _ => panic!(),
    }
}

fn generics_declaration(names: &Vec<String>, suffix: &str) -> String {
    if suffix.is_empty() {
        if names.is_empty() {
            "".to_owned()
        } else {
            "<".to_owned() + &names.join(", ") + ">"
        }
    } else {
        if names.is_empty() {
            "<'a>".to_owned()
        } else {
            "<'a, ".to_owned() + &names.join(", ") + ">"
        }
    }
}

fn phantom_generics(names: &Vec<String>) -> String {
    if names.is_empty() {
        "<()>".to_owned()
    } else if names.len() == 1 {
        "<".to_owned() + names.first().unwrap() + ">"
    } else {
        "<(".to_owned() + &names.join(", ") + ")>"
    }
}

fn unwrap_extend(extend: &Type, namespace: &Namespace, interface_suffix: &str) -> Result<String> {
    let interface_path = (fix_path_inner(extend.as_interface_object().unwrap().0.string_path(), namespace)).join("::");
    let a = extend.as_interface_object().unwrap().1;
    Ok(if a.is_empty() {
        interface_path + interface_suffix + "Trait"
    } else {
        interface_path + interface_suffix + "Trait" + "<" + if !interface_suffix.is_empty() { "'a, " } else { "" } + &a.iter().map(|e| {
            if e.is_interface_object() {
                unwrap_extend(e, namespace, interface_suffix)
            } else {
                Ok(rust::lookup(e)?)
            }
        }).collect::<Result<Vec<String>>>()?.join(", ") + ">"
    })
}

fn unwrap_extends(extends: &Vec<Type>, namespace: &Namespace, interface_suffix: &str) -> Result<Vec<String>> {
    Ok(extends.iter().map(|extend| {
        unwrap_extend(extend, namespace, interface_suffix)
    }).collect::<Result<Vec<String>>>()?)
}

#[derive(Template)]
#[template(path = "entity/rust/mod.rs.jinja", escape = "none")]
pub(self) struct RustModuleTemplate<'a> {
    pub(self) namespace: &'a Namespace,
    pub(self) outline: Outline,
    pub(self) has_date: bool,
    pub(self) has_datetime: bool,
    pub(self) has_decimal: bool,
    pub(self) has_object_id: bool,
    pub(self) lookup: &'static dyn Lookup,
    pub(self) lookup_ref: &'static dyn Lookup,
    pub(self) lookup_ref_mut: &'static dyn Lookup,
    pub(self) format_model_path: &'static dyn Fn(Vec<&str>) -> String,
    pub(self) generics_declaration: &'static dyn Fn(&Vec<String>, &str) -> String,
    pub(self) phantom_generics: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) unwrap_extends: &'static dyn Fn(&Vec<Type>, &Namespace, &str) -> Result<Vec<String>>,
    pub(self) super_keywords: &'static dyn Fn(Vec<&str>) -> String,
    pub(self) fix_path: &'static dyn Fn(&Type, &Namespace) -> Type,
    pub(self) interface_suffixes: Vec<&'static str>,
    pub(self) suffix_is_ref: &'static dyn Fn(&str) -> bool,
    pub(self) suffix_is_ref_mut: &'static dyn Fn(&str) -> bool,
    pub(self) suffix_is_none: &'static dyn Fn(&str) -> bool,
    pub(self) value_for_suffix: &'static dyn Fn(&str) -> &'static str,
}

fn value_for_suffix(suffix: &str) -> &'static str {
    match suffix {
        "" => "Value",
        "Ref" => "&'a Value",
        "RefMut" => "&'a mut Value",
        _ => unreachable!(),
    }
}

fn suffix_is_ref(suffix: &str) -> bool {
    suffix == "Ref"
}

fn suffix_is_ref_mut(suffix: &str) -> bool {
    suffix == "RefMut"
}

fn suffix_is_none(suffix: &str) -> bool {
    suffix == ""
}

unsafe impl Send for RustModuleTemplate<'_> { }
unsafe impl Sync for RustModuleTemplate<'_> { }

impl<'a> RustModuleTemplate<'a> {

    fn new(namespace: &'a Namespace) -> Self {
        let mut has_date = false;
        let mut has_datetime = false;
        let mut has_decimal = false;
        let mut has_object_id = false;
        if !namespace.is_std() {
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
            namespace.interfaces.values().for_each(|c| c.fields.values().for_each(|f| {
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
        } else {
            has_date = true;
            has_datetime = true;
            has_decimal = true;
            has_object_id = true;
        }
        Self {
            namespace,
            outline: Outline::new(namespace, Mode::Entity),
            has_date,
            has_datetime,
            has_decimal,
            has_object_id,
            lookup: &rust::lookup,
            lookup_ref: &rust::lookup_ref,
            lookup_ref_mut: &rust::lookup_ref_mut,
            format_model_path: &format_model_path,
            generics_declaration: &generics_declaration,
            phantom_generics: &phantom_generics,
            unwrap_extends: &unwrap_extends,
            super_keywords: &super_keywords,
            fix_path: &fix_path,
            interface_suffixes: vec!["", "Ref", "RefMut"],
            suffix_is_ref: &suffix_is_ref,
            suffix_is_ref_mut: &suffix_is_ref_mut,
            suffix_is_none: &suffix_is_none,
            value_for_suffix: &value_for_suffix,
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
                deps["chrono"]["version"] = value("0.4.31");
            }
        }
        if package_requirements.contains(&"bson") {
            if deps.get("bson").is_none() {
                deps["bson"]["version"] = value("2.7.0");
            }
        }
        if package_requirements.contains(&"bigdecimal") {
            if deps.get("bigdecimal").is_none() {
                deps["bigdecimal"]["version"] = value("=0.3.1");
            }
        }
        fs::write(cargo_toml, doc.to_string()).await?;
        Ok(())
    }

    async fn generate_module_file(&self, namespace: &Namespace, filename: impl AsRef<Path>, generator: &FileUtil) -> Result<()> {
        let template = RustModuleTemplate::new(namespace);
        generator.generate_file(filename.as_ref(), template.render().unwrap()).await?;
        Ok(())
    }

    #[async_recursion]
    async fn generate_module_for_namespace(&self, namespace: &Namespace, generator: &FileUtil) -> Result<()> {
        if namespace.is_main() || !namespace.namespaces.is_empty() {
            // create dir and create mod.rs
            if !namespace.is_main() {
                generator.ensure_directory(namespace.path().join("/")).await?;
            }
            self.generate_module_file(
                namespace,
                PathBuf::from_str(&namespace.path().join("/")).unwrap().join("mod.rs"),
                generator
            ).await?;
        } else {
            // create file
            self.generate_module_file(
                namespace,
                PathBuf::from_str(&namespace.path().iter().rev().skip(1).rev().map(|s| *s).collect::<Vec<&str>>().join("/")).unwrap().join(fix_stdlib(namespace.path().last().unwrap()).to_string() + ".rs"),
                generator
            ).await?;
        }
        for namespace in namespace.namespaces.values() {
            self.generate_module_for_namespace(namespace, generator).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Generator for RustGenerator {

    async fn generate_entity_files(&self, ctx: &Ctx, generator: &FileUtil) -> Result<()> {
        // module files
        self.generate_module_for_namespace(ctx.main_namespace, generator).await?;
        // helpers
        generator.ensure_directory("helpers").await?;
        generator.generate_file("helpers/mod.rs", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/entity/rust/helpers/mod.rs.jinja"))).await?;
        generator.generate_file("helpers/enum.rs", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/entity/rust/helpers/enum.rs.jinja"))).await?;
        generator.generate_file("helpers/interface.rs", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/entity/rust/helpers/interface.rs.jinja"))).await?;
        // Modify files
        let mut package_requirements = btreeset![];
        package_requirements.insert("chrono");
        package_requirements.insert("bigdecimal");
        package_requirements.insert("bson");
        self.find_and_update_cargo_toml(&package_requirements, generator).await?;
        Ok(())
    }
}


