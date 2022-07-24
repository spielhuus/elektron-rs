use crate::sexp::Sexp;

/// Test if noode contains Node by name or value.
pub trait Test<T> {
    fn has(&self, index: T) -> bool;
    fn contains(&self, index: T) -> bool;
}
/// Get the value as String by index.
impl Test<&str> for Sexp {
    fn has(&self, value: &str) -> bool {
        if let Sexp::Node(_, values) = &self {
            for v in values {
                if let Sexp::Value(val) = v {
                    if *val == value {
                        return true;
                    }
                } else if let Sexp::Text(text) = v {
                    if *text == value {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn contains(&self, key: &str) -> bool {
        if let Sexp::Node(_, values) = &self {
            for v in values {
                if let Sexp::Node(name, _) = v {
                    if *name == key {
                        return true;
                    }
                }
            }
        }
        false
    }
}
