use crate::outline::delegate::{GroupItem, NamespaceItem, RequestItem};

pub struct Delegate {
    name: String,
    group_items: Vec<GroupItem>,
    namespace_items: Vec<NamespaceItem>,
    request_items: Vec<RequestItem>,
}

impl Delegate {

    pub fn new(name: String, group_items: Vec<GroupItem>, namespace_items: Vec<NamespaceItem>, request_items: Vec<RequestItem>) -> Self {
        Self { name, group_items, namespace_items, request_items }
    }

    pub fn group_items(&self) -> &Vec<GroupItem> {
        &self.group_items
    }

    pub fn namespace_items(&self) -> &Vec<NamespaceItem> {
        &self.namespace_items
    }

    pub fn request_items(&self) -> &Vec<RequestItem> {
        &self.request_items
    }
}