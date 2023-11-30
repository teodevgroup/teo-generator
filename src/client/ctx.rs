use teo_runtime::config::client::Client;
use teo_runtime::namespace::Namespace;
use crate::outline::outline::Outline;

pub(in crate::client) struct Ctx<'a> {
    pub(in crate::client) conf: &'a Client,
    pub(in crate::client) main_namespace: &'a Namespace,
    pub(in crate::client) outline: &'a Outline,
}

impl<'a> Ctx<'a> {

    pub(in crate::client) fn new(conf: &'a Client, main_namespace: &'a Namespace, outline: &'a Outline) -> Self {
        Self {
            conf, main_namespace, outline,
        }
    }
}
