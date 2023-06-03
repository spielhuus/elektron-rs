use super::plotter::{text, Circle, Line, PlotItem, Text, Theme, Arc, Polyline};
use super::{
    plotter::{FillType, ItemPlot, Style},
    theme::Themer,
};
use crate::error::Error;
use crate::plot::plotter::LineCap;
use crate::sexp::{self, normalize_angle};
use crate::sexp::{Effects, Pcb, PcbElements, Shape, TitleBlock, Transform};
use ndarray::{arr2, arr1, Array1, Array2, ArrayView};


struct RoundedCorner {
    pos: Array1<f64>,
    radius: f64,
}

/* fn CornerListToPolygon( aCorners: Vec<RoundedCorner>, aInflate: i32) {

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
                let cosDen = euclidian_norm(incoming) * euclidian_norm(outgoing);
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

/* fn TransformRoundChamferedRectToPolygon( SHAPE_POLY_SET& aCornerBuffer, const wxPoint& aPosition,
                                           const wxSize& aSize, double aRotation, int aCornerRadius,
                                           double aChamferRatio, int aChamferCorners, int aInflate,
                                           int aError, ERROR_LOC aErrorLoc ) */

/* fn TransformRoundChamferedRectToPolygon( aCornerBuffer: Array2<f64>, aPosition: &Array1<f64>,
                                           aSize: Array1<f64>, aRotation: f64, aCornerRadius, i32,
                                           aChamferRatio: f64, aChamferCorners: u32, aInflate: f64) -> Result<(), Error> {

    // SHAPE_POLY_SET outline;
    // wxSize         size( aSize / 2.0 );
    let size = aSize / 2.0;
    // int            chamferCnt = std::bitset<8>( aChamferCorners ).count();
    let chamferDeduct = 0.0;

    if aInflate < 0.0 {
        size = arr1(&[if size[0] + aInflate > 1.0 { size[0] + aInflate } else { 1.0 },
                      if size[1] + aInflate > 1.0 { size[1] + aInflate } else { 1.0 }]);
        chamferDeduct = aInflate * ( 2.0 - std::f64::consts::SQRT_2 );
        aCornerRadius = if aCornerRadius + aInflate > 0.0 { aCornerRadius + aInflate } else { 0.0 };
        aInflate = 0.0;
    }

    // std::vector<ROUNDED_CORNER> corners;
    // corners.reserve( 4 + chamferCnt );
    let corners: Array2<f64> = arr2(&[
        [-size[0], -size[1], aCornerRadius],
        [size[0], -size[1], aCornerRadius],
        [size[0], size[1], aCornerRadius],
        [-size[0], size[1], aCornerRadius]
    ]);

    if aChamferCorners > 0 {
        /* int shorterSide = std::min( aSize.x, aSize.y );
        int chamfer = std::max( 0, KiROUND( aChamferRatio * shorterSide + chamferDeduct ) );
        int chamId[4] = { RECT_CHAMFER_TOP_LEFT, RECT_CHAMFER_TOP_RIGHT,
                          RECT_CHAMFER_BOTTOM_RIGHT, RECT_CHAMFER_BOTTOM_LEFT };
        int sign[8] = { 0, 1, -1, 0, 0, -1, 1, 0 };

        for( int cc = 0, pos = 0; cc < 4; cc++, pos++ )
        {
            if( !( aChamferCorners & chamId[cc] ) )
                continue;

            corners[pos].m_radius = 0;

            if( chamfer == 0 )
                continue;

            corners.insert( corners.begin() + pos + 1, corners[pos] );
            corners[pos].m_position.x += sign[( 2 * cc ) & 7] * chamfer;
            corners[pos].m_position.y += sign[( 2 * cc - 2 ) & 7] * chamfer;
            corners[pos + 1].m_position.x += sign[( 2 * cc + 1 ) & 7] * chamfer;
            corners[pos + 1].m_position.y += sign[( 2 * cc - 1 ) & 7] * chamfer;
            pos++;
        }

        if( chamferCnt > 1 && 2 * chamfer >= shorterSide )
            CornerListRemoveDuplicates( corners ); */
    }

    CornerListToPolygon( outline, corners, aInflate, aError, aErrorLoc );

    /* if( aRotation != 0.0 )
        outline.Rotate( DECIDEG2RAD( -aRotation ), VECTOR2I( 0, 0 ) ); */

    // outline.Move( VECTOR2I( aPosition ) );
    aCornerBuffer.append( outline );
    Ok(())
} */

/* macro_rules! theme {
    ($self:expr, $element:expr) => {
        Themer::get(
            &Stroke {
                width: $element.width,
                linetype: "default".to_string(),
                color: (0.0, 0.0, 0.0, 0.0),
            },
            &$self.theme.stroke(&$element.layer).unwrap(),
        )
    };
} */

