pub fn uuid() -> String { String::from(uuid::Uuid::new_v4()) }

pub fn scan_id() -> String {
    format!("v_{}", uuid().replace('-', "_"))
}