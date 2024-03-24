//! Create a Mouser BOM for the Schema.
//!
//! # Example:
//!
//! use elektron::sexp::Schema;
//! use elektron::reports::bom::bom;
//! use elektron::reports::mouser::mouser;
//!
//! let schema = Schema::load("files/summe/summe.kicad_sch").unwrap();
//! let result = bom(&schema, true, Some(String::from("files/partlist.yaml"))).unwrap();
//! mouser("target/mouser-bom.xls", &result.0).unwrap();
use std::collections::HashMap;

use crate::{bom::BomItem, Error};
use xlsxwriter::{Workbook, Worksheet};

use colored::*;

fn set_new_max_width(col: u16, new: usize, width_map: &mut HashMap<u16, usize>) {
    match width_map.get(&col) {
        Some(max) => {
            if new > *max {
                width_map.insert(col, new);
            }
        }
        None => {
            width_map.insert(col, new);
        }
    };
}

static ROWS: [&str; 28] = [
    "Mfr Part Number (Input)",
    "Manufacturer Part Number",
    "Mouser Part Number",
    "Manufacturer Name",
    "Description",
    "Quantity 1",
    "Unit Price 1",
    "Quantity 2",
    "Unit Price 2",
    "Quantity 3",
    "Unit Price 3",
    "Quantity 4",
    "Unit Price 4",
    "Quantity 5",
    "Unit Price 5",
    "Order Quantity",
    "Order Unit Price",
    "Min./Mult.",
    "Availability",
    "Lead Time in Days",
    "Lifecycle",
    "NCNR",
    "RoHS",
    "Pb Free",
    "Package Type",
    "Datasheet URL",
    "Product Image",
    "Design Risk",
];

fn create_headers(sheet: &mut Worksheet, width_map: &mut HashMap<u16, usize>) {
    for (column, row) in ROWS.iter().enumerate() {
        let _ = sheet.write_string(0, column as u16, row, None);
    }

    for (column, row) in ROWS.iter().enumerate() {
        set_new_max_width(column as u16, row.len(), width_map);
    }
}

fn add_row(row: u32, item: &BomItem, sheet: &mut Worksheet) {
    sheet.write_string(row, 2, &item.mouser_nr, None).unwrap();
    sheet.write_string(row, 3, &item.value, None).unwrap();
    sheet.write_string(row, 4, &item.description, None).unwrap();
    sheet.write_string(row, 24, &item.footprint, None).unwrap();
    sheet.write_string(row, 25, &item.datasheet, None).unwrap();
    sheet
        .write_number(row, 5, item.amount as f64, None)
        .unwrap();
}

/// Create the Mouser BOM for a Schema.
///
/// # Arguments
///
/// * `file`     - Output filename.
/// * `bom`      - The BOM list.
pub fn mouser(file: &str, bom: &Vec<BomItem>) -> Result<(), Error> {
    let workbook = Workbook::new(file).unwrap();
    let mut sheet = workbook.add_worksheet(None).unwrap();
    let mut width_map: HashMap<u16, usize> = HashMap::new();
    create_headers(&mut sheet, &mut width_map);

    let mut row = 1;
    for item in bom {
        if !item.mouser_nr.is_empty() {
            add_row(row, item, &mut sheet);
            row += 1;
        } else {
            println!(
                "{} mouser nr not found: '{}:{}'",
                "Warning:".yellow(),
                item.references.join(" ").bold(),
                item.value.bold()
            );
        }
    }

    workbook.close().unwrap();

    Ok(())
}
