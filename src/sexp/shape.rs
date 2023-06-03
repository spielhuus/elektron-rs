use lazy_static::lazy_static;
use ndarray::{arr1, arr2, s, Array, Array1, Array2};
use std::collections::HashMap;

use crate::error::Error;

const M_SQRT1_2: f64 =	0.70710678118654752440;	/* 1/sqrt(2) */

//TODO: make global
macro_rules! round {
    ($val: expr) => {
        $val.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    };
}

use super::{
    model::{Footprint, Graph, LibrarySymbol, Symbol},
    Arc, Pin,
};

lazy_static! {
    pub static ref MIRROR: HashMap<String, Array2<f64>> = HashMap::from([ //TODO make global
        (String::from(""), arr2(&[[1., 0.], [0., -1.]])),
        (String::from("x"), arr2(&[[1., 0.], [0., 1.]])),
        (String::from("y"), arr2(&[[-1., 0.], [0., -1.]])),
    ]);
}

pub struct Shape {}

impl Shape {
    pub fn pin_angle(symbol: &Symbol, pin: &Pin) -> f64 {
        let mut angle = normalize_angle(pin.angle + symbol.angle);
        if let Some(mirror) = &symbol.mirror {
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
impl Transform<Symbol, Array2<f64>> for Shape {
    fn transform(symbol: &Symbol, pts: &Array2<f64>) -> Array2<f64> {
        let theta = -symbol.angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array2<f64> = pts.dot(&rot);
        verts = if let Some(mirror) = &symbol.mirror {
            verts.dot(MIRROR.get(mirror).unwrap())
        } else {
            verts.dot(MIRROR.get(&String::new()).unwrap())
        };
        let verts = &symbol.at + verts;
        verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    }
}
impl Transform<Symbol, Array1<f64>> for Shape {
    fn transform(symbol: &Symbol, pts: &Array1<f64>) -> Array1<f64> {
        let theta = -symbol.angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let mut verts: Array1<f64> = pts.dot(&rot);
        verts = if let Some(mirror) = &symbol.mirror {
            verts.dot(MIRROR.get(mirror).unwrap())
        } else {
            verts.dot(MIRROR.get(&String::new()).unwrap())
        };
        let verts = &symbol.at + verts;
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
impl Transform<Footprint, Array2<f64>> for Shape {
    fn transform(footprint: &Footprint, pts: &Array2<f64>) -> Array2<f64> {
        let theta = /* TODO - */ footprint.angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let verts: Array2<f64> = pts.dot(&rot);
        //verts = verts.dot(MIRROR.get(&symbol.mirror.join("")).unwrap());
        let verts = &footprint.at + verts;
        verts.mapv_into(|v| format!("{:.2}", v).parse::<f64>().unwrap())
    }
}
impl Transform<Footprint, Array1<f64>> for Shape {
    fn transform(symbol: &Footprint, pts: &Array1<f64>) -> Array1<f64> {
        let theta = /* TODO - */ symbol.angle.to_radians();
        let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
        let verts: Array1<f64> = pts.dot(&rot);
        //verts = verts.dot(MIRROR.get(&symbol.mirror.join("")).unwrap());
        let verts = &symbol.at + verts;
        verts.mapv_into(|v| {
            let res = format!("{:.2}", v).parse::<f64>().unwrap();
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
    fn bounds(&self, libs: &LibrarySymbol) -> Result<T, Error>;
}
impl Bounds<Array2<f64>> for Symbol {
    fn bounds(&self, libs: &LibrarySymbol) -> Result<Array2<f64>, Error> {
        let mut boundery: Array2<f64> = Array2::default((0, 2));
        let mut array = Vec::new();
        let mut rows: usize = 0;
        for symbol in &libs.symbols {
            if self.unit == symbol.unit || symbol.unit == 0 {
                for element in &symbol.graph {
                    match element {
                        Graph::Polyline(polyline) => {
                            for row in polyline.pts.rows() {
                                let x = row[0];
                                let y = row[1];
                                array.extend_from_slice(&[x, y]);
                                rows += 1;
                            }
                        }
                        Graph::Rectangle(rectangle) => {
                            array.extend_from_slice(&[rectangle.start[0], rectangle.start[1]]);
                            array.extend_from_slice(&[rectangle.end[0], rectangle.end[1]]);
                            rows += 2;
                        }
                        Graph::Circle(circle) => {
                            array.extend_from_slice(&[
                                circle.center[0] - circle.radius,
                                circle.center[1] - circle.radius,
                            ]);
                            array.extend_from_slice(&[
                                circle.center[0] + circle.radius,
                                circle.center[1] + circle.radius,
                            ]);
                            rows += 2;
                        }
                        Graph::Arc(arc) => {
                            array.extend_from_slice(&[arc.start[0], arc.start[1]]);
                            array.extend_from_slice(&[arc.end[0], arc.end[1]]);
                            array.extend_from_slice(&[arc.mid[0], arc.mid[1]]);
                            rows += 3;
                        }
                        _ => {} //TODO: implement
                    }
                }
                for pin in &symbol.pin {
                    array.extend_from_slice(&[pin.at[0], pin.at[1]]);
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

impl CalcArc for Arc {
    fn radius(&self) -> f64 {
        let center = self.center();
        (center[0] - self.start[0]).hypot(center[1] - self.start[1])
    }
    fn center(&self) -> Array1<f64> {
        //translated from kicad trigo.cpp
        let y_delta_21 = self.mid[1] - self.start[1];
        let mut x_delta_21 = self.mid[0] - self.start[0];
        let y_delta_32 = self.end[1] - self.mid[1];
        let mut x_delta_32 = self.end[0] - self.mid[0];

        // This is a special case for aMid as the half-way point when aSlope = 0 and bSlope = inf
        // or the other way around.  In that case, the center lies in a straight line between
        // aStart and aEnd
        if ( ( x_delta_21 == 0.0 ) && ( y_delta_32 == 0.0 ) ) ||
           ( ( y_delta_21 == 0.0 ) && ( x_delta_32 == 0.0 ) )
        {
            return arr1(&[(self.start[0] + self.end[0] ) / 2.0,
                          (self.start[1] + self.end[1] ) / 2.0]);
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
            if self.start == self.end  {
                // This is a special case for a 360 degrees arc.  In this case, the center is halfway between
                // the midpoint and either end point
                return arr1 (&[(self.start[0] + self.mid[0]) / 2.0,
                               (self.start[1] + self.mid[1]) / 2.0]);
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
        let ab_slope_start_end_y = a_slope * b_slope * ( self.start[1] - self.end[1] );
        let dab_slope_start_end_y = ab_slope_start_end_y * ( ( da_slope / a_slope * da_slope / a_slope )
                                                               + ( db_slope / b_slope * db_slope / b_slope )
                                                               + ( M_SQRT1_2 / ( self.start[1] - self.end[1] )
                                                                   * M_SQRT1_2 / ( self.start[1] - self.end[1] ) ) ).sqrt();

        let b_slope_start_mid_x = b_slope * ( self.start[0] + self.mid[0] );
        let db_slope_start_mid_x = b_slope_start_mid_x * ( ( db_slope / b_slope * db_slope / b_slope )
                                                             + ( M_SQRT1_2 / ( self.start[0] + self.mid[0] )
                                                                     * M_SQRT1_2 / ( self.start[0] + self.mid[0] ) ) ).sqrt();

        let a_slope_mid_end_x = a_slope * ( self.mid[0] + self.end[0] );
        let da_slope_mid_end_x = a_slope_mid_end_x * ( ( da_slope / a_slope * da_slope / a_slope )
                                                         + ( M_SQRT1_2 / ( self.mid[0] + self.end[0] )
                                                                 * M_SQRT1_2 / ( self.mid[0] + self.end[0] ) ) ).sqrt();

        let twice_ba_slope_diff = 2.0 * ( b_slope - a_slope );
        let d_twice_ba_slope_diff = 2.0 * ( db_slope * db_slope + da_slope * da_slope ).sqrt();

        let center_numerator_x = ab_slope_start_end_y + b_slope_start_mid_x - a_slope_mid_end_x;
        let d_center_numerator_x = ( dab_slope_start_end_y * dab_slope_start_end_y
                                           + db_slope_start_mid_x * db_slope_start_mid_x
                                           + da_slope_mid_end_x * da_slope_mid_end_x ).sqrt();

        let center_x = ( ab_slope_start_end_y + b_slope_start_mid_x - a_slope_mid_end_x ) / twice_ba_slope_diff;

        let d_center_x = center_x * ( ( d_center_numerator_x / center_numerator_x * d_center_numerator_x / center_numerator_x )
                                             + ( d_twice_ba_slope_diff / twice_ba_slope_diff * d_twice_ba_slope_diff / twice_ba_slope_diff ) ).sqrt();


        let center_numerator_y = ( self.start[0] + self.mid[0] ) / 2.0 - center_x;
        let d_center_numerator_y = ( 1.0 / 8.0 + d_center_x * d_center_x ).sqrt();

        let center_first_term = center_numerator_y / a_slope;
        let dcenter_first_term_y = center_first_term * (
                                              ( d_center_numerator_y/ center_numerator_y * d_center_numerator_y / center_numerator_y )
                                            + ( da_slope / a_slope * da_slope / a_slope ) ).sqrt();

        let center_y = center_first_term + ( self.start[1] + self.mid[1] ) / 2.0;
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
        let center = self.center();
        normalize_angle(
            (self.start[1] - center[1])
                .atan2(self.start[0] - center[0])
                .to_degrees(),
        )
    }
    fn end_angle(&self) -> f64 {
        let center = self.center();
        normalize_angle(
            (self.end[1] - center[1])
                .atan2(self.end[0] - center[0])
                .to_degrees(),
        )
    }
}

pub struct MathUtils;

pub struct RoundedCorner {
    pos: Array1<f64>,
    radius: f64,
}

const MIN_SEGCOUNT_FOR_CIRCLE: f64 = 8.0;

impl MathUtils {
    ///calculate vector end pos from start, langth and angle.
    pub fn projection(point: &Array1<f64>, angle: f64, length: f64) -> Array1<f64> {
        round!(arr1(&[
            point[0] + length * angle.to_radians().cos(),
            point[1] + length * angle.to_radians().sin(),
        ]))
    }

    fn euclidian_norm(pos: Array1<f64>) -> f64 {
        (pos[0] * pos[0] + pos[1] * pos[1]).sqrt()
    }
    ///calculate the number of segments to approximate a circle by segments
    ///given the max distance between the middle of a segment and the circle
    pub fn arc_to_segment_count(radius: f64, error_max: i32, arc_angle_degree: f64) -> u32 {
        // avoid divide-by-zero
        let radius = if 1.0 > radius { 1.0 } else { radius };

        // error relative to the radius value:
        let rel_error = error_max as f64 / radius;
        // minimal arc increment in degrees:
        let mut arc_increment = 180.0 / std::f64::consts::PI * (1.0 - rel_error).acos() * 2.0;

        // Ensure a minimal arc increment reasonable value for a circle
        // (360.0 degrees). For very small radius values, this is mandatory.
        arc_increment = if 360.0 / MIN_SEGCOUNT_FOR_CIRCLE < arc_increment {
            360.0 / MIN_SEGCOUNT_FOR_CIRCLE
        } else {
            arc_increment
        };

        let seg_count = arc_angle_degree.abs() / arc_increment;

        // Ensure at least two segments are used for algorithmic safety
        if seg_count > 2.0 {
            seg_count as u32
        } else {
            2
        }
    }

    /* pub fn CornerListToPolygon( aCorners: Vec<RoundedCorner>, aInflate: i32) {

        // outline.NewOutline();
        let mut outline: Array2::<f64> = Array2::zeros((0, 2));
        let incoming = aCorners[0].pos - aCorners.last().unwrap().pos;

        // for( int n = 0, count = aCorners.size(); n < count; n++ )
        for corner in aCorners.windows(2) {

            /* ROUNDED_CORNER& cur = aCorners[n];
            ROUNDED_CORNER& next = aCorners[( n + 1 ) % count]; */
            let outgoing = corner[1].pos - corner[0].pos;

            if aInflate == 0 || corner[0].radius == 0.0 {
                outline.push_row(ArrayView::from(&corner[0].pos));
            } else {
                // VECTOR2I cornerPosition = cur.m_position;
                let endAngle = corner[0].radius;
                let radius = corner[0].radius;
                let mut tanAngle2 = 0.0;

                if ( incoming[0] == 0.0 && outgoing[1] == 0.0 ) || ( incoming[1] == 0.0 && outgoing[0] == 0.0 ) {
                    endAngle = 90.0;
                    tanAngle2 = 1.0;
                } else {
                    let cosNum = incoming[0] * outgoing[0] + incoming[1] * outgoing[1];
                    // let cosDen = incoming.EuclideanNorm() * outgoing.EuclideanNorm();
                    let cosDen = Self::euclidian_norm(incoming) * Self::euclidian_norm(outgoing);
                    let angle = ( cosNum / cosDen ).acos();
                    tanAngle2 = ( ( std::f64::consts::PI - angle ) / 2 ).tan();
                    endAngle = angle.to_degrees();
                }

                if aInflate > 0 && tanAngle2 > 0.0 {
                    radius += aInflate as f64;
                    cornerPosition += incoming.Resize( aInflate / tanAngle2 )
                                    + incoming.Perpendicular().Resize( -aInflate );
                }

                // Ensure 16+ segments per 360deg and ensure first & last segment are the same size
                let numSegs = std::max( 16, GetArcToSegmentCount( radius, aError, 360.0 ) );
                let angDelta = 3600 / numSegs;
                let lastSegLen = endAngle % angDelta; // or 0 if last seg length is angDelta
                let angPos = lastSegLen ? ( angDelta + lastSegLen ) / 2 : angDelta;

                let arcTransitionDistance = radius / tanAngle2;
                VECTOR2I arcStart = cornerPosition - incoming.Resize( arcTransitionDistance );
                VECTOR2I arcCenter = arcStart + incoming.Perpendicular().Resize( radius );
                VECTOR2I arcEnd, arcStartOrigin;

                if( aErrorLoc == ERROR_INSIDE )
                {
                    arcEnd = SEG( cornerPosition, arcCenter ).ReflectPoint( arcStart );
                    arcStartOrigin = arcStart - arcCenter;
                    outline.Append( arcStart );
                }
                else
                {
                    // The outer radius should be radius+aError, recalculate because numSegs is clamped
                    int actualDeltaRadius = CircleToEndSegmentDeltaRadius( radius, numSegs );
                    int radiusExtend = GetCircleToPolyCorrection( actualDeltaRadius );
                    arcStart += incoming.Perpendicular().Resize( -radiusExtend );
                    arcStartOrigin = arcStart - arcCenter;

                    // To avoid "ears", we only add segments crossing/within the non-rounded outline
                    // Note: outlineIn is short and must be treated as defining an infinite line
                    SEG      outlineIn( cornerPosition - incoming, cornerPosition );
                    VECTOR2I prevPt = arcStart;
                    arcEnd = cornerPosition; // default if no points within the outline are found

                    while( angPos < endAngle )
                    {
                        VECTOR2I pt = arcStartOrigin;
                        RotatePoint( pt, -angPos );
                        pt += arcCenter;
                        angPos += angDelta;

                        if( outlineIn.Side( pt ) > 0 )
                        {
                            VECTOR2I intersect = outlineIn.IntersectLines( SEG( prevPt, pt ) ).get();
                            outline.Append( intersect );
                            outline.Append( pt );
                            arcEnd = SEG( cornerPosition, arcCenter ).ReflectPoint( intersect );
                            break;
                        }

                        endAngle -= angDelta; // if skipping first, also skip last
                        prevPt = pt;
                    }
                }

                for( ; angPos < endAngle; angPos += angDelta )
                {
                    VECTOR2I pt = arcStartOrigin;
                    RotatePoint( pt, -angPos );
                    outline.Append( pt + arcCenter );
                }

                outline.Append( arcEnd );
            }

            incoming = outgoing;
        }
    } */
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PinOrientation {
    Left,
    Right,
    Up,
    Down,
}

impl PinOrientation {
    pub fn from(symbol: &Symbol, pin: &Pin) -> Self {
        let mut position: Vec<usize> = vec![0; 4];
        let symbol_shift: usize = (symbol.angle / 90.0).round() as usize;

        let lib_pos: usize = (pin.angle / 90.0).round() as usize;
        position[lib_pos] += 1;

        position.rotate_right(symbol_shift);
        if let Some(mirror) = &symbol.mirror {
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

pub fn normalize_angle(angle: f64) -> f64 {
    if angle > 360.0 {
        angle - 360.0
    } else if angle < 0.0 {
        angle + 360.0
    } else {
        angle
    }
}

#[cfg(test)]
mod tests {
    use ndarray::{arr1, arr2};

    use super::{normalize_angle, CalcArc, MathUtils};
    use crate::sexp::{Arc, Schema, Stroke};
    use crate::sexp::{Bounds, Shape, Transform};

    #[test]
    fn pin_pos_r1() {
        let doc = Schema::load("files/pinpos.kicad_sch").unwrap();
        let symbol = doc.get_symbol("R1", 1).unwrap();
        let lib_symbol = doc.get_library("Device:R").unwrap();

        let pin1 = lib_symbol.get_pin(String::from("1")).unwrap();
        let pos = Shape::transform(symbol, &pin1.at);
        assert_eq!(arr1(&[48.26, 38.1]), pos);
        let pin2 = lib_symbol.get_pin(String::from("2")).unwrap();
        let pos = Shape::transform(symbol, &pin2.at);
        assert_eq!(arr1(&[48.26, 45.72]), pos);
    }
    #[test]
    fn pin_pos_r2() {
        let doc = Schema::load("files/pinpos.kicad_sch").unwrap();
        let symbol = doc.get_symbol("R2", 1).unwrap();
        let lib_symbol = doc.get_library("Device:R").unwrap();

        let pin1 = lib_symbol.get_pin(String::from("1")).unwrap();
        let pos = Shape::transform(symbol, &pin1.at);
        assert_eq!(arr1(&[58.42, 41.91]), pos);
        let pin2 = lib_symbol.get_pin(String::from("2")).unwrap();
        let pos = Shape::transform(symbol, &pin2.at);
        assert_eq!(arr1(&[66.04, 41.91]), pos);
    }
    #[test]
    fn pin_pos_r3() {
        let doc = Schema::load("files/pinpos.kicad_sch").unwrap();
        let symbol = doc.get_symbol("R3", 1).unwrap();
        let lib_symbol = doc.get_library("Device:R").unwrap();

        let pin1 = lib_symbol.get_pin(String::from("1")).unwrap();
        let pos = Shape::transform(symbol, &pin1.at);
        assert_eq!(arr1(&[76.2, 45.72]), pos);
        let pin2 = lib_symbol.get_pin(String::from("2")).unwrap();
        let pos = Shape::transform(symbol, &pin2.at);
        assert_eq!(arr1(&[76.2, 38.1]), pos);
    }
    #[test]
    fn pin_pos_r4() {
        let doc = Schema::load("files/pinpos.kicad_sch").unwrap();
        let symbol = doc.get_symbol("R4", 1).unwrap();
        let lib_symbol = doc.get_library("Device:R").unwrap();

        let pin1 = lib_symbol.get_pin(String::from("1")).unwrap();
        let pos = Shape::transform(symbol, &pin1.at);
        assert_eq!(arr1(&[93.98, 41.91]), pos);
        let pin2 = lib_symbol.get_pin(String::from("2")).unwrap();
        let pos = Shape::transform(symbol, &pin2.at);
        assert_eq!(arr1(&[86.36, 41.91]), pos);
    }
    #[test]
    fn pin_pos_4069() {
        let doc = Schema::load("files/pinpos_2.kicad_sch").unwrap();
        let symbol = doc.get_symbol("R3", 1).unwrap();
        let lib_symbol = doc.get_library("Device:R").unwrap();

        let pin1 = lib_symbol.get_pin(String::from("1")).unwrap();
        let pos = Shape::transform(symbol, &pin1.at);
        assert_eq!(arr1(&[63.5, 33.02]), pos);
        let pin2 = lib_symbol.get_pin(String::from("2")).unwrap();
        let pos = Shape::transform(symbol, &pin2.at);
        assert_eq!(arr1(&[63.5, 25.4]), pos);
    }
    #[test]
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
    }
    #[test]
    fn test_angle_to_segment_count() {
        assert_eq!(14, MathUtils::arc_to_segment_count(200000.0, 5000, 360.0));
        assert_eq!(21, MathUtils::arc_to_segment_count(450000.0, 5000, 360.0));
    }
}
