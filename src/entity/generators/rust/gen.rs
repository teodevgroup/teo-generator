use std::borrow::Borrow;
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
use teo_parser::r#type::shape_reference::ShapeReference;
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

fn fix_path_shape_reference(shape_reference: &ShapeReference, namespace: &Namespace) -> ShapeReference {
    match shape_reference {
        ShapeReference::EnumFilter(t) => ShapeReference::EnumFilter(Box::new(fix_path(t.as_ref(), namespace))),
        ShapeReference::EnumNullableFilter(t) => ShapeReference::EnumNullableFilter(Box::new(fix_path(t.as_ref(), namespace))),
        ShapeReference::ArrayFilter(t) => ShapeReference::ArrayFilter(Box::new(fix_path(t.as_ref(), namespace))),
        ShapeReference::ArrayNullableFilter(t) => ShapeReference::ArrayNullableFilter(Box::new(fix_path(t.as_ref(), namespace))),
        ShapeReference::EnumWithAggregatesFilter(t) => ShapeReference::EnumWithAggregatesFilter(Box::new(fix_path(t.as_ref(), namespace))),
        ShapeReference::EnumNullableWithAggregatesFilter(t) => ShapeReference::EnumNullableWithAggregatesFilter(Box::new(fix_path(t.as_ref(), namespace))),
        ShapeReference::ArrayWithAggregatesFilter(t) => ShapeReference::ArrayWithAggregatesFilter(Box::new(fix_path(t.as_ref(), namespace))),
        ShapeReference::ArrayNullableWithAggregatesFilter(t) => ShapeReference::ArrayNullableWithAggregatesFilter(Box::new(fix_path(t.as_ref(), namespace))),
        ShapeReference::ArrayAtomicUpdateOperationInput(t) => ShapeReference::ArrayAtomicUpdateOperationInput(Box::new(fix_path(t.as_ref(), namespace))),
        ShapeReference::Args(a, path) => ShapeReference::Args(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::FindManyArgs(a, path) => ShapeReference::FindManyArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::FindFirstArgs(a, path) => ShapeReference::FindFirstArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::FindUniqueArgs(a, path) => ShapeReference::FindUniqueArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CreateArgs(a, path) => ShapeReference::CreateArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::UpdateArgs(a, path) => ShapeReference::UpdateArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::UpsertArgs(a, path) => ShapeReference::UpsertArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CopyArgs(a, path) => ShapeReference::CopyArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::DeleteArgs(a, path) => ShapeReference::DeleteArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CreateManyArgs(a, path) => ShapeReference::CreateManyArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::UpdateManyArgs(a, path) => ShapeReference::UpdateManyArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CopyManyArgs(a, path) => ShapeReference::CopyManyArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::DeleteManyArgs(a, path) => ShapeReference::DeleteManyArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CountArgs(a, path) => ShapeReference::CountArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::AggregateArgs(a, path) => ShapeReference::AggregateArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::GroupByArgs(a, path) => ShapeReference::GroupByArgs(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::RelationFilter(a, path) => ShapeReference::RelationFilter(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::ListRelationFilter(a, path) => ShapeReference::ListRelationFilter(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::WhereInput(a, path) => ShapeReference::WhereInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::WhereUniqueInput(a, path) => ShapeReference::WhereUniqueInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::ScalarFieldEnum(a, path) => ShapeReference::ScalarFieldEnum(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::ScalarWhereWithAggregatesInput(a, path) => ShapeReference::ScalarWhereWithAggregatesInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CountAggregateInputType(a, path) => ShapeReference::CountAggregateInputType(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::SumAggregateInputType(a, path) => ShapeReference::SumAggregateInputType(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::AvgAggregateInputType(a, path) => ShapeReference::AvgAggregateInputType(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::MaxAggregateInputType(a, path) => ShapeReference::MaxAggregateInputType(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::MinAggregateInputType(a, path) => ShapeReference::MinAggregateInputType(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CreateInput(a, path) => ShapeReference::CreateInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CreateInputWithout(a, path, without) => ShapeReference::CreateInputWithout(a.clone(), fix_path_inner(path, namespace), without.clone()),
        ShapeReference::CreateNestedOneInput(a, path) => ShapeReference::CreateNestedOneInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CreateNestedOneInputWithout(a, path, without) => ShapeReference::CreateNestedOneInputWithout(a.clone(), fix_path_inner(path, namespace), without.clone()),
        ShapeReference::CreateNestedManyInput(a, path) => ShapeReference::CreateNestedManyInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CreateNestedManyInputWithout(a, path, without) => ShapeReference::CreateNestedManyInputWithout(a.clone(), fix_path_inner(path, namespace), without.clone()),
        ShapeReference::UpdateInput(a, path) => ShapeReference::UpdateInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::UpdateInputWithout(a, path, without) => ShapeReference::UpdateInputWithout(a.clone(), fix_path_inner(path, namespace), without.clone()),
        ShapeReference::UpdateNestedOneInput(a, path) => ShapeReference::UpdateNestedOneInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::UpdateNestedOneInputWithout(a, path, without) => ShapeReference::UpdateNestedOneInputWithout(a.clone(), fix_path_inner(path, namespace), without.clone()),
        ShapeReference::UpdateNestedManyInput(a, path) => ShapeReference::UpdateNestedManyInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::UpdateNestedManyInputWithout(a, path, without) => ShapeReference::UpdateNestedManyInputWithout(a.clone(), fix_path_inner(path, namespace), without.clone()),
        ShapeReference::ConnectOrCreateInput(a, path) => ShapeReference::ConnectOrCreateInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::ConnectOrCreateInputWithout(a, path, without) => ShapeReference::ConnectOrCreateInputWithout(a.clone(), fix_path_inner(path, namespace), without.clone()),
        ShapeReference::UpdateWithWhereUniqueInput(a, path) => ShapeReference::UpdateWithWhereUniqueInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::UpdateWithWhereUniqueInputWithout(a, path, without) => ShapeReference::UpdateWithWhereUniqueInputWithout(a.clone(), fix_path_inner(path, namespace), without.clone()),
        ShapeReference::UpsertWithWhereUniqueInput(a, path) => ShapeReference::UpsertWithWhereUniqueInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::UpsertWithWhereUniqueInputWithout(a, path, without) => ShapeReference::UpsertWithWhereUniqueInputWithout(a.clone(), fix_path_inner(path, namespace), without.clone()),
        ShapeReference::UpdateManyWithWhereInput(a, path) => ShapeReference::UpdateManyWithWhereInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::UpdateManyWithWhereInputWithout(a, path, without) => ShapeReference::UpdateManyWithWhereInputWithout(a.clone(), fix_path_inner(path, namespace), without.clone()),
        ShapeReference::Select(a, path) => ShapeReference::Select(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::Include(a, path) => ShapeReference::Include(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::OrderByInput(a, path) => ShapeReference::OrderByInput(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::Result(a, path) => ShapeReference::Result(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::CountAggregateResult(a, path) => ShapeReference::CountAggregateResult(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::SumAggregateResult(a, path) => ShapeReference::SumAggregateResult(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::AvgAggregateResult(a, path) => ShapeReference::AvgAggregateResult(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::MinAggregateResult(a, path) => ShapeReference::MinAggregateResult(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::MaxAggregateResult(a, path) => ShapeReference::MaxAggregateResult(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::AggregateResult(a, path) => ShapeReference::AggregateResult(a.clone(), fix_path_inner(path, namespace)),
        ShapeReference::GroupByResult(a, path) => ShapeReference::GroupByResult(a.clone(), fix_path_inner(path, namespace)),
        _ => shape_reference.clone()
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
        Type::EnumVariant(a, path) => Type::EnumVariant(a.clone(), fix_path_inner(path, namespace)),
        Type::InterfaceObject(a, types, path) => Type::InterfaceObject(a.clone(), types.iter().map(|t| fix_path(t, namespace)).collect(), fix_path_inner(path, namespace)),
        Type::ModelObject(a, path) => Type::ModelObject(a.clone(), fix_path_inner(path, namespace)),
        Type::StructObject(a, path) => Type::StructObject(a.clone(), fix_path_inner(path, namespace)),
        Type::ModelScalarFields(_, _) => panic!(),
        Type::ModelScalarFieldsWithoutVirtuals(_, _) => panic!(),
        Type::ModelScalarFieldsAndCachedPropertiesWithoutVirtuals(_, _) => panic!(),
        Type::ModelRelations(_, _) => panic!(),
        Type::ModelDirectRelations(_, _) => panic!(),
        Type::FieldType(_, _) => panic!(),
        Type::FieldReference(_) => panic!(),
        Type::GenericItem(name) => Type::GenericItem(name.clone()),
        Type::Keyword(keyword) => Type::Keyword(keyword.clone()),
        Type::Optional(inner) => Type::Optional(Box::new(fix_path(inner.as_ref(), namespace))),
        Type::Pipeline(_) => panic!(),
        Type::DataSetObject(_, _) => panic!(),
        Type::DataSetRecord(_, _) => panic!(),
        Type::ShapeReference(shape_reference) => Type::ShapeReference(fix_path_shape_reference(shape_reference, namespace)),
    }
}

fn generics_declaration(names: &Vec<String>) -> String {
    if names.is_empty() {
        "".to_owned()
    } else {
        "<".to_owned() + &names.join(", ") + ">"
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

fn unwrap_extend(extend: &Type) -> Result<String> {
    let interface_path = extend.as_interface_object().unwrap().2.join("::");
    let a = extend.as_interface_object().unwrap().1;
    Ok(if a.is_empty() {
        interface_path
    } else {
        interface_path + "<" + &a.iter().map(|e| {
            if e.is_interface_object() {
                unwrap_extend(e)
            } else {
                Ok(rust::lookup(e)?)
            }
        }).collect::<Result<Vec<String>>>()?.join(", ") + ">"
    })
}

fn unwrap_extends(extends: &Vec<Type>) -> Result<Vec<String>> {
    Ok(extends.iter().map(|extend| {
        unwrap_extend(extend)
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
    pub(self) format_model_path: &'static dyn Fn(Vec<&str>) -> String,
    pub(self) generics_declaration: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) phantom_generics: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) unwrap_extends: &'static dyn Fn(&Vec<Type>) -> Result<Vec<String>>,
    pub(self) super_keywords: &'static dyn Fn(Vec<&str>) -> String,
    pub(self) fix_path: &'static dyn Fn(&Type, &Namespace) -> Type,
}

unsafe impl Send for RustModuleTemplate<'_> { }
unsafe impl Sync for RustModuleTemplate<'_> { }

impl<'a> RustModuleTemplate<'a> {

    fn new(namespace: &'a Namespace) -> Self {
        let mut has_date = false;
        let mut has_datetime = false;
        let mut has_decimal = false;
        let mut has_object_id = false;
        if !namespace.is_main() {
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
            format_model_path: &format_model_path,
            generics_declaration: &generics_declaration,
            phantom_generics: &phantom_generics,
            unwrap_extends: &unwrap_extends,
            super_keywords: &super_keywords,
            fix_path: &fix_path,
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