pub struct PcbPlot<'a, I> {
    iter: I,
    border: bool,
    title_block: &'a Option<TitleBlock>,
    paper_size: (f64, f64),
    pcb: &'a Pcb,
}

impl<'a, I> Iterator for PcbPlot<'a, I>
where
    I: Iterator<Item = &'a PcbElements>,
{
    type Item = Vec<PlotItem>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(PcbElements::Line(line)) => {
                    return self.item(line);
                }
                Some(PcbElements::Segment(segment)) => {
                    return self.item(segment);
                }
                Some(PcbElements::Footprint(footprint)) => {
                    return self.item(footprint);
                }
                Some(PcbElements::Text(text)) => {
                    return self.item(text);
                }
                Some(PcbElements::Zone(zone)) => {
                    //TODO: return self.item(footprint);
                }
                Some(PcbElements::Via(via)) => {
                    return self.item(via);
                }
                Some(PcbElements::GrPoly(poly)) => {
                    return self.item(poly);
                }
                Some(PcbElements::GrCircle(circle)) => {
                     return self.item(circle);
                }
                None => {
                    return None;
                }
                Some(e) => {
                    println!("unknown element {:?}", e);
                }
            }
        }
    }
}

impl<'a, I> PcbPlot<'a, I> {
    pub fn new(
        iter: I,
        pcb: &'a Pcb,
        title_block: &'a Option<TitleBlock>,
        paper_size: (f64, f64),
        border: bool,
    ) -> Self {
        Self {
            iter,
            pcb,
            title_block,
            paper_size,
            border,
        }
    }
}

pub trait PcbPlotIterator<T>: Iterator<Item = T> + Sized {
    fn plot<'a>(
        self,
        pcb: &'a Pcb,
        title_block: &'a Option<TitleBlock>,
        paper_size: (f64, f64),
        border: bool,
    ) -> PcbPlot<'a, Self> {
        PcbPlot::new(self, pcb, title_block, paper_size, border)
    }
}

impl<T, I: Iterator<Item = T>> PcbPlotIterator<T> for I {}

