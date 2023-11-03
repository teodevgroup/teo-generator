use std::borrow::Cow;
use inflector::Inflector;
use teo_parser::shape::shape::Shape;
use teo_parser::shape::synthesized_enum::SynthesizedEnum;
use teo_runtime::model::field::typed::Typed;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::documentable::Documentable;
use teo_runtime::traits::named::Named;
use crate::entity::outline::interface::{Field, Interface};
use crate::entity::outline::r#enum::{Enum, Member};

pub(in crate::entity) struct Outline {
    interfaces: Vec<Interface>,
    enums: Vec<Enum>,
}

impl Outline {

    pub fn new<L>(namespace: &Namespace) -> Self {
        let mut interfaces = vec![];
        let mut enums = vec![];
        // enums
        for r#enum in namespace.enums.values() {
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
        // interfaces
        for interface in namespace.interfaces.values() {
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
            });
        }
        // model caches
        for model in namespace.models.values() {
            for ((shape_name, shape_without), input) in &model.cache.shape.map {
                if let Some(shape) = input.as_shape() {
                    interfaces.push(shape_interface_from_cache(shape, shape_name, shape_without, model));
                } else if let Some(r#enum) = input.as_synthesized_enum() {
                    enums.push(shape_enum_from_cache(&r#enum, shape_name, model));
                } else if input.is_or() {
                    let shape = input.or_to_shape();
                    interfaces.push(shape_interface_from_cache(&shape, shape_name, shape_without, model));
                }
            }
        }
        Self { interfaces, enums }
    }
}

fn shape_interface_from_cache(shape: &Shape, shape_name: &String, shape_without: &Option<String>, model: &Model) -> Interface {
    let name = model.name().to_owned() + shape_name + &if let Some(without) = shape_without {
        Cow::Owned("Without".to_owned() + without.as_str())
    } else {
        Cow::Borrowed("")
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
        fields: shape.iter().map(|(name, field)| {
            Field {
                title: name.to_title_case(),
                desc: field.desc(),
                name: name.clone(),
                r#type: field.r#type().clone(),
            }
        }).collect(),
        synthesized: Some((shape_name.clone(), shape_without.clone())),
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