use teo_runtime::config::client::Client;
use teo_runtime::namespace::Namespace;

pub(in crate::client) struct Ctx<'a> {
    pub(in crate::client) conf: &'a Client,
    pub(in crate::client) main_namespace: &'a Namespace,
}

impl<'a> Ctx<'a> {

    pub(in crate::client) fn new(conf: &'a Client, main_namespace: &'a Namespace) -> Self {
        Self {
            conf, main_namespace,
        }
    }
}
