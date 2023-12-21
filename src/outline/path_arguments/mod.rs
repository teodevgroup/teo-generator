use regex::Regex;

pub(crate) struct PathArguments {
    pub(in super::super) name: String,
    pub(in super::super) items: Vec<String>,
}

impl PathArguments {

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn items(&self) -> Vec<&str> {
        self.items.iter().map(AsRef::as_ref).collect()
    }

    pub(crate) fn fetch_items(string: Option<&String>) -> Vec<String> {
        if let Some(string) = string {
            let regex = Regex::new("(:|\\*)(\\w+)").unwrap();
            let mut results = vec![];
            for result in regex.find_iter(string) {
                results.push(result.as_str()[1..].to_string());
            }
            results
        } else {
            vec![]
        }
    }
}