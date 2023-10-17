pub mod error;
mod cells;
mod parser;
mod plot;
mod runner;
mod utils;

pub use self::runner::Document;

pub extern crate pest;

#[macro_use]
pub extern crate pest_derive;

use crate::error::Error;

fn check_directory(filename: &str) -> Result<(), Error> {
    let path = std::path::Path::new(filename);
    let parent = path.parent();
    if let Some(parent) = parent {
        if parent.to_str().unwrap() != "" && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}
