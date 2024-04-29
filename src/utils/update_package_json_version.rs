use serde_json::Value;

pub(crate) fn update_package_json_version(content: String) -> String {
    let mut json_value: Value = serde_json::from_str(&content).unwrap();
    let version = json_value.get("version");
    match version {
        Some(v) => {
            let previous = v.as_str().unwrap();
            let parts = previous.split(".");
            let last = parts.clone().last().unwrap();
            match last.parse::<u32>() {
                Ok(num) => {
                    let new_last = format!("{}", num + 1);
                    let vec_parts: Vec<&str> = parts.into_iter().collect();
                    let new_version = vec_parts.split_last().unwrap().1.join(".") + "." + &new_last;
                    json_value.as_object_mut().unwrap().insert("version".to_owned(), Value::String(new_version));
                },
                Err(_) => (),
            }
        },
        None => {
            json_value.as_object_mut().unwrap().insert("version".to_owned(), Value::String("0.1.1".to_owned()));
        },
    }
    serde_json::to_string_pretty(&json_value).unwrap() + "\n"
}
