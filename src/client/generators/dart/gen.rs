use async_trait::async_trait;
use teo_runtime::config::client::Client;
use crate::client::ctx::Ctx;
use crate::client::generator::Generator;
use std::process::Command;
use askama::Template;
use crate::outline::outline::{Mode, Outline};
use crate::utils::exts::ClientExt;
use crate::utils::file::FileUtil;
use crate::utils::message::green_message;
use crate::utils::filters;
use async_recursion::async_recursion;
use teo_result::Result;
use regex::Regex;
use teo_runtime::namespace::Namespace;
use tokio::fs;
use std::borrow::Borrow;
use std::collections::BTreeSet;
use itertools::Itertools;
use maplit::btreeset;
use teo_parser::r#type::reference::Reference;
use teo_parser::r#type::synthesized_enum_reference::SynthesizedEnumReference;
use teo_parser::r#type::synthesized_shape_reference::SynthesizedShapeReference;
use teo_parser::r#type::Type;
use crate::client::generators::dart::lookup;
use crate::client::generators::dart::pubspec::updated_pubspec_yaml_for_existing_project;
use crate::utils::lookup::Lookup;

fn import_dots(namespace: &Namespace) -> String {
    if namespace.path.len() <= 1 {
        "".to_owned()
    } else {
        "../".repeat(namespace.path().len() - 1)
    }
}

fn should_escape(name: &str) -> bool {
    name.starts_with("_") || ["is", "in", "AND", "OR", "NOT"].contains(&name)
}

fn type_is_not_dynamic(t: &str) -> bool {
    t != "dynamic"
}

fn type_is_dynamic(t: &str) -> bool {
    t == "dynamic"
}

fn from_json_parameters(names: &Vec<String>) -> String {
    names.iter().map(|n| format!(", {} Function(Object? json) fromJson{}", n, n)).join("")
}

fn from_json_arguments(names: &Vec<String>) -> String {
    names.iter().map(|n| format!(", fromJson{}", n)).join("")
}

fn to_json_parameters(names: &Vec<String>) -> String {
    names.iter().map(|n| format!(", Object Function({} value) toJson{}", n, n)).join("")
}

fn to_json_arguments(names: &Vec<String>) -> String {
    names.iter().map(|n| ", anyToJson".to_string()).join("")
    //names.iter().map(|n| format!(", toJson{}", n)).join("")
}

fn from_json_from_type(t: &Type) -> String {
    match t {
        Type::Optional(t) => from_json_from_type(t),
        Type::Date => "(p0) => fromTeoDate(p0 as dynamic)".to_owned(),
        Type::DateTime => "(p0) => fromTeoDateTime(p0 as dynamic)".to_owned(),
        Type::Decimal => "(p0) => fromTeoDecimal(p0 as dynamic)".to_owned(),
        Type::Int | Type::Int64 => "(p0) => p0 as dynamic".to_owned(),
        Type::Float | Type::Float32 => "(p0) => (p0 as dynamic).toDouble()".to_owned(),
        Type::Bool => "(p0) => p0 as dynamic".to_owned(),
        Type::String | Type::ObjectId => "(p0) => p0 as dynamic".to_owned(),
        Type::Null => "(p0) => null".to_owned(),
        Type::Array(inner) => format!("(p0) => (p0 as List).map({}).toList()", from_json_from_type(inner.as_ref())),
        _ => {
            let args = t.generic_types().iter().map(|gt| format!(", {}", from_json_from_type(gt))).join("");
            let mut this_str = lookup(t).unwrap();
            let without_generics = &this_str.as_str()[0..this_str.find("<").unwrap_or(this_str.len())];
            format!("(p0) => {}.fromJson(p0 as dynamic{})", without_generics, args)
        },
    }
}

fn append_question(original: String, output: bool) -> String {
    if output {
        if !type_is_dynamic(original.as_str()) && !original.ends_with("?") {
            original + "?"
        } else {
            original
        }
    } else {
        original
    }
}

fn module_name(path: Vec<&str>, client: &Client) -> String {
    if path.len() == 0 {
        client.object_name.clone()
    } else {
        path.join("_")
    }
}

