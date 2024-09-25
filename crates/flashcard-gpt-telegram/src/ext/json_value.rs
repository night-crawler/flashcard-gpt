use serde_json::Value;

pub trait ValueExt {
    fn get_str_by(&self, key: &str) -> Option<&Value>;
}

impl ValueExt for Value {
    fn get_str_by(&self, key: &str) -> Option<&Value> {
        if let Value::Object(obj) = self {
            return obj.get(key);
        }
        None
    }
}