impl<T> ItemPlot<sexp::Footprint> for T {
    fn item(&self, item: &sexp::Footprint) -> Option<Vec<PlotItem>> {
        let mut graphics = Vec::new();
        for pad in &item.pads {
            match pad.padshape {
                sexp::PadShape::Circle => {
                    graphics.push(PlotItem::Circle(
                        40,
                        Circle::new(
                            Shape::transform(item, &pad.at),
                            pad.size[0] / 2.0,
                            Some(1.0), //TODO: Some(pad.width),
                            None,
                            None,
                            vec![Style::PadFront],
                        ),
                    ));
                },
                sexp::PadShape::Oval => {
                    let deltaxy = pad.size[1] - pad.size[0];       /* distance between centers of the oval */
                    let radius   = pad.size[0] / 2.0;
                    let mut cx = -radius;
                    let mut cy = -deltaxy / 2.0;
                    //RotatePoint( &cx, &cy, orient );
                    let start = ( cx + pad.at[0], cy + pad.at[1]);
                    cx = -radius;
                    cy = deltaxy / 2.0;
                    // RotatePoint( &cx, &cy, orient );
                    let end = ( cx + pad.at[0], cy + pad.at[1] );
                    
                    graphics.push(PlotItem::Line(
                        10,
                        Line::new(
                            Shape::transform(
                                item,
                                &arr2(&[
                                    [start.0, start.1],
                                    [end.0, end.1],
                                ]),
                            ),
                            Some(1.0),
                            None,
                            None,
                            None,
                            vec![Style::Test],
                            // vec![Style::Layer(pad.layer.replace('.', "_"))],
                        ),
                    ));

                    cx = radius;
                    cy = -deltaxy / 2.0;
                    // RotatePoint( &cx, &cy, orient );
                    let start = ( cx + pad.at[0], cy + pad.at[1]);
                    cx = radius;
                    cy = deltaxy / 2.0;
                    // RotatePoint( &cx, &cy, orient );
                    let end = ( cx + pad.at[0], cy + pad.at[1]);

                    graphics.push(PlotItem::Line(
                        10,
                        Line::new(
                            Shape::transform(
                                item,
                                &arr2(&[
                                    [start.0, start.1],
                                    [end.0, end.1],
                                ]),
                            ),
                            Some(1.0),
                            None,
                            None,
                            None,
                            vec![Style::Test],
                            // vec![Style::Layer(pad.layer.replace('.', "_"))],
                        ),
                    ));

                    cx = 0.0;
                    cy = deltaxy / 2.0;
                    // RotatePoint( &cx, &cy, orient );
                    // Arc( wxPoint( cx + pos.x, cy + pos.y ), orient + 1800, orient + 3600, radius, FILL_T::NO_FILL );
                    graphics.push(PlotItem::Arc(
                        1,
                        Arc::from_center(
                            Shape::transform(item, &arr1(&[cx, cy])),
                            radius,
                            normalize_angle(item.angle + 180.0),
                            normalize_angle(item.angle + 360.0),
                            Some(1.0),
                            // if arc.stroke.width == 0.0 { None } else { Some(arc.stroke.width) },
                            None,
                            None,
                            vec![Style::Test],
                        ),
                    ));

                    cx = 0.0;
                    cy = -deltaxy / 2.0;
                    // RotatePoint( &cx, &cy, orient );
                    // Arc( wxPoint( cx + pos.x, cy + pos.y ), orient, orient + 1800, radius, FILL_T::NO_FILL );
                    graphics.push(PlotItem::Arc(
                        1,
                        Arc::from_center(
                            Shape::transform(item, &arr1(&[cx, cy])),
                            radius,
                            normalize_angle(item.angle),
                            normalize_angle(item.angle + 180.0),
                            Some(1.0),
                            // if arc.stroke.width == 0.0 { None } else { Some(arc.stroke.width) },
                            None,
                            None,
                            vec![Style::Test],
                        ),
                    ));
                },
                sexp::PadShape::Rect => {
                    /* static std::vector< wxPoint > cornerList;
                    wxSize size( aSize );
                    cornerList.clear();

                    if( aTraceMode == FILLED )
                        SetCurrentLineWidth( 0 );
                    else
                        SetCurrentLineWidth( USE_DEFAULT_LINE_WIDTH ); */

                    let dx = pad.size[0] / 2.0;
                    let dy = pad.size[1] / 2.0;

                    /* let pts = arr2(&[
                        [item.at[0] - dx, item.at[1] + dy],
                        [item.at[0] - dx, item.at[1] - dy],
                        [item.at[0] + dx, item.at[1] - dy],
                        [item.at[0] + dx, item.at[1] + dy],
                        [item.at[0] - dx, item.at[1] + dy],
                    ]); */

                    let pts = arr2(&[
                        [-dx, dy],
                        [-dx, -dy],
                        [dx, -dy],
                        [dx, dy],
                        [-dx, dy],
                    ]);

                    graphics.push(PlotItem::Polyline(
                        1,
                        Polyline::new(
                            Shape::transform(item, &pts),
                            None,
                            None,
                            None,
                            vec![
                                Style::Test,
                            ],
                        ),
                    ));

                    // PlotPoly( cornerList, ( aTraceMode == FILLED ) ? FILL_T::FILLED_SHAPE : FILL_T::NO_FILL,
                },
                sexp::PadShape::Trapezoid => { todo!() },
                sexp::PadShape::RoundRect => {
                    //TODO println!("PAD roundrect");
                    /* wxSize size( aSize );

                    if( aTraceMode == FILLED )
                    {
                        SetCurrentLineWidth( 0 );
                    }
                    else
                    {
                        SetCurrentLineWidth( USE_DEFAULT_LINE_WIDTH );
                    }


                    SHAPE_POLY_SET outline;
                    TransformRoundChamferedRectToPolygon( outline, aPadPos, size, aOrient, aCornerRadius,
                                                          0.0, 0, 0, GetPlotterArcHighDef(), ERROR_INSIDE );

                    std::vector< wxPoint > cornerList;

                    // TransformRoundRectToPolygon creates only one convex polygon
                    SHAPE_LINE_CHAIN& poly = outline.Outline( 0 );
                    cornerList.reserve( poly.PointCount() );

                    for( int ii = 0; ii < poly.PointCount(); ++ii )
                        cornerList.emplace_back( poly.CPoint( ii ).x, poly.CPoint( ii ).y );

                    // Close polygon
                    cornerList.push_back( cornerList[0] );

                    PlotPoly( cornerList, ( aTraceMode == FILLED ) ? FILL_T::FILLED_SHAPE : FILL_T::NO_FILL, GetCurrentLineWidth() ); */
                },
                sexp::PadShape::ChamferedRect => todo!(),
                sexp::PadShape::Custom => { todo!() },
            }
        }
        for graphic in &item.graphics {
            match graphic {
                sexp::Graphics::FpText(text) => {
                    if !text.hidden {
                        let angle = if let Some(angle) = text.angle {
                            angle
                        } else {
                            0.0
                        };
                        graphics.push(text!(
                            Shape::transform(item, &text.at),
                            angle,
                            text.value.clone(),
                            Effects::new(),    //TODO:
                            vec![Style::Text /*TODO:, Style::Layer(text.layer.replace('.', "_")) */ ]
                        ));
                    }
                }
                sexp::Graphics::FpLine(line) => {
                    graphics.push(PlotItem::Line(
                        10,
                        Line::new(
                            Shape::transform(
                                item,
                                &arr2(&[
                                    [line.start[0], line.start[1]],
                                    [line.end[0], line.end[1]],
                                ]),
                            ),
                            Some(line.width),
                            None,
                            None,
                            None,
                            vec![Style::Layer(line.layer.replace('.', "_"))],
                        ),
                    ));
                }
                sexp::Graphics::FpCircle(circle) => {
                    // let stroke = theme!(self, circle);
                    graphics.push(PlotItem::Circle(
                        1,
                        Circle::new(
                            Shape::transform(item, &circle.center),
                            ((circle.end[0] - circle.center[0]).powf(2.0)
                                + (circle.end[1] - circle.center[1]).powf(2.0))
                            .sqrt(),
                            Some(circle.width),
                            None,
                            None,
                            vec![Style::Layer(circle.layer.replace('.', "_"))],
                        ),
                    ));
                }
                sexp::Graphics::FpArc(_) => {}
            }
        }
        Some(graphics)
    }
}

