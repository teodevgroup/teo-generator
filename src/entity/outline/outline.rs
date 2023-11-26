use inflector::Inflector;
use indexmap::indexmap;
use teo_parser::r#type::synthesized_enum::SynthesizedEnum;
use teo_parser::r#type::synthesized_shape::SynthesizedShape;
use teo_parser::r#type::Type;
use teo_runtime::model::field::typed::Typed;
use teo_runtime::model::Model;
use teo_runtime::namespace::Namespace;
use teo_runtime::traits::documentable::Documentable;
use teo_runtime::traits::named::Named;
use crate::entity::outline::interface::{Field, Interface};
use crate::entity::outline::r#enum::{Enum, Member};

#[derive(Copy, Clone, PartialEq, Eq)]
pub(in crate::entity) enum Mode {
    Client,
    Entity,
}

pub(in crate::entity) struct Outline {
    interfaces: Vec<Interface>,
    enums: Vec<Enum>,
}

impl Outline {

    pub fn new(namespace: &Namespace, mode: Mode) -> Self {
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
            if (mode == Mode::Entity && model.generate_entity) || (mode == Mode::Client && model.generate_client) {
                for ((shape_name, shape_without), input) in &model.cache.shape.shapes {
                    if let Some(shape) = input.as_synthesized_shape() {
                        interfaces.push(shape_interface_from_cache(shape, &shape_name.to_string(), shape_without, Some(model)));
                    } else if let Some(union) = input.as_union() {
                        let shape = make_shape_from_union(union);
                        interfaces.push(shape_interface_from_cache(&shape, &shape_name.to_string(), shape_without, Some(model)));
                    }
                }
                for (enum_name, input) in &model.cache.shape.enums {
                    enums.push(shape_enum_from_cache(input, &enum_name.to_string(), model));
                }
            }
        }
        Self { interfaces, enums }
    }

    pub(in crate::entity) fn interfaces(&self) -> &Vec<Interface> {
        &self.interfaces
    }

    pub(in crate::entity) fn enums(&self) -> &Vec<Enum> {
        &self.enums
    }
}

fn shape_interface_from_cache(shape: &SynthesizedShape, shape_name: &String, shape_without: &Option<String>, model: Option<&Model>) -> Interface {
    let name = if let Some(model) = model {
        if let Some(without) = shape_without {
            model.name().to_owned() + shape_name.as_str().strip_suffix("Input").unwrap() + "Without" + &without.to_pascal_case() + "Input"
        } else {
            model.name().to_owned() + shape_name
        }
    } else {
        shape_name.to_owned()
    };
    Interface {
        title: name.to_sentence_case(),
        desc: "This synthesized interface doesn't have a description".to_owned(),
        path: if let Some(model) = model {
            let mut result = model.path.clone();
            result.pop();
            result.push(name.clone());
            result
        } else {
            vec![shape_name.to_owned()]
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
