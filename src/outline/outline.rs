use inflector::Inflector;
use indexmap::indexmap;
use itertools::Itertools;
use teo_parser::r#type::synthesized_enum::SynthesizedEnum;
use teo_parser::r#type::synthesized_shape::SynthesizedShape;
use teo_parser::r#type::Type;
use teo_runtime::handler::Handler;
use teo_runtime::model::field::typed::Typed;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::documentable::Documentable;
use teo_runtime::traits::named::Named;
use crate::outline::delegate::{Delegate, GroupItem, NamespaceItem, RequestItem};
use crate::outline::interface::{Field, Interface};
use crate::outline::path_arguments::PathArguments;
use crate::outline::r#enum::{Enum, Member};

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Mode {
    Client,
    Entity,
}

impl Mode {

    pub fn is_client(&self) -> bool {
        match self {
            Mode::Client => true,
            _ => false,
        }
    }

    pub fn is_entity(&self) -> bool {
        match self {
            Mode::Entity => true,
            _ => false,
        }
    }
}

pub(crate) struct Outline {
    interfaces: Vec<Interface>,
    enums: Vec<Enum>,
    path_arguments: Vec<PathArguments>,
    delegates: Vec<Delegate>,
}

impl Outline {

    pub fn new(namespace: &Namespace, mode: Mode, main_namespace: &Namespace) -> Self {
        let mut interfaces = vec![];
        let mut enums = vec![];
        // enums
        for r#enum in namespace.enums.values() {
            if !r#enum.interface && !r#enum.option {
                enums.push(Enum {
                    title: r#enum.title(),
                    desc: r#enum.desc(),
                    path: r#enum.path.clone(),
                    name: r#enum.name().to_owned(),
                    members: r#enum.members().iter().map(|member| {
                        Member {
                            title: member.title(),
                            desc: member.desc(),
                            name: member.name().to_owned(),
                        }
                    }).collect()
                });
            }
        }
        // interfaces
        for interface in namespace.interfaces.values().sorted_by_key(|i| i.parser_path.last().unwrap()) {
            interfaces.push(Interface {
                title: interface.title(),
                desc: interface.desc(),
                path: interface.path.clone(),
                name: interface.name().to_owned(),
                generic_names: interface.generic_names.clone(),
                extends: interface.extends.clone(),
                fields: interface.fields.values().map(|field| {
                    Field {
                        title: field.title(),
                        desc: field.desc(),
                        name: field.name().to_owned(),
                        r#type: field.r#type().clone(),
                    }
                }).collect(),
                synthesized: None,
                model_name: None,
            });
        }
        // model caches
        for model in namespace.models.values() {
            if (mode == Mode::Entity && model.generate_entity) || (mode == Mode::Client && model.generate_client) {
                for ((shape_name, shape_without), input) in &model.cache.shape.shapes {
                    if let Some(shape) = input.as_synthesized_shape() {
                        interfaces.push(shape_interface_from_cache(shape, &shape_name.to_string(), shape_without, model, mode));
                    } else if let Some(union) = input.as_union() {
                        let shape = make_shape_from_union(union);
                        interfaces.push(shape_interface_from_cache(&shape, &shape_name.to_string(), shape_without, model, mode));
                    }
                }
                for (enum_name, input) in &model.cache.shape.enums {
                    enums.push(shape_enum_from_cache(input, &enum_name.to_string(), model));
                }
                for ((def_path, shape)) in &model.cache.shape.declared_shapes {
                    interfaces.push(shape_interface_from_cache(shape, def_path.last().unwrap(), &None, model, mode));
                }
            }
        }
        // delegates
        let mut delegates = vec![];
        for model in namespace.models.values() {
            if !(model.generate_client && model.synthesize_shapes) {
                continue
            }
            let mut request_items = vec![];
            for action in &model.builtin_handlers {
                let input_type = model.input_type_for_builtin_handler(*action);
                let output_type = model.output_type_for_builtin_handler(*action, main_namespace);
                request_items.push(RequestItem {
                    name: action.as_handler_str().to_owned(),
                    input_type,
                    output_type,
                    has_custom_url_args: false,
                    is_form: false,
                    has_body_input: true,
                    is_aggregate: action.as_handler_str() == "aggregate",
                    is_count: action.as_handler_str() == "count",
                    is_group_by: action.as_handler_str() == "groupBy",
                    path: format!("{}/{}", model.path.join("/"), action.as_handler_str()),
                    method: "POST",
                    custom_url_args_path: None,
                    is_builtin: true,
                });
            }
            if let Some(handler_group) = namespace.model_handler_groups.get(model.name()) {
                for (name, handler) in &handler_group.handlers {
                    if !handler.nonapi {
                        request_items.push(RequestItem {
                            name: name.to_owned(),
                            input_type: handler.input_type.clone(),
                            output_type: handler.output_type.clone(),
                            has_custom_url_args: handler.has_custom_url_args(),
                            is_form: handler.format.is_form(),
                            has_body_input: handler.has_body_input(),
                            is_group_by: false,
                            is_count: false,
                            is_aggregate: false,
                            path: path_for_custom_handler(handler),
                            method: handler.method.capitalized_name(),
                            custom_url_args_path: handler.custom_url_args_path(),
                            is_builtin: false,
                        });
                    }
                }
            }
            let delegate = Delegate::new(model.name().to_owned() + "Delegate", vec![], vec![], request_items);
            delegates.push(delegate);
        }
        for handler_group in namespace.handler_groups.values() {
            let mut request_items = vec![];
            for (name, handler) in &handler_group.handlers {
                if !handler.nonapi {
                    request_items.push(RequestItem {
                        name: name.to_owned(),
                        input_type: handler.input_type.clone(),
                        output_type: handler.output_type.clone(),
                        has_custom_url_args: handler.has_custom_url_args(),
                        is_form: handler.format.is_form(),
                        has_body_input: handler.has_body_input(),
                        is_aggregate: false,
                        is_group_by: false,
                        is_count: false,
                        path: path_for_custom_handler(handler),
                        method: handler.method.capitalized_name(),
                        custom_url_args_path: handler.custom_url_args_path(),
                        is_builtin: false,
                    });
                }
            }
            let delegate = Delegate::new(handler_group.path.last().unwrap().to_owned() + "Delegate", vec![], vec![], request_items);
            delegates.push(delegate);
        }
        let self_delegate_name = if namespace.path().is_empty() {
            "".to_owned()
        } else {
            namespace.path.last().unwrap().to_pascal_case() + "NamespaceDelegate"
        };
        let mut model_items = vec![];
        let mut namespace_items = vec![];
        let mut request_items = vec![];
        for model in namespace.models.values() {
            if !(model.generate_entity && model.synthesize_shapes) {
                continue
            }
            model_items.push(GroupItem {
                name: model.name().to_owned() + "Delegate",
                path: {
                    let mut path = model.path.clone();
                    path.pop();
                    path.push(model.name().to_owned() + "Delegate");
                    path
                },
                property_name: model.name().to_camel_case(),
            })
        }
        for handler in namespace.handlers.values() {
            if !handler.nonapi {
                request_items.push(RequestItem {
                    name: handler.name().to_owned(),
                    input_type: handler.input_type.clone(),
                    output_type: handler.output_type.clone(),
                    has_custom_url_args: handler.has_custom_url_args(),
                    is_form: handler.format.is_form(),
                    has_body_input: handler.has_body_input(),
                    is_count: false,
                    is_aggregate: false,
                    is_group_by: false,
                    path: path_for_custom_handler(handler),
                    method: handler.method.capitalized_name(),
                    custom_url_args_path: handler.custom_url_args_path(),
                    is_builtin: false,
                });
            }
        }
        for handler_group in namespace.handler_groups.values() {
            model_items.push(GroupItem {
                name: handler_group.name().to_owned() + "Delegate",
                path: {
                    let mut path = handler_group.path.clone();
                    path.pop();
                    path.push(handler_group.name().to_owned() + "Delegate");
                    path
                },
                property_name: handler_group.name().to_camel_case(),
            })
        }
        for child_ns in namespace.namespaces.values() {
            namespace_items.push(NamespaceItem {
                name: child_ns.name().to_owned() + "NamespaceDelegate",
                path: {
                    let mut path = child_ns.path.clone();
                    path.push(child_ns.name().to_pascal_case() + "NamespaceDelegate");
                    path
                },
                property_name: child_ns.name().to_camel_case(),
            })
        }
        delegates.push(Delegate::new(self_delegate_name, model_items, namespace_items, request_items));
        // path arguments
        let mut path_arguments = vec![];
        for handler in namespace.handlers.values() {
            install_path_arguments(&mut path_arguments, handler);
        }
        for handler_group in namespace.handler_groups.values() {
            for handler in handler_group.handlers.values() {
                install_path_arguments(&mut path_arguments, handler);
            }
        }
        for model_handler_group in namespace.model_handler_groups.values() {
            for handler in model_handler_group.handlers.values() {
                install_path_arguments(&mut path_arguments, handler);
            }
        }
        Self { interfaces, enums, delegates, path_arguments }
    }

    pub(crate) fn interfaces(&self) -> &Vec<Interface> {
        &self.interfaces
    }

    pub(crate) fn enums(&self) -> &Vec<Enum> {
        &self.enums
    }

    pub(crate) fn delegates(&self) -> &Vec<Delegate> {
        &self.delegates
    }

    pub(crate) fn path_arguments(&self) -> &Vec<PathArguments> {
        &self.path_arguments
    }
}

