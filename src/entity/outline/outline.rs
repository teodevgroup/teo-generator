use inflector::Inflector;
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
        // put default shapes into main
        if namespace.is_main() {
            let (i, e) = default_shapes_and_enums();
            interfaces.extend(i);
            enums.extend(e);
        }
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
            if (mode == Mode::Entity && model.generate_entity) || (mode == Mode::Client && model.generate_client) {
                for ((shape_name, shape_without), input) in &model.cache.shape.map {
                    if let Some(shape) = input.as_shape() {
                        interfaces.push(shape_interface_from_cache(shape, shape_name, shape_without, Some(model)));
                    } else if let Some(r#enum) = input.as_synthesized_enum() {
                        enums.push(shape_enum_from_cache(&r#enum, shape_name, model));
                    } else if input.is_or() {
                        let shape = input.or_to_shape();
                        interfaces.push(shape_interface_from_cache(&shape, shape_name, shape_without, Some(model)));
                    }
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

fn shape_interface_from_cache(shape: &Shape, shape_name: &String, shape_without: &Option<String>, model: Option<&Model>) -> Interface {
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
        generic_names: if model.is_none() {
            generic_names_for_builtin_shape(shape_name)
        } else { vec![] },
        extends: vec![],
        fields: shape.iter().map(|(name, input)| {
            let r#type = input.as_type().unwrap();
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

fn default_shapes_and_enums() -> (Vec<Interface>, Vec<Enum>) {
    let mut interfaces = vec![];
    let mut enums = vec![];
    for (name, input) in STATIC_TYPES.iter() {
        if let Some(shape) = input.as_shape() {
            interfaces.push(shape_interface_from_cache(shape, name, &None, None));
        } else {
        }
    }
    (interfaces, enums)
}

fn generic_names_for_builtin_shape(shape_name: &String) -> Vec<String> {
    if vec!["EnumFilter", "EnumNullableFilter", "ArrayFilter", "ArrayNullableFilter", "EnumWithAggregatesFilter", "EnumNullableWithAggregatesFilter", "ArrayWithAggregatesFilter", "ArrayNullableWithAggregatesFilter", "ArrayAtomicUpdateOperationInput"].contains(&shape_name.as_str()) {
        vec!["T".to_owned()]
    } else {
        vec![]
    }
}