fn insert_to_import_set_if_needed(target_path: &Vec<String>, this_path: &Vec<String>, exist_check_set: &mut BTreeSet<Vec<String>>, result: &mut BTreeSet<(String, String)>, client: &Client) {
    if exist_check_set.contains(target_path) {
        return
    }
    if target_path == this_path {
        return
    }
    let mut left = this_path.len();
    let mut results = vec![];
    for (index, component) in target_path.iter().enumerate() {
        if let Some(ns_component) = this_path.get(index) {
            if component == ns_component {
                left -= 1;
            } else {
                results.push(component.clone());
            }
        } else {
            results.push(component.clone());
        }
    }
    for _ in 0..left {
        results.insert(0, "..".to_owned());
    }
    if target_path.is_empty() {
        results.push(format!("{}", client.object_name.as_str()));
    }
    result.insert((format!("{}.dart", results.join("/")), if target_path.is_empty() {
        client.object_name.clone()
    } else {
        target_path.join("_")
    }));
}

fn figure_out_imports_from_type(t: &Type, this_path: &Vec<String>, exist_check_set: &mut BTreeSet<Vec<String>>, result: &mut BTreeSet<(String, String)>, client: &Client) {
    match t {
        Type::Optional(inner) => {
            figure_out_imports_from_type(inner.as_ref(), this_path, exist_check_set, result, client);
        }
        Type::Array(inner) => {
            figure_out_imports_from_type(inner.as_ref(), this_path, exist_check_set, result, client);
        }
        Type::Dictionary(inner) => {
            figure_out_imports_from_type(inner.as_ref(), this_path, exist_check_set, result, client);
        }
        Type::ModelObject(r) => {
            insert_to_import_set_if_needed(&r.string_path_without_last(1), this_path, exist_check_set, result, client);
        }
        Type::InterfaceObject(r, types) => {
            insert_to_import_set_if_needed(&r.string_path_without_last(1), this_path, exist_check_set, result, client);
            for ty in types {
                figure_out_imports_from_type(ty, this_path, exist_check_set, result, client);
            }
        }
        Type::SynthesizedShapeReference(s) => figure_out_imports_from_type(s.owner.as_ref(), this_path, exist_check_set, result, client),
        Type::SynthesizedEnumReference(e) => figure_out_imports_from_type(e.owner.as_ref(), this_path, exist_check_set, result, client),
        _ => ()
    }
}

fn namespace_imports(namespace: &Namespace, outline: &Outline, client: &Client) -> String {
    let this_path = namespace.path.clone();
    let mut exist_check_set: BTreeSet<Vec<String>> = btreeset!{};
    let mut result: BTreeSet<(String, String)> = btreeset!{};
    for interface in outline.interfaces() {
        for field in interface.fields() {
            figure_out_imports_from_type(field.r#type(), &this_path, &mut exist_check_set, &mut result, client);
        }
    }
    for child_namespace in namespace.namespaces.values() {
        insert_to_import_set_if_needed(&child_namespace.path, &this_path, &mut exist_check_set, &mut result, client);
    }
    for delegate in outline.delegates() {
        for request_item in delegate.request_items() {
            figure_out_imports_from_type(request_item.input_type(), &this_path, &mut exist_check_set, &mut result, client);
            figure_out_imports_from_type(request_item.output_type(), &this_path, &mut exist_check_set, &mut result, client);
        }
    }
    result.iter().map(|(s, a)| format!("import '{}' as {};", s, a)).join("\n")
}

#[derive(Template)]
#[template(path = "client/dart/readme.md.jinja", escape = "none")]
pub(self) struct DartReadMeTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/dart/pubspec.yaml.jinja", escape = "none")]
pub(self) struct DartPubspecTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/dart/helper.dart.jinja", escape = "none")]
pub(self) struct DartHelperTemplate<'a> {
    pub(self) conf: &'a Client,
}

#[derive(Template)]
#[template(path = "client/dart/namespace.dart.jinja", escape = "none")]
pub(self) struct DartMainTemplate<'a> {
    pub(self) namespace: &'a Namespace,
    pub(self) outline: &'a Outline,
    pub(self) conf: &'a Client,
    pub(self) should_escape: &'static dyn Fn(&str) -> bool,
    pub(self) type_is_not_dynamic: &'static dyn Fn(&str) -> bool,
    pub(self) type_is_dynamic: &'static dyn Fn(&str) -> bool,
    pub(self) import_dots: &'static dyn Fn(&Namespace) -> String,
    pub(self) append_question: &'static dyn Fn(String, bool) -> String,
    pub(self) from_json_parameters: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) from_json_arguments: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) to_json_parameters: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) to_json_arguments: &'static dyn Fn(&Vec<String>) -> String,
    pub(self) from_json_from_type: &'static dyn Fn(&Type) -> String,
    pub(self) namespace_imports: &'static dyn Fn(&Namespace, &Outline, &Client) -> String,
    pub(self) fix_path: &'static dyn Fn(&Type, &Namespace, &Client) -> Type,
    pub(self) lookup: &'static dyn Lookup,
}

