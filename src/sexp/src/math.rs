use std::collections::HashMap;

use lazy_static::lazy_static;
use ndarray::{arr1, arr2, s, Array, Array1, Array2};

use crate::{
    {el, utils, Sexp, SexpValueQuery, SexpValuesQuery},
    error::Error,
};

macro_rules! round {
    ($val: expr) => {
        $val.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    };
}

lazy_static! {
    pub static ref MIRROR: HashMap<String, Array2<f64>> = HashMap::from([
        (String::from(""), arr2(&[[1., 0.], [0., -1.]])),
        (String::from("x"), arr2(&[[1., 0.], [0., 1.]])),
        (String::from("y"), arr2(&[[-1., 0.], [0., -1.]])),
    ]);
}

pub fn normalize_angle(angle: f64) -> f64 {
    if angle > 360.0 {
        angle - 360.0
    } else if angle < 0.0 {
        angle + 360.0
    } else {
        angle
    }
}

pub struct Shape {}

impl Shape {
    pub fn pin_angle(symbol: &Sexp, pin: &Sexp) -> f64 {
        let mut angle = normalize_angle(utils::angle(pin).unwrap() + utils::angle(symbol).unwrap());
        let mirror: Option<String> = symbol.value("mirror");
        if let Some(mirror) = mirror {
            if mirror == "x" && angle == 90.0 {
                angle = 270.0
            } else if mirror == "x" && angle == 270.0 {
                angle = 90.0
            } else if mirror == "y" && angle == 0.0 {
                angle = 180.0
            } else if mirror == "y" && angle == 180.0 {
                angle = 0.0
            }
        }
        angle
    }
}