fn install_path_arguments(path_arguments: &mut Vec<PathArguments>, handler: &Handler) {
    if let Some(interface) = handler.interface.as_ref() {
        path_arguments.push(PathArguments {
            name: interface.to_string(),
            items: PathArguments::fetch_items(handler.url.as_ref())
        })
    }
}

fn shape_interface_from_cache(shape: &SynthesizedShape, shape_name: &String, shape_without: &Option<String>, model: &Model, mode: Mode) -> Interface {
    let name = if let Some(without) = shape_without {
        model.name().to_owned() + shape_name.as_str().strip_suffix("InputWithout").unwrap() + "Without" + &without.to_pascal_case() + "Input"
    } else {
        if mode == Mode::Client && shape_name == "Result" {
            model.name().to_owned()
        } else {
            model.name().to_owned() + shape_name
        }
    };
    Interface {
        title: name.to_sentence_case(),
        desc: "This synthesized interface doesn't have a description".to_owned(),
        path: {
            let mut result = model.path.clone();
            result.pop();
            result.push(name.clone());
            result
        },
        name,
        generic_names: vec![],
        extends: vec![],
        fields: shape.iter().map(|(name, r#type)| {
            Field {
                title: name.to_title_case(),
                desc: "This synthesized field doesn't have a description.".to_owned(),
                name: name.clone(),
                r#type: r#type.clone(),
            }
        }).collect(),
        synthesized: Some((shape_name.clone(), shape_without.clone())),
        model_name: Some(model.name().to_owned()),
    }
}

