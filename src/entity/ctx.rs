use teo_runtime::config::entity::Entity;
use teo_runtime::namespace::Namespace;
use crate::outline::outline::Outline;

pub(crate) struct Ctx<'a> {
    pub(crate) conf: &'a Entity,
    pub(crate) main_namespace: &'a Namespace,
}

impl<'a> Ctx<'a> {

    pub(crate) fn new(conf: &'a Entity, main_namespace: &'a Namespace) -> Self {
        Self {
            main_namespace,
            conf,
        }
    }
}
