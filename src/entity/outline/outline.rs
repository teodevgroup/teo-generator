use teo_runtime::namespace::Namespace;
use crate::entity::outline::interface::Interface;
use crate::entity::outline::r#enum::Enum;
use crate::utils::lookup::Lookup;

pub(in crate::entity) struct Outline {
    interfaces: Vec<Interface>,
    enums: Vec<Enum>,
}

impl Outline {

    pub fn new<L>(namespace: &Namespace) -> Self {
        let mut interfaces = vec![];
        let mut enums = vec![];
        Self { interfaces, enums }
    }
}