use regex::Regex;

pub fn updated_pubspec_yaml_for_existing_project(mut yaml_data: String) -> String {
    let block_end_matcher = Regex::new("\n(^\\s)|^#").unwrap();
    let dependencies_regex = Regex::new("^dependencies\\s*:|\ndependencies\\s*:").unwrap();
    if let Some(mdata) = dependencies_regex.find(yaml_data.as_str()) {
        let end_position = mdata.end();
        let rest = &yaml_data.as_str()[end_position..];
        let block_end_position = if let Some(end_mdata) = block_end_matcher.find(rest) {
            end_mdata.start() + end_position
        } else {
            yaml_data.len()
        };
        let block_content = &yaml_data.as_str()[end_position..block_end_position];
        let mut to_insert = "".to_owned();
        for (name, version) in [("http", "^0.13.5"), ("json_annotation", "^4.8.0")] {
            let regex = Regex::new(format!("\n\\s+{name}\\s*:").as_str()).unwrap();
            if !regex.is_match(block_content) {
                to_insert += format!("\n  {name}: {version}").as_str();
            }
        }
        if !to_insert.is_empty() {
            yaml_data.insert_str(block_end_position, (to_insert + "\n").as_str());
        }
    } else {
        yaml_data += r#"
dependencies:
  http: ^0.13.5
  json_annotation: ^4.8.0"#;
    }
    let dev_dependencies_regex = Regex::new("^dev_dependencies\\s*:|\ndev_dependencies\\s*:").unwrap();
    if let Some(mdata) = dev_dependencies_regex.find(yaml_data.as_str()) {
        let end_position = mdata.end();
        let rest = &yaml_data.as_str()[end_position..];
        let block_end_position = if let Some(end_mdata) = block_end_matcher.find(rest) {
            end_mdata.start() + end_position
        } else {
            yaml_data.len()
        };
        let block_content = &yaml_data.as_str()[end_position..block_end_position];
        let mut to_insert = "".to_owned();
        for (name, version) in [("build_runner", "^2.3.3"), ("json_serializable", "^6.6.1")] {
            let regex = Regex::new(format!("\n\\s+{name}\\s*:").as_str()).unwrap();
            if !regex.is_match(block_content) {
                to_insert += format!("\n  {name}: {version}").as_str();
            }
        }
        if !to_insert.is_empty() {
            yaml_data.insert_str(block_end_position, (to_insert + "\n").as_str());
        }
    } else {
        yaml_data += r#"
dev_dependencies:
  build_runner: ^2.3.3
  json_serializable: ^6.6.1"#;
    }
    yaml_data
}