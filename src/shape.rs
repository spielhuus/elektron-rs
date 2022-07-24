use crate::Error;
use crate::sexp::Sexp;
use crate::sexp::test::Test;
use lazy_static::lazy_static;
use ndarray::{arr2, s, Array, Array1, Array2};
use std::collections::HashMap;

use crate::sexp::get_unit;
use crate::sexp::get::{Get, get};

lazy_static! {
    pub static ref MIRROR: HashMap<String, Array2<f64>> = HashMap::from([ //TODO make global
        (String::from(""), arr2(&[[1., 0.], [0., -1.]])),
        (String::from("x"), arr2(&[[1., 0.], [0., 1.]])),
        (String::from("y"), arr2(&[[-1., 0.], [0., -1.]])),
        (String::from("xy"), arr2(&[[0., 0.], [0., 0.]]))
    ]);
}

pub struct Shape {}

/// transform the coordinates to absolute values.
pub trait Transform<T> {
    fn transform(node: &Sexp, pts: &T) -> T;
}
impl Transform<Array2<f64>> for Shape {
    fn transform(node: &Sexp, pts: &Array2<f64>) -> Array2<f64> {
        let pos: Array1<f64> = get!(node, "at").unwrap();
        let angle: f64 = get!(node, "at", 2);
        let mirror: String = if node.contains("mirror") {
            get!(node, "mirror", 0)
        } else {
            String::from("")
        };
        let theta = -angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array2<f64> = pts.dot(&rot);
        verts = verts.dot(MIRROR.get(mirror.as_str()).unwrap());
        let verts = pos + verts;
        verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    }
}
impl Transform<Array1<f64>> for Shape {
    fn transform(node: &Sexp, pts: &Array1<f64>) -> Array1<f64> {
        let pos: Array1<f64> = get!(node, "at").unwrap();
        let angle: f64 = get!(node, "at", 2);
        let mirror: String = if node.contains("mirror") {
            get!(node, "mirror", 0)
        } else {
            String::from("")
        };
        let theta = -angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = pts.dot(&rot);
        verts = verts.dot(MIRROR.get(mirror.as_str()).unwrap());
        let verts = pos + verts;
        verts.mapv_into(|v| {
            let res = format!("{:.2}", v).parse::<f64>().unwrap(); 
            if res == -0.0 { 0.0 } else { res } 
        })
    }
}

/// transform the coordinates to absolute values.
pub trait Bounds<T> {
    fn bounds(&self, libs: &Sexp) -> Result<T, Error>;
}
impl Bounds<Array2<f64>> for Sexp {
    fn bounds(&self, libs: &Sexp) -> Result<Array2<f64>, Error> {
        let mut boundery: Array2<f64> = Array2::default((0, 2));
        let _at: Array1<f64> = get!(self, "at")?;
        let _lib_id: String = get!(self, "lib_id", 0);

        let syms: Vec<&Sexp> = libs.get("symbol")?;
        for symbol in syms {
            if get_unit(self)? == get_unit(symbol)? || get_unit(symbol).unwrap() == 0 {
                let mut array = Vec::new();
                let mut rows: usize = 0;
                if let Sexp::Node(name, values) = symbol {
                    for element in values {
                        if let Sexp::Node(name, values) = element {
                            if name == "polyline" {
                                let pts: Array2<f64> = get!(element, "pts").unwrap();
                                for row in pts.rows() {
                                    let x = row[0].clone();
                                    let y = row[1].clone();
                                    array.extend_from_slice(&[x, y]);
                                    rows += 1;
                                }
                            } else if name == "rectangle" {
                                let start: Array1<f64> = get!(element, "start").unwrap();
                                let end: Array1<f64> = get!(element, "end").unwrap();
                                array.extend_from_slice(&[start[0], start[1]]);
                                array.extend_from_slice(&[end[0], end[1]]);
                                rows += 2;
                            } else if name != "pin" {
                                println!("Unknown: {:?}", name);
                            }
                        }
                    }
                }
                if rows > 0 {
                    let array = Array::from_shape_vec((rows, 2), array).unwrap();
                    let axis1 = array.slice(s![.., 0]);
                    let axis2 = array.slice(s![.., 1]);
                    boundery = arr2(&[
                        [
                            axis1
                                .iter()
                                .min_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap()
                                .clone(),
                            axis2
                                .iter()
                                .min_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap()
                                .clone(),
                        ],
                        [
                            axis1
                                .iter()
                                .max_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap()
                                .clone(),
                            axis2
                                .iter()
                                .max_by(|a, b| a.partial_cmp(b).unwrap())
                                .unwrap()
                                .clone(),
                        ],
                    ]);
                }
            }
        }

        /* let axis1 = arr.slice(s![.., 0]);
        let axis2 = arr.slice(s![.., 1]);
        let boundery = arr2(&[
            [axis1.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
             axis2.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()],
            [axis1.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
             axis2.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()],
        ]); */

        Ok(boundery)
    }
}
