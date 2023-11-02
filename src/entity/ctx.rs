use teo_runtime::config::entity::Entity;
use teo_runtime::namespace::Namespace;
use crate::entity::outline::outline::Outline;

pub(in crate::entity) struct Ctx<'a> {
    pub(in crate::entity) conf: &'a Entity,
    pub(in crate::entity) main_namespace: &'a Namespace,
}

impl<'a> Ctx<'a> {

    pub(in crate::entity) fn new(conf: &'a Entity, main_namespace: &'a Namespace) -> Self {
        Self {
            main_namespace,
            conf,
        }
    }
}