unsafe impl Send for DartMainTemplate<'_> { }
unsafe impl Sync for DartMainTemplate<'_> { }

pub(in crate::client) struct DartGenerator {}

impl DartGenerator {

    pub fn new() -> Self {
        Self {}
    }

    #[async_recursion]
    async fn generate_module_for_namespace(&self, namespace: &Namespace, generator: &FileUtil, main_namespace: &Namespace, conf: &Client) -> Result<()> {
        let outline = Outline::new(namespace, Mode::Client, main_namespace);
        generator.generate_file(if namespace.path().is_empty() {
            format!("{}.dart", conf.inferred_package_name_snake_case())
        } else {
            format!("{}.dart", namespace.path().join("/"))
        }, DartMainTemplate {
            namespace,
            outline: &outline,
            conf,
            should_escape: &should_escape,
            type_is_not_dynamic: &type_is_not_dynamic,
            type_is_dynamic: &type_is_dynamic,
            import_dots: &import_dots,
            append_question: &append_question,
            from_json_parameters: &from_json_parameters,
            from_json_arguments: &from_json_arguments,
            to_json_parameters: &to_json_parameters,
            to_json_arguments: &to_json_arguments,
            from_json_from_type: &from_json_from_type,
            namespace_imports: &namespace_imports,
            fix_path: &fix_path,
            lookup: &lookup,
        }.render().unwrap()).await?;
        for child in namespace.namespaces.values() {
            self.generate_module_for_namespace(child, generator, main_namespace, conf).await?;
        }
        Ok(())
    }

    async fn generate_helper(&self, generator: &FileUtil, conf: &Client) -> Result<()> {
        generator.generate_file("_helper.dart", DartHelperTemplate { conf }.render().unwrap()).await?;
        Ok(())
    }
}

#[async_trait]
impl Generator for DartGenerator {

    fn module_directory_in_package(&self, conf: &Client) -> String {
        "lib".to_owned()
    }

    async fn generate_module_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.clear_root_directory().await?;
        Ok(())
    }

    async fn generate_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        generator.ensure_root_directory().await?;
        generator.generate_file(".gitignore", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/client/dart/gitignore"))).await?;
        generator.generate_file("README.md", DartReadMeTemplate { conf: ctx.conf }.render().unwrap()).await?;
        if generator.generate_file_if_not_exist("pubspec.yaml", DartPubspecTemplate { conf: ctx.conf }.render().unwrap()).await? {
            // if exists, update pubspec.yaml with a minor version
            let yaml_data = std::fs::read_to_string(generator.get_file_path("pubspec.yaml"))
                .expect("Unable to read pubspec.yaml");
            generator.generate_file("pubspec.yaml", update_pubspec_yaml_version(yaml_data)).await?;
        }
        Ok(())
    }

    async fn update_parent_package_files(&self, ctx: &Ctx, generator: &FileUtil) -> teo_result::Result<()> {
        if let Some(pubspec_yaml) = generator.find_file_upwards("pubspec.yaml") {
            let yaml_data = std::fs::read_to_string(&pubspec_yaml).expect("Unable to read pubspec.yaml");
            let updated_json_data = updated_pubspec_yaml_for_existing_project(yaml_data);
            fs::write(pubspec_yaml, updated_json_data).await?;
        }
        Ok(())
    }

    async fn generate_main(&self, ctx: &Ctx, generator: &FileUtil) -> Result<()> {
        // module files
        self.generate_module_for_namespace(ctx.main_namespace, generator, ctx.main_namespace, ctx.conf).await?;
        self.generate_helper(generator, ctx.conf).await?;
        // run commands
        //println!("debug error?: see base dir: {:?}", generator.get_base_dir());
        if let Some(pubspec_yaml) = generator.find_file_upwards("pubspec.yaml") {
            //println!("debug error?: see pubspec yaml dir: {:?}", pubspec_yaml);
            let project_root = pubspec_yaml.parent().unwrap();
            std::env::set_current_dir(project_root).unwrap();
            green_message("run", "`dart pub get`".to_owned());
            Command::new("dart").arg("pub").arg("get").spawn()?.wait()?;
            green_message("run", "`dart run build_runner build --delete-conflicting-outputs`".to_owned());
            Command::new("dart").arg("run").arg("build_runner").arg("build").arg("--delete-conflicting-outputs").spawn()?.wait()?;
        }
        Ok(())
    }
}