fn shape_enum_from_cache(r#enum: &SynthesizedEnum, shape_name: &String, model: &Model) -> Enum {
    let name = model.name().to_owned() + shape_name;
    Enum {
        title: name.to_sentence_case(),
        desc: "This synthesized enum doesn't have a description.".to_owned(),
        path: {
            let mut result = model.path.clone();
            result.pop();
            result.push(name.clone());
            result
        },
        name,
        members: r#enum.members.values().map(|member| {
            Member {
                title: member.name.to_sentence_case(),
                desc: "This synthesized enum member doesn't have a description.".to_owned(),
                name: member.name.clone()
            }
        }).collect(),
    }
}

fn make_shape_from_union(union: &Vec<Type>) -> SynthesizedShape {
    let mut result = SynthesizedShape::new(indexmap! {});
    let mut times = 0;
    for t in union {
        if let Some(shape) = t.as_synthesized_shape() {
            result.extend(shape.clone().into_iter());
            times += 1;
        }
    }
    if times > 1 {
        result.iter_mut().for_each(|(_, t)| {
            *t = t.wrap_in_optional();
        })
    }
    result
}

fn path_for_custom_handler(handler: &Handler) -> String {
    if let Some(url) = &handler.url {
        if handler.ignore_prefix {
            url.clone()
        } else {
            format!("{}{}", handler.path.join("/"), if url.starts_with("/") {
                url.as_str().to_owned()
            } else {
                "/".to_owned() + url.as_str()
            })
        }
    } else {
        handler.path.join("/") + "/" + handler.name()
    }
}