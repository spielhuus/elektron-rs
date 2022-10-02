mod library;
pub mod model;
mod parser;
pub mod pcb;
pub mod schema;
mod shape;
mod write;

pub use self::library::Library;
pub use self::parser::{SexpParser, State};
pub use self::schema::Schema;
pub use self::shape::{Bounds, Shape, Transform};


macro_rules! uuid {
    () => {
        Uuid::new_v4().to_string()
    };
}
pub(crate) use uuid;