/// transform the coordinates to absolute values.
pub trait Transform<U, T> {
    fn transform(node: &U, pts: &T) -> T;
}
impl Transform<Sexp, Array2<f64>> for Shape {
    fn transform(symbol: &Sexp, pts: &Array2<f64>) -> Array2<f64> {
        let symbol_pos = utils::at(symbol).unwrap();
        let angle = utils::angle(symbol).unwrap();
        let mirror: Option<String> = if let Some(mirror) = symbol.query(el::MIRROR).next() {
            mirror.get(0)
        } else {
            None
        };

        let theta = -angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array2<f64> = pts.dot(&rot);
        verts = if let Some(mirror) = &mirror {
            verts.dot(MIRROR.get(mirror).unwrap())
        } else {
            verts.dot(MIRROR.get(&String::new()).unwrap())
        };
        let verts = &symbol_pos + verts;
        verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    }
}
impl Transform<Sexp, Array1<f64>> for Shape {
    fn transform(symbol: &Sexp, pts: &Array1<f64>) -> Array1<f64> {
        /* let symbol_pos = utils::at(symbol).unwrap();
        let angle = utils::angle(symbol).unwrap(); */
        let symbol_at = symbol.query(el::AT).next().unwrap();
        let symbol_x: f64 = symbol_at.get(0).unwrap();
        let symbol_y: f64 = symbol_at.get(1).unwrap();
        let symbol_pos = arr1(&[symbol_x, symbol_y]);

        let angle: f64 = symbol.query(el::AT).next().unwrap().get(2).unwrap();
        let mirror: Option<String> = if let Some(mirror) = symbol.query(el::MIRROR).next() {
            mirror.get(0)
        } else {
            None
        };

        let theta = -angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = pts.dot(&rot);
        verts = if let Some(mirror) = mirror {
            verts.dot(MIRROR.get(&mirror).unwrap())
        } else {
            verts.dot(MIRROR.get(&String::new()).unwrap())
        };
        let verts = &symbol_pos + verts;
        verts.mapv_into(|v| {
            let res = format!("{:.3}", v).parse::<f64>().unwrap(); //TODO: use global round macro!
            if res == -0.0 {
                0.0
            } else {
                res
            }
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
        let mut array = Vec::new();
        let mut rows: usize = 0;
        // for symbol in &libs.symbols {
        for symbol in libs.query(el::SYMBOL) {
            let lib_unit = utils::unit_number(symbol.get(0).unwrap());
            let unit: usize = self.value(el::SYMBOL_UNIT).unwrap();
            if unit == lib_unit || lib_unit == 0 {
                for graph in symbol.nodes() {
                    if graph.name == "polyline" {
                        for pt in graph.query("pts") {
                            for xy in pt.query("xy") {
                                array.extend_from_slice(&[xy.get(0).unwrap(), xy.get(1).unwrap()]);
                                rows += 1;
                            }
                        }

                        /* for row in polyline.pts.rows() {
                            let x = row[0];
                            let y = row[1];
                            array.extend_from_slice(&[x, y]);
                            rows += 1;
                        } */
                    } else if graph.name == "rectangle" {
                        let start: Vec<f64> = graph.query("start").next().unwrap().values();
                        let end: Vec<f64> = graph.query("end").next().unwrap().values();
                        array.extend_from_slice(&[start[0], start[1]]);
                        array.extend_from_slice(&[end[0], end[1]]);
                        rows += 2;
                    } else if graph.name == "circle" {
                        let center: Array1<f64> = graph.value("center").unwrap();
                        let radius: f64 = graph.value("radius").unwrap();

                        array.extend_from_slice(&[center[0] - radius, center[1] - radius]);
                        array.extend_from_slice(&[center[0] + radius, center[1] + radius]);
                        rows += 2;
                    } else if graph.name == "arc" {
                        let start: Array1<f64> = graph.query("start").next().unwrap().values();
                        let mid: Array1<f64> = graph.query("mid").next().unwrap().values();
                        let end: Array1<f64> = graph.query("end").next().unwrap().values();

                        array.extend_from_slice(&[start[0], start[1]]);
                        array.extend_from_slice(&[end[0], end[1]]);
                        array.extend_from_slice(&[mid[0], mid[1]]);
                        rows += 3;
                    }
                }
                for pin in symbol.query(el::PIN) {
                    let at = utils::at(pin).unwrap();
                    array.extend_from_slice(&[at[0], at[1]]);
                    rows += 1;
                }
            }
        }
        if rows > 0 {
            let array = Array::from_shape_vec((rows, 2), array).unwrap();
            let axis1 = array.slice(s![.., 0]);
            let axis2 = array.slice(s![.., 1]);
            boundery = arr2(&[
                [
                    *axis1
                        .iter()
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                    *axis2
                        .iter()
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        //.min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                ],
                [
                    *axis1
                        .iter()
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                    *axis2
                        .iter()
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap(),
                ],
            ]);
        }
        Ok(boundery)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PinOrientation {
    Left,
    Right,
    Up,
    Down,
}

impl PinOrientation {
    pub fn from(symbol: &Sexp, pin: &Sexp) -> Self {
        let mut position: Vec<usize> = vec![0; 4];
        let symbol_shift: usize = (utils::angle(symbol).unwrap() / 90.0).round() as usize;

        let lib_pos: usize = (utils::angle(pin).unwrap() / 90.0).round() as usize;
        position[lib_pos] += 1;

        position.rotate_right(symbol_shift);
        let mirror: Option<String> = symbol.value("mirror");
        if let Some(mirror) = &mirror {
            if mirror == "x" {
                position = vec![position[0], position[3], position[2], position[1]];
            } else if mirror == "y" {
                position = vec![position[2], position[1], position[0], position[3]];
            }
        }
        if position == vec![1, 0, 0, 0] {
            Self::Left
        } else if position == vec![0, 1, 0, 0] {
            Self::Down
        } else if position == vec![0, 0, 1, 0] {
            Self::Right
        } else if position == vec![0, 0, 0, 1] {
            Self::Up
        } else {
            panic!("unknown pin position: {:?}", position);
        }
    }
}

pub struct MathUtils {}

impl MathUtils {
    ///calculate vector end pos from start, langth and angle.
    pub fn projection(point: &Array1<f64>, angle: f64, length: f64) -> Array1<f64> {
        round!(arr1(&[
            point[0] + length * angle.to_radians().cos(),
            point[1] + length * angle.to_radians().sin(),
        ]))
    }
}


const M_SQRT1_2: f64 =	0.70710678118654752440;	/* 1/sqrt(2) */

///Utils for the Arc coordinates.
pub trait CalcArc {
    ///The Arc radius.
    fn radius(&self) -> f64;
    ///Get the Arc center position.
    fn center(&self) -> Array1<f64>;
    ///Calculate the start angle.
    fn start_angle(&self) -> f64;
    ///Calculate the end angle.
    fn end_angle(&self) -> f64;
}

impl CalcArc for Sexp {
    fn radius(&self) -> f64 {
        let center = self.center();
        let start: Array1<f64> = self.value("start").unwrap();
        (center[0] - start[0]).hypot(center[1] - start[1])
    }
    fn center(&self) -> Array1<f64> {
        //translated from kicad trigo.cpp
        let start: Array1<f64> = self.value("start").unwrap();
        let mid: Array1<f64> = self.value("mid").unwrap();
        let end: Array1<f64> = self.value("end").unwrap();
        let y_delta_21 = mid[1] - start[1];
        let mut x_delta_21 = mid[0] - start[0];
        let y_delta_32 = end[1] - mid[1];
        let mut x_delta_32 = end[0] - mid[0];

        // This is a special case for aMid as the half-way point when aSlope = 0 and bSlope = inf
        // or the other way around.  In that case, the center lies in a straight line between
        // aStart and aEnd
        if ( ( x_delta_21 == 0.0 ) && ( y_delta_32 == 0.0 ) ) ||
           ( ( y_delta_21 == 0.0 ) && ( x_delta_32 == 0.0 ) )
        {
            return arr1(&[(start[0] + end[0] ) / 2.0,
                          (start[1] + end[1] ) / 2.0]);
        }

        // Prevent div=0 errors
        if x_delta_21 == 0.0 {
            x_delta_21 = std::f64::EPSILON;
        }

        if x_delta_32 == 0.0 {
            x_delta_32 = -std::f64::EPSILON;
        }

        let mut a_slope = y_delta_21 / x_delta_21;
        let mut b_slope = y_delta_32 / x_delta_32;

        let da_slope = a_slope * (0.5 / y_delta_21).hypot(0.5 / x_delta_21);
        let db_slope = b_slope * (0.5 / y_delta_32).hypot(0.5 / x_delta_32);

        if a_slope == b_slope {
            if start == end  {
                // This is a special case for a 360 degrees arc.  In this case, the center is halfway between
                // the midpoint and either end point
                return arr1 (&[(start[0] + mid[0]) / 2.0,
                               (start[1] + mid[1]) / 2.0]);
            } else {
                // If the points are colinear, the center is at infinity, so offset
                // the slope by a minimal amount
                // Warning: This will induce a small error in the center location
                a_slope += std::f64::EPSILON;
                b_slope -= std::f64::EPSILON;
            }
        }

        // Prevent divide by zero error
        if a_slope == 0.0 {
            a_slope = std::f64::EPSILON;
        }
        // What follows is the calculation of the center using the slope of the two lines as well as
        // the propagated error that occurs when rounding to the nearest nanometer.  The error can be
        // ±0.5 units but can add up to multiple nanometers after the full calculation is performed.
        // All variables starting with `d` are the delta of that variable.  This is approximately equal
        // to the standard deviation.
        // We ignore the possible covariance between variables.  We also truncate our series expansion
        // at the first term.  These are reasonable assumptions as the worst-case scenario is that we
        // underestimate the potential uncertainty, which would potentially put us back at the status quo
        let ab_slope_start_end_y = a_slope * b_slope * ( start[1] - end[1] );
        let dab_slope_start_end_y = ab_slope_start_end_y * ( ( da_slope / a_slope * da_slope / a_slope )
                                                               + ( db_slope / b_slope * db_slope / b_slope )
                                                               + ( M_SQRT1_2 / ( start[1] - end[1] )
                                                                   * M_SQRT1_2 / ( start[1] - end[1] ) ) ).sqrt();

        let b_slope_start_mid_x = b_slope * ( start[0] + mid[0] );
        let db_slope_start_mid_x = b_slope_start_mid_x * ( ( db_slope / b_slope * db_slope / b_slope )
                                                             + ( M_SQRT1_2 / ( start[0] + mid[0] )
                                                                     * M_SQRT1_2 / ( start[0] + mid[0] ) ) ).sqrt();

        let a_slope_mid_end_x = a_slope * ( mid[0] + end[0] );
        let da_slope_mid_end_x = a_slope_mid_end_x * ( ( da_slope / a_slope * da_slope / a_slope )
                                                         + ( M_SQRT1_2 / ( mid[0] + end[0] )
                                                                 * M_SQRT1_2 / ( mid[0] + end[0] ) ) ).sqrt();

        let twice_ba_slope_diff = 2.0 * ( b_slope - a_slope );
        let d_twice_ba_slope_diff = 2.0 * ( db_slope * db_slope + da_slope * da_slope ).sqrt();

        let center_numerator_x = ab_slope_start_end_y + b_slope_start_mid_x - a_slope_mid_end_x;
        let d_center_numerator_x = ( dab_slope_start_end_y * dab_slope_start_end_y
                                           + db_slope_start_mid_x * db_slope_start_mid_x
                                           + da_slope_mid_end_x * da_slope_mid_end_x ).sqrt();

        let center_x = ( ab_slope_start_end_y + b_slope_start_mid_x - a_slope_mid_end_x ) / twice_ba_slope_diff;

        let d_center_x = center_x * ( ( d_center_numerator_x / center_numerator_x * d_center_numerator_x / center_numerator_x )
                                             + ( d_twice_ba_slope_diff / twice_ba_slope_diff * d_twice_ba_slope_diff / twice_ba_slope_diff ) ).sqrt();


        let center_numerator_y = ( start[0] + mid[0] ) / 2.0 - center_x;
        let d_center_numerator_y = ( 1.0 / 8.0 + d_center_x * d_center_x ).sqrt();

        let center_first_term = center_numerator_y / a_slope;
        let dcenter_first_term_y = center_first_term * (
                                              ( d_center_numerator_y/ center_numerator_y * d_center_numerator_y / center_numerator_y )
                                            + ( da_slope / a_slope * da_slope / a_slope ) ).sqrt();

        let center_y = center_first_term + ( start[1] + mid[1] ) / 2.0;
        let d_center_y = ( dcenter_first_term_y * dcenter_first_term_y + 1.0 / 8.0 ).sqrt();

        let rounded_100_center_x = ( ( center_x + 50.0 ) / 100.0 ).floor() * 100.0;
        let rounded_100_center_y = ( ( center_y + 50.0 ) / 100.0 ).floor() * 100.0;
        let rounded_10_center_x = ( ( center_x + 5.0 ) / 10.0 ).floor() * 10.0;
        let rounded_10_center_y = ( ( center_y + 5.0 ) / 10.0 ).floor() * 10.0;

        // The last step is to find the nice, round numbers near our baseline estimate and see if they are within our uncertainty
        // range.  If they are, then we use this round value as the true value.  This is justified because ALL values within the
        // uncertainty range are equally true.  Using a round number will make sure that we are on a multiple of 1mil or 100nm
        // when calculating centers.
        if ( rounded_100_center_x - center_x ).abs() < d_center_x && ( rounded_100_center_y - center_y ).abs() < d_center_y {
            arr1(&[rounded_100_center_x, rounded_100_center_y])
        } else if ( rounded_10_center_x - center_x ).abs() < d_center_x && ( rounded_10_center_y - center_y ).abs() < d_center_y {
            arr1(&[rounded_10_center_x, rounded_10_center_y])
        } else {
            arr1(&[center_x, center_y])
        }
    }
    fn start_angle(&self) -> f64 {
        let start: Array1<f64> = self.value("start").unwrap();
        let center = self.center();
        normalize_angle(
            (start[1] - center[1])
                .atan2(start[0] - center[0])
                .to_degrees(),
        )
    }
    fn end_angle(&self) -> f64 {
        let end: Array1<f64> = self.value("end").unwrap();
        let center = self.center();
        normalize_angle(
            (end[1] - center[1])
                .atan2(end[0] - center[0])
                .to_degrees(),
        )
    }
}





#[cfg(test)]
mod tests {
    /* use ndarray::{arr1, s, Array1};

    use crate::{el, utils, Sexp, SexpParser, SexpProperty, SexpTree, SexpValueQuery};

    use super::{Shape, Transform}; */

    /* #[test]
    fn shape_opamp_a() {
        let doc = Schema::load("files/opamp.kicad_sch").unwrap();
        let symbol = doc.get_symbol("U1", 1).unwrap();
        let lib_symbol = doc.get_library("Amplifier_Operational:TL072").unwrap();
        let size = symbol.bounds(lib_symbol).unwrap();
        assert_eq!(arr2(&[[-7.62, -5.08], [7.62, 5.08]]), size)
    }
    #[test]
    fn shape_opamp_c() {
        let doc = Schema::load("files/opamp.kicad_sch").unwrap();
        let symbol = doc.get_symbol("U1", 3).unwrap();
        let lib_symbol = doc.get_library("Amplifier_Operational:TL072").unwrap();
        let size = symbol.bounds(lib_symbol).unwrap();
        assert_eq!(arr2(&[[-2.54, -7.62], [-2.54, 7.62]]), size)
    }
    #[test]
    fn shape_r() {
        let doc = Schema::load("files/opamp.kicad_sch").unwrap();
        let symbol = doc.get_symbol("R1", 1).unwrap();
        let lib_symbol = doc.get_library("Device:R").unwrap();
        let size = symbol.bounds(lib_symbol).unwrap();
        assert_eq!(arr2(&[[-1.016, -3.81], [1.016, 3.81]]), size)
    }
    #[test]
    fn calc_arc() {
        /* (arc (start 0 0.508) (mid -0.508 0) (end 0 -0.508)
            (stroke (width 0.1524) (type default) (color 0 0 0 0))
            (fill (type none))
        ) */

        let arc: Arc = Arc {
            start: arr1(&[0.0, 0.508]),
            mid: arr1(&[-0.508, 0.0]),
            end: arr1(&[0.0, -0.508]),
            stroke: Stroke::new(),
            fill_type: String::new(),
        };
        assert_eq!(0.508, arc.radius());
        assert_eq!(arr1(&[0.0, 0.0]), arc.center());
        assert_eq!(90.0, arc.start_angle());
        assert_eq!(270.0, arc.end_angle());
    }
    #[test]
    fn calc_arc_center1() {
        /* (arc (start 0 0.508) (mid -0.508 0) (end 0 -0.508)
            (stroke (width 0.1524) (type default) (color 0 0 0 0))
            (fill (type none))
        ) */

        let arc: Arc = Arc {
            start: arr1(&[38.1, -69.85]),
            mid: arr1(&[31.75, -63.5]),
            end: arr1(&[25.4, -69.85]),
            stroke: Stroke::new(),
            fill_type: String::new(),
        };
        assert_eq!(arr1(&[31.75, -69.85]), arc.center());
    }
    #[test]
    fn calc_arc_center2() {
        let arc: Arc = Arc {
            start: arr1(&[-44196.0, -38100.0]),
            mid: arr1(&[-32033.0, 0.0]),
            end: arr1(&[-44196.0, 38100.0]),
            stroke: Stroke::new(),
            fill_type: String::new(),
        };
        assert_eq!(arr1(&[-97787.6891803009, 0.0]), arc.center());
    }
    #[test]
    fn test_normalize_angle() {
        assert_eq!(270.0, normalize_angle(-90.0));
        assert_eq!(90.0, normalize_angle(450.0));
        assert_eq!(180.0, normalize_angle(180.0));
    }
    #[test]
    fn test_vector_distance() {
        assert_eq!(arr1(&[10.0, 0.0]), MathUtils::projection(&arr1(&[0.0, 0.0]), 0.0, 10.0));
        assert_eq!(arr1(&[0.0, 10.0]), MathUtils::projection(&arr1(&[0.0, 0.0]), 90.0, 10.0));
        assert_eq!(arr1(&[-10.0, 0.0]), MathUtils::projection(&arr1(&[0.0, 0.0]), 180.0, 10.0));
    } */
    /* #[test]
    fn test_angle_to_segment_count() {
        assert_eq!(14, MathUtils::arc_to_segment_count(200000.0, 5000, 360.0));
        assert_eq!(21, MathUtils::arc_to_segment_count(450000.0, 5000, 360.0));
    } */
}
