use teo_runtime::namespace::Namespace;
use crate::utils::lookup::Lookup;

pub(in crate::entity) struct Outline {

}

impl Outline {

    pub fn new<L>(main_namespace: &Namespace, lookup: L) -> Self where L: Lookup {
        Self { }
    }
}