use crate::sexp::{Error, SexpNode, SexpType};
use ndarray::Array1;

macro_rules! set {
    ($node:expr, $key:expr, $index:expr, $value:expr) => {
        for v in &mut $node.values {
            if let SexpType::ChildSexpNode(n) = v {
                if n.name == $key {
                    if let SexpType::ChildSexpValue(v) = &mut n.values[$index] {
                        v.value = $value.to_string();
                    }
                }
            }
        }
    };
}

pub(crate) use set;


/// Write to the nodes
pub trait Set<S, T> {
    fn set(&mut self, index: S, value: T) -> Result<(), Error>;
}
/// Get the value as String by index.
impl Set<usize, String> for SexpNode {
    /// Get the value as String by index.
    fn set(&mut self, index: usize, value: String) -> Result<(), Error> {
        match &mut self.values[index] {
            SexpType::ChildSexpValue(n) => {
                n.value = value;
                Ok(())
            }
            SexpType::ChildSexpText(n) => {
                n.value = value;
                Ok(())
            }
            _ => { Ok(()) }
        }
    }
}
impl Set<usize, f64> for SexpNode {
    /// Get the value as String by index.
    fn set(&mut self, index: usize, value: f64) -> Result<(), Error> {
        match &mut self.values[index] {
            SexpType::ChildSexpValue(n) => {
                n.value = value.to_string();
                Ok(())
            }
            SexpType::ChildSexpText(n) => {
                n.value = value.to_string();
                Ok(())
            }
            SexpType::ChildSexpNode(_) => Err(Error::ExpectValueNode),
        }
    }
}
impl Set<&str, Array1<f64>> for SexpNode {
    /// Get the value as String by index.
    fn set(&mut self, key: &str, value: Array1<f64>) -> Result<(), Error> {
        for v in &mut self.values {
            if let SexpType::ChildSexpNode(n) = v {
                if n.name == key {
                    if let SexpType::ChildSexpValue(v) = &mut n.values[0] {
                        v.value = value[0].to_string();
                    }
                    if let SexpType::ChildSexpValue(v) = &mut n.values[1] {
                        v.value = value[1].to_string();
                    }
                    return Ok(());
                }
            }
        }
        Err(Error::ExpectValueNode)
    }
}
