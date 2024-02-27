use regex::Regex;

pub fn updated_pubspec_yaml_for_existing_project(mut yaml_data: String) -> String {
    let dependencies_regex = Regex::new("^dependencies\\s*:|\ndependencies\\s*:").unwrap();
    if let Some(mdata) = dependencies_regex.find(yaml_data.as_str()) {
        println!("see {:?}", mdata);
    } else {
        yaml_data += r#"
dependencies:
  http: ^0.13.5
  json_annotation: ^4.8.0"#;
    }
    let dev_dependencies_regex = Regex::new("^dev_dependencies\\s*:|\ndev_dependencies\\s*:").unwrap();
    if let Some(mdata) = dev_dependencies_regex.find(yaml_data.as_str()) {
        println!("see {:?}", mdata);
    } else {
        yaml_data += r#"
dev_dependencies:
  build_runner: ^2.3.3
  json_serializable: ^6.6.1"#;
    }
    yaml_data
}