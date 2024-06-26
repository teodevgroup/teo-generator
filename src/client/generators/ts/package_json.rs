use std::path::Path;
use inflector::Inflector;
use serde_json::{json, Value};

pub(crate) fn generate_package_json(path: &Path) -> String {
    let name = path.file_name().unwrap().to_str().unwrap().to_kebab_case();
    let version = "0.1.0";
    let json = json!({
        "name": name,
        "version": version,
        "private": true,
        "description": "This package is generated by TEO.",
        "main": "src/index.js",
        "types": "src/index.d.ts",
        "type": "module",
        "files": ["src/**/*"],
        "dependencies": {
            "decimal.js": "^10.4.3"
        },
        "devDependencies": {
            "ts-node": "^10.9.2",
            "typescript": "^5.3.3"
        }
    });
    serde_json::to_string_pretty(&json).unwrap() + "\n"
}

pub(crate) fn updated_package_json_for_existing_project(content: String) -> String {
    let mut json_value: Value = serde_json::from_str(&content).unwrap();
    if let Some(dependencies) = json_value.get_mut("dependencies") {
        if dependencies.get("decimal.js").is_none() {
            dependencies.as_object_mut().unwrap().insert("decimal.js".to_owned(), Value::String("^10.4.3".to_owned()));
        }
    } else {
        json_value.as_object_mut().unwrap().insert("dependencies".to_owned(), json!({
            "decimal.js": "^10.4.3"
        }));
    }
    serde_json::to_string_pretty(&json_value).unwrap() + "\n"
}