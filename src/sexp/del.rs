use crate::sexp::{Error, SexpNode, SexpType};

/// Write to the nodes
pub trait Del<S, T> {
    fn delete(&mut self, index: S) -> Result<(), Error>;
}
/// Get the value as String by index.
impl Del<String, String> for SexpNode {
    /// Get the value as String by index.
    fn delete(&mut self, key: String) -> Result<(), Error> {
        self.values.remove(self.values.iter().position(|x| {
            if let SexpType::ChildSexpNode(node) = x {
                *node.name == *key
            } else { false }
        }).unwrap());
        Ok(())
    }
}


