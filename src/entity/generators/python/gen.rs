use std::path::{Path, PathBuf};
use askama::Template;
use async_recursion::async_recursion;
use async_trait::async_trait;
use itertools::Itertools;
use teo_runtime::namespace::Namespace;
use std::borrow::Borrow;
use crate::entity::ctx::Ctx;
use crate::entity::generator::Generator;
use crate::outline::outline::{Mode, Outline};
use crate::utils::file::FileUtil;
use std::str::FromStr;
use inflector::Inflector;
use teo_parser::r#type::reference::Reference;
use teo_parser::r#type::synthesized_enum_reference::SynthesizedEnumReference;
use teo_parser::r#type::synthesized_shape_reference::SynthesizedShapeReference;
use teo_parser::r#type::Type;
use teo_runtime::traits::named::Named;
use teo_runtime::model::field::typed::Typed;
use crate::entity::generators::python::lookup;
use crate::utils::filters;
use crate::utils::lookup::Lookup;

fn typed_dict_not_required(original: String) -> String {
    if original.starts_with("Optional[") {
        format!("NotRequired[{}]", original)
    } else {
        original
    }
}

fn fix_path_inner(components: &Vec<String>, namespace: &Namespace, root_module_name: &str) -> Vec<String> {
    let namespace_path = namespace.path();
    let components_without_last: Vec<String> = components.iter().rev().skip(1).rev().map(Clone::clone).collect();
    if namespace_path == &components_without_last {
        vec![components.last().unwrap().to_owned()]
    } else {
        if namespace.path().len() > 0 {
            let mut result = components.clone();
            result.insert(0, root_module_name.to_owned());
            result
        } else {
            components.clone()
        }
    }
}

fn fix_path_enum_reference(enum_reference: &SynthesizedEnumReference, namespace: &Namespace, root_module_name: &str) -> SynthesizedEnumReference {
    SynthesizedEnumReference {
        kind: enum_reference.kind,
        owner: Box::new(fix_path(enum_reference.owner.as_ref(), namespace, root_module_name)),
    }
}

fn fix_path_shape_reference(shape_reference: &SynthesizedShapeReference, namespace: &Namespace, root_module_name: &str) -> SynthesizedShapeReference {
    SynthesizedShapeReference {
        kind: shape_reference.kind,
        owner: Box::new(fix_path(shape_reference.owner.as_ref(), namespace, root_module_name)),
        without: shape_reference.without.clone(),
    }
}

fn fix_path(t: &Type, namespace: &Namespace, root_module_name: &str) -> Type {
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
        Type::Enumerable(inner) => Type::Enumerable(Box::new(fix_path(inner.as_ref(), namespace, root_module_name))),
        Type::Array(inner) => Type::Array(Box::new(fix_path(inner.as_ref(), namespace, root_module_name))),
        Type::Dictionary(inner) => Type::Dictionary(Box::new(fix_path(inner.as_ref(), namespace, root_module_name))),
        Type::Tuple(types) => Type::Tuple(types.iter().map(|t| fix_path(t, namespace, root_module_name)).collect()),
        Type::Range(inner) => Type::Range(Box::new(fix_path(inner.as_ref(), namespace, root_module_name))),
        Type::Union(types) => Type::Union(types.iter().map(|t| fix_path(t, namespace, root_module_name)).collect()),
        Type::EnumVariant(reference) => Type::EnumVariant(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace, root_module_name))),
        Type::InterfaceObject(reference, types) => Type::InterfaceObject(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace, root_module_name)), types.iter().map(|t| fix_path(t, namespace, root_module_name)).collect()),
        Type::ModelObject(reference) => Type::ModelObject(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace, root_module_name))),
        Type::StructObject(reference, types) => Type::StructObject(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace, root_module_name)), types.iter().map(|t| fix_path(t, namespace, root_module_name)).collect()),
        Type::GenericItem(name) => Type::GenericItem(name.clone()),
        Type::Keyword(keyword) => Type::Keyword(keyword.clone()),
        Type::Optional(inner) => Type::Optional(Box::new(fix_path(inner.as_ref(), namespace, root_module_name))),
        Type::SynthesizedShapeReference(shape_reference) => Type::SynthesizedShapeReference(fix_path_shape_reference(shape_reference, namespace, root_module_name)),
        Type::SynthesizedEnumReference(enum_reference) => Type::SynthesizedEnumReference(fix_path_enum_reference(enum_reference, namespace, root_module_name)),
        Type::DeclaredSynthesizedShape(reference, inner) => Type::DeclaredSynthesizedShape(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace, root_module_name)), Box::new(fix_path(inner, namespace, root_module_name))),
        _ => panic!(),
    }
}

