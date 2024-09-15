use std::fmt;

pub trait OptionDisplayExt {
    fn to_string_or_dash(&self) -> String;
}

impl<T: fmt::Display> OptionDisplayExt for Option<T> {
    fn to_string_or_dash(&self) -> String {
        match self {
            Some(value) => value.to_string(),
            None => "-".to_string(),
        }
    }
}

pub trait VecDisplayExt {
    fn join_or_dash(&self) -> String;
}

impl<T: fmt::Display> VecDisplayExt for Vec<T> {
    fn join_or_dash(&self) -> String {
        if self.is_empty() {
            "-".to_string()
        } else {
            self.iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        }
    }
}