impl<T> ItemPlot<sexp::GrText> for T {
    fn item(&self, item: &sexp::GrText) -> Option<Vec<PlotItem>> {
        /*TODO:  if !item.hidden {
            let angle = if let Some(angle) = text.angle {
                angle
            } else {
                0.0
            }; */
            Some(vec![text!(
                item.at.clone(),
                item.angle,
                item.text.clone(),
                Effects::new(),    //TODO:
                vec![Style::Text /*TODO:, Style::Layer(item.layer.replace('.', "_"))*/]
            )])
        // } else { vec![] }
    }
}

impl<T> ItemPlot<sexp::Segment> for T {
    fn item(&self, item: &sexp::Segment) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(
                    arr2(&[[item.start[0], item.start[1]], [item.end[0], item.end[1]]]),
                    Some(item.width),
                    None,
                    None,
                    Some(LineCap::Round),
                    // vec![Style::Segment, Style::Layer(item.layer.replace('.', "_"))], //TODO
                    vec![],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::GrLine> for T {
    fn item(&self, item: &sexp::GrLine) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(
                    arr2(&[[item.start[0], item.start[1]], [item.end[0], item.end[1]]]),
                    Some(item.width),
                    None,
                    None,
                    None,
                    vec![Style::Layer(item.layer.replace('.', "_"))],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::GrPoly> for T {
    fn item(&self, item: &sexp::GrPoly) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Line(
                10,
                Line::new(
                    item.pts.clone(),
                    Some(item.width),
                    None,
                    None,
                    None,
                    vec![Style::Layer(item.layer.replace('.', "_"))],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::GrCircle> for T {
    fn item(&self, item: &sexp::GrCircle) -> Option<Vec<PlotItem>> {
        let radius = ((item.center[0] -item.end[0]).powf(2.0) + (item.center[1] - item.end[1]).powf(2.0)).sqrt().abs();
        Some(vec![
            (PlotItem::Circle(
                1,
                Circle::new(
                    item.center.clone(),
                    radius,
                    None,
                    None,
                    None,
                    vec![Style::Layer(item.layer.replace('.', "_"))],
                ),
            )),
        ])
    }
}

impl<T> ItemPlot<sexp::Via> for T {
    fn item(&self, item: &sexp::Via) -> Option<Vec<PlotItem>> {
        Some(vec![
            (PlotItem::Circle(
                1,
                Circle::new(
                    item.at.clone(),
                    item.size,
                    None,
                    None,
                    None,
                    vec![Style::Layer(item.layers[0].replace('.', "_"))],
                ),
            )),
            (PlotItem::Circle(
                1,
                Circle::new(
                    item.at.clone(),
                    item.drill,
                    None,
                    None,
                    None,
                    vec![Style::Layer(item.layers[0].replace('.', "_"))],
                ),
            )),
        ])
    }
}

#[cfg(test)]
mod tests {
    /* use crate::sexp::Schema;
    use std::path::Path;

    #[test]
    fn bom() {
        let doc = Schema::load("samples/files/summe/summe.kicad_sch").unwrap();
        doc.plot("/tmp/summe.svg", 1.0, true, "kicad_2000").unwrap();
        assert!(Path::new("/tmp/summe.svg").exists());
        assert!(Path::new("/tmp/summe.svg").metadata().unwrap().len() > 0);
    } */
}