#[derive(Template)]
#[template(path = "entity/python/__init__.py.jinja", escape = "none")]
pub(self) struct PythonModuleTemplate<'a> {
    pub(self) root_module_name: String,
    pub(self) namespace: &'a Namespace,
    pub(self) outline: Outline,
    pub(self) lookup: &'static dyn Lookup,
    pub(self) fix_path: &'static dyn Fn(&Type, &Namespace, &str) -> Type,
    pub(self) dots: &'static dyn Fn(usize) -> String,
    pub(self) typed_dict_not_required: &'static dyn Fn(String) -> String,
}

unsafe impl Send for PythonModuleTemplate<'_> { }
unsafe impl Sync for PythonModuleTemplate<'_> { }

impl<'a> PythonModuleTemplate<'a> {

    fn new(namespace: &'a Namespace, main_namespace: &'a Namespace, last_path_component: String) -> Self {
        Self {
            namespace,
            outline: Outline::new(namespace, Mode::Entity, main_namespace, false),
            lookup: &lookup,
            fix_path: &fix_path,
            root_module_name: last_path_component,
            dots: &dots,
            typed_dict_not_required: &typed_dict_not_required,
        }
    }
}

pub(crate) struct PythonGenerator { }

impl PythonGenerator {

    pub fn new() -> Self {
        Self { }
    }

    async fn generate_module_file(&self, namespace: &Namespace, filename: impl AsRef<Path>, generator: &FileUtil, main_namespace: &Namespace, last_path_component: &str) -> teo_result::Result<()> {
        let template = PythonModuleTemplate::new(namespace, main_namespace, last_path_component.to_owned());
        generator.generate_file(filename.as_ref(), template.render().unwrap()).await?;
        Ok(())
    }

    #[async_recursion]
    async fn generate_module_for_namespace(&self, namespace: &Namespace, generator: &FileUtil, main_namespace: &Namespace, last_path_component: &str) -> teo_result::Result<()> {
        if namespace.is_main() || !namespace.namespaces().is_empty() {
            // create dir and create mod.rs
            if !namespace.is_main() {
                generator.ensure_directory(namespace.path().iter().map(|s| s.to_snake_case()).join("/")).await?;
            }
            self.generate_module_file(
                namespace,
                PathBuf::from_str(&namespace.path().iter().map(|s| s.to_snake_case()).join("/")).unwrap().join("__init__.py"),
                generator,
                main_namespace,
                last_path_component,
            ).await?;
        } else {
            // create file
            self.generate_module_file(
                namespace,
                PathBuf::from_str(&namespace.path().iter().rev().skip(1).rev().map(|s| s.to_snake_case()).collect::<Vec<String>>().join("/")).unwrap().join(namespace.path().last().unwrap().to_snake_case() + ".py"),
                generator,
                main_namespace,
                last_path_component,
            ).await?;
        }
        for namespace in namespace.namespaces().values() {
            self.generate_module_for_namespace(namespace, generator, main_namespace, last_path_component).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl Generator for PythonGenerator {

    async fn generate_entity_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        // module files
        self.generate_module_for_namespace(ctx.main_namespace, generator, ctx.main_namespace, last_path_component(&ctx.conf.dest).as_str()).await?;
        Ok(())
    }
}

fn last_path_component(dest: &str) -> String {
    let path = Path::new(dest);
    path.components().last().unwrap().as_os_str().to_str().unwrap().to_owned()
}

fn dots(times: usize) -> String {
    (0..times).map(|_| ".").collect::<String>()
}