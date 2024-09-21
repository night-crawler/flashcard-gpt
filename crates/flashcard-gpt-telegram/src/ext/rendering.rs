use itertools::Itertools;
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

pub trait DisplayJoinOrDash {
    fn join_or_dash(&self) -> String;
}

impl<I, T> DisplayJoinOrDash for I
where
    T: ToString,
    I: IntoIterator,
    I::Item: fmt::Display,
    for<'a> &'a I: IntoIterator<Item = &'a T>,
{
    fn join_or_dash(&self) -> String {
        let mut iter = self.into_iter().peekable();
        if iter.peek().is_none() {
            "-".to_string()
        } else {
            iter.map(|item| item.to_string()).join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn test_option_display_ext() {
        let some = Some(1);
        let none: Option<i32> = None;

        assert_eq!(some.to_string_or_dash(), "1");
        assert_eq!(none.to_string_or_dash(), "-");
    }

    #[test]
    fn test_display_join_or_dash() {
        let vec = vec![1, 2, 3];
        let empty: Vec<i32> = vec![];

        assert_eq!(vec.join_or_dash(), "1, 2, 3");
        assert_eq!(empty.join_or_dash(), "-");

        let set = BTreeSet::from([1, 2, 3]);
        let empty_set: BTreeSet<i32> = BTreeSet::new();

        assert_eq!(set.join_or_dash(), "1, 2, 3");
        assert_eq!(empty_set.join_or_dash(), "-");
    }
}
