use std::fmt::Debug;

pub fn assert<T>(v: T) -> Assert<T> {
    Assert { v }
}

pub struct Assert<T> {
    v: T,
}

impl Assert<bool> {
    #[track_caller]
    pub fn is_true(self) {
        assert!(self.v);
    }
    #[track_caller]
    pub fn is_false(self) {
        assert!(!self.v);
    }
}

impl<T: Debug> Assert<T> {
    #[track_caller]
    pub fn equals<U: Debug>(self, other: U)
    where
        T: PartialEq<U>,
    {
        assert_eq!(self.v, other);
    }
    #[track_caller]
    pub fn not_equals<U: Debug>(self, other: U)
    where
        T: PartialEq<U>,
    {
        assert_ne!(self.v, other);
    }
}

impl<T: AsRef<str>> Assert<T> {
    #[track_caller]
    pub fn contains(self, other: &str) {
        assert!(self.v.as_ref().contains(other), "pattern '{other}' not found in string: {}", self.v.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_bool() {
        assert(true).is_true();
        assert(false).is_false();
    }

    #[test]
    fn assert_equal() {
        assert(1).equals(1);
        assert(2).not_equals(1);
    }

    #[test]
    fn assert_str() {
        assert("uwu owo").contains("uwu");
        assert("uwu owo".to_owned()).contains("uwu");
    }
}