fn update_pubspec_yaml_version(mut content: String) -> String {
    let regex = Regex::new("version\\s*:\\s*([0-9\\.\\+]+)").unwrap();
    if let Some(captures) = regex.captures(content.as_str()) {
        if let Some(capture) = captures.get(1) {
            let current_version = capture.as_str();
            content.replace_range(capture.range(), next_minor_version(current_version).as_str());
            content
        } else {
            content
        }
    } else {
        content
    }
}

fn next_minor_version(current: &str) -> String {
    let regex = Regex::new("([0-9\\.]+)(\\+[0-9]+)").unwrap();
    if let Some(caps) = regex.captures(current) {
        if let Some(version_number) = caps.get(1) {
            let version_number_str = version_number.as_str();
            let parts = version_number_str.split(".");
            let last = parts.clone().last().unwrap();
            match last.parse::<u32>() {
                Ok(num) => {
                    let new_last = format!("{}", num + 1);
                    let vec_parts: Vec<&str> = parts.into_iter().collect();
                    let new_version = vec_parts.split_last().unwrap().1.join(".") + "." + &new_last;
                    new_version + "+1"
                },
                Err(_) => current.to_owned(),
            }
        } else {
            current.to_owned()
        }
    } else {
        current.to_owned()
    }
}

fn fix_path_inner(components: &Vec<String>, namespace: &Namespace, client: &Client) -> Vec<String> {
    let ns_path = namespace.path.clone();
    if components.len() == 1 && ns_path.is_empty() {
        components.clone()
    } else if components.len() == 1 && !ns_path.is_empty() {
        vec![client.object_name.clone(), components.first().unwrap().to_owned()]
    } else {
        let mut without_last = components.clone();
        without_last.pop();
        if without_last == ns_path {
            vec![components.last().unwrap().to_owned()]
        } else {
            components.clone()
        }
    }
}

fn fix_path_enum_reference(enum_reference: &SynthesizedEnumReference, namespace: &Namespace, client: &Client) -> SynthesizedEnumReference {
    SynthesizedEnumReference {
        kind: enum_reference.kind,
        owner: Box::new(fix_path(enum_reference.owner.as_ref(), namespace, client)),
    }
}

fn fix_path_shape_reference(shape_reference: &SynthesizedShapeReference, namespace: &Namespace, client: &Client) -> SynthesizedShapeReference {
    SynthesizedShapeReference {
        kind: shape_reference.kind,
        owner: Box::new(fix_path(shape_reference.owner.as_ref(), namespace, client)),
        without: shape_reference.without.clone(),
    }
}

fn fix_path(t: &Type, namespace: &Namespace, client: &Client) -> Type {
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
        Type::Enumerable(inner) => Type::Enumerable(Box::new(fix_path(inner.as_ref(), namespace, client))),
        Type::Array(inner) => Type::Array(Box::new(fix_path(inner.as_ref(), namespace, client))),
        Type::Dictionary(inner) => Type::Dictionary(Box::new(fix_path(inner.as_ref(), namespace, client))),
        Type::Tuple(types) => Type::Tuple(types.iter().map(|t| fix_path(t, namespace, client)).collect()),
        Type::Range(inner) => Type::Range(Box::new(fix_path(inner.as_ref(), namespace, client))),
        Type::Union(types) => Type::Union(types.iter().map(|t| fix_path(t, namespace, client)).collect()),
        Type::EnumVariant(reference) => Type::EnumVariant(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace, client))),
        Type::InterfaceObject(reference, types) => Type::InterfaceObject(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace, client)), types.iter().map(|t| fix_path(t, namespace, client)).collect()),
        Type::ModelObject(reference) => Type::ModelObject(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace, client))),
        Type::StructObject(reference, types) => Type::StructObject(Reference::new(reference.path().clone(), fix_path_inner(reference.string_path(), namespace, client)), types.iter().map(|t| fix_path(t, namespace, client)).collect()),
        Type::GenericItem(name) => Type::GenericItem(name.clone()),
        Type::Keyword(keyword) => Type::Keyword(keyword.clone()),
        Type::Optional(inner) => Type::Optional(Box::new(fix_path(inner.as_ref(), namespace, client))),
        Type::SynthesizedShapeReference(shape_reference) => Type::SynthesizedShapeReference(fix_path_shape_reference(shape_reference, namespace, client)),
        Type::SynthesizedEnumReference(enum_reference) => Type::SynthesizedEnumReference(fix_path_enum_reference(enum_reference, namespace, client)),
        _ => panic!(),
    }
}