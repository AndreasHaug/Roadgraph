use serde_json::Value;

pub fn read_file(file: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(file).expect("Could not read file"))
        .expect("Could not parse as json")
}
