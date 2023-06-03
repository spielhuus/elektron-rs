use super::{
    plotter::{
        Arc, Circle, Draw, Drawer, ImageType, Line, Outline, PlotItem, PlotterImpl, Polyline,
        Rectangle, Text, Theme,
    },
    theme::Themer,
};
use crate::{error::Error, sexp::LayerId};
use crate::sexp::Pcb;
use chrono::{DateTime, Local};
use itertools::Itertools;
use ndarray::{arr2, Array2};
use pangocairo::{create_layout, pango::SCALE, show_layout, update_layout};
use std::{io::Write, path::Path};
extern crate cairo;
use cairo::{Context, Format, ImageSurface, PdfSurface, SvgSurface};

fn rgba_color(color: (f64, f64, f64, f64)) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        (color.0 * 255.0) as u32,
        (color.1 * 255.0) as u32,
        (color.2 * 255.0) as u32,
        (color.3 * 255.0) as u32
    )
}

pub mod paper {
    pub const A4: (f64, f64) = (297.0, 210.0);
}

macro_rules! color {
    ($element:expr, $themer:expr) => {
        if let Some(color) = $element.color {
            if color != (0.0, 0.0, 0.0, 0.0) {
                color
            } else {
                $themer.stroke(&$element.class)
            }
        } else {
            $themer.stroke(&$element.class)
        }
    };
}

macro_rules! stroke {
    ($context:expr, $element:expr, $themer:expr) => {
        let color = color!($element, $themer);
        $context.set_source_rgba(color.0, color.1, color.2, color.3);
        $context.set_line_width($themer.stroke_width($element.width, &$element.class));
    };
}
macro_rules! fill {
    ($context:expr, $element:expr, $themer:expr) => {
        let fill = $themer.fill(&$element.class);
        if let Some(fill) = fill {
            $context.set_source_rgba(fill.0, fill.1, fill.2, fill.3);
            $context.fill().unwrap();
        }
    };
}

/// Plotter implemntation for gerber files.
pub struct GerberPlotter<'a> {
    context: Context,
    paper_size: (f64, f64),
    image_type: ImageType,
    themer: Themer<'a>,
    x2_headers: Vec<String>,
}

impl<'a> GerberPlotter<'a> {
    pub fn new() -> GerberPlotter<'a> {
        let surface = ImageSurface::create(
            Format::Rgb24,
            (297.0 * 72.0 / 25.4) as i32,
            (210.0 * 72.0 / 25.4) as i32,
        )
        .unwrap();
        let context = Context::new(&surface).unwrap();
        context.scale(72.0 / 25.4, 72.0 / 25.4);
        GerberPlotter {
            context,
            paper_size: paper::A4,
            image_type: ImageType::Svg,
            themer: Themer::new(Theme::Kicad2020),
            x2_headers: Vec::new(),
        }
    }

    fn guid(name: &str) -> String {
        /* Gerber GUID format should be RFC4122 Version 1 or 4.
         * See en.wikipedia.org/wiki/Universally_unique_identifier
         * The format is:
         * xxxxxxxx-xxxx-Mxxx-Nxxx-xxxxxxxxxxxx
         * with
         *   x = hexDigit lower/upper case
         * and
         *  M = '1' or '4' (UUID version: 1 (basic) or 4 (random)) (we use 4: UUID random)
         * and
         *  N = '8' or '9' or 'A|a' or 'B|b' : UUID variant 1: 2 MSB bits have meaning) (we use N = 9)
         *  N = 1000 or 1001 or 1010 or 1011  : 10xx means Variant 1 (Variant2: 110x and 111x are
         *  reserved)
         */

        if name.len() < 16 {
            format!("{:X<1$}", name, 16)
        } else {
            name.to_string()
        }
        .chars()
        .take(15)
        .enumerate()
        .map(|(i, c)| {
            if i == 4 {
                format!("-{:02x}", c as u8)
            } else if i == 6 {
                format!("-4{:02x}", c as u8)
            } else if i == 7 {
                format!("{:1x}-9{:1x}", c as u8 >> 4, c as u8 & 0x0F)
            } else if i == 9 {
                format!("-{:02x}", c as u8)
            } else {
                format!("{:02x}", c as u8)
            }
        })
        .collect::<Vec<String>>()
        .join("")
    }

    fn layer_function(id: LayerId) -> String {

        match id {
            LayerId::FAdhes => String::from("Glue,Top"),
            LayerId::BAdhes => String::from("Glue,Bot"),
            LayerId::FSilkS => String::from("Legend,Top"),
            LayerId::BSilkS => String::from("Legend,Bot"),
            LayerId::FMask => String::from("Soldermask,Top"),
            LayerId::BMask => String::from("Soldermask,Bot"),
            LayerId::FPaste => String::from("Paste,Top"),
            LayerId::BPaste => String::from("Paste,Bot"),
            LayerId::EdgeCuts => String::from("Profile,NP"),
            LayerId::DwgsUser => String::from("OtherDrawing,Comment"),
            LayerId::CmtsUser => String::from("Other,Comment"),
            LayerId::Eco1User => String::from("Other,ECO1"),
            LayerId::Eco2User => String::from("Other,ECO2" ),
            LayerId::BFab => String::from("AssemblyDrawing,Bot"),
            LayerId::FFab => String::from("AssemblyDrawing,Top"),
            /* B_Cu => String::from(
                attrib.Printf( wxT( "Copper,L%d,Bot" ), aBoard->GetCopperLayerCount() );

            F_Cu => String::from("Copper,L1,Top"),
            _ => {
                if( IsCopperLayer( aLayer ) ) {
                    attrib.Printf( wxT( "Copper,L%d,Inr" ), aLayer+1 );
                } else {
                    attrib.Printf( wxT( "Other,User" ), aLayer+1 );
                }
            } */
            _ => { todo!(); }
        }
    }

    fn plot_graphics_items(&self, pcb: &Pcb, layer: LayerId) -> Result<(), Error> {

        Ok(())
    }

    fn plot_footprints(&self, pcb: &Pcb, layer: LayerId) -> Result<(), Error> {


        Ok(())
    }

    fn plot_layer(&self, pcb: &Pcb, layer: LayerId) -> Result<(), Error> {
        if layer.is_copper() {
            println!("copper layer: {}", layer);

            /* plotOpt.SetSkipPlotNPTH_Pads( true );
            PlotStandardLayer( aBoard, aPlotter, layer_mask, plotOpt ); */

            /* pcb.elements().unwrap().filter_map(Pcb::footprints).for_each(|fp| {
                println!("\t\tfp: {}", fp.descr);
            }); */

            /* // Draw footprint texts:
            for( const FOOTPRINT* footprint : aBoard->Footprints() )
                itemplotter.PlotFootprintTextItems( footprint );

            // Draw footprint other graphic items:
            for( const FOOTPRINT* footprint : aBoard->Footprints() )
                itemplotter.PlotFootprintGraphicItems( footprint );

            // Plot footprint pads
            for( FOOTPRINT* footprint : aBoard->Footprints() )
            {
                aPlotter->StartBlock( nullptr );

                for( PAD* pad : footprint->Pads() )
                {
                    OUTLINE_MODE padPlotMode = plotMode;

                    if( !( pad->GetLayerSet() & aLayerMask ).any() )
                    {
                        if( sketchPads &&
                                ( ( onFrontFab && pad->GetLayerSet().Contains( F_Cu ) ) ||
                                  ( onBackFab && pad->GetLayerSet().Contains( B_Cu ) ) ) )
                        {
                            padPlotMode = SKETCH;
                        }
                        else
                        {
                            continue;
                        }
                    }

                    /// pads not connected to copper are optionally not drawn
                    if( onCopperLayer && !pad->FlashLayer( aLayerMask ) )
                        continue;

                    COLOR4D color = COLOR4D::BLACK;

                    if( ( pad->GetLayerSet() & aLayerMask )[B_Cu] )
                       color = aPlotOpt.ColorSettings()->GetColor( B_Cu );

                    if( ( pad->GetLayerSet() & aLayerMask )[F_Cu] )
                        color = color.LegacyMix( aPlotOpt.ColorSettings()->GetColor( F_Cu ) );

                    if( sketchPads && aLayerMask[F_Fab] )
                        color = aPlotOpt.ColorSettings()->GetColor( F_Fab );
                    else if( sketchPads && aLayerMask[B_Fab] )
                        color = aPlotOpt.ColorSettings()->GetColor( B_Fab );

                    wxSize margin;
                    int width_adj = 0;

                    if( onCopperLayer )
                        width_adj = itemplotter.getFineWidthAdj();

                    if( onSolderMaskLayer )
                        margin.x = margin.y = pad->GetSolderMaskMargin();

                    if( onSolderPasteLayer )
                        margin = pad->GetSolderPasteMargin();

                    // not all shapes can have a different margin for x and y axis
                    // in fact only oval and rect shapes can have different values.
                    // Round shape have always the same x,y margin
                    // so define a unique value for other shapes that do not support different values
                    int mask_clearance = margin.x;

                    // Now offset the pad size by margin + width_adj
                    wxSize padPlotsSize = pad->GetSize() + margin * 2 + wxSize( width_adj, width_adj );

                    // Store these parameters that can be modified to plot inflated/deflated pads shape
                    PAD_SHAPE padShape = pad->GetShape();
                    wxSize      padSize  = pad->GetSize();
                    wxSize      padDelta = pad->GetDelta(); // has meaning only for trapezoidal pads
                    double      padCornerRadius = pad->GetRoundRectCornerRadius();

                    // Don't draw a 0 sized pad.
                    // Note: a custom pad can have its pad anchor with size = 0
                    if( pad->GetShape() != PAD_SHAPE::CUSTOM
                        && ( padPlotsSize.x <= 0 || padPlotsSize.y <= 0 ) )
                        continue;

                    switch( pad->GetShape() )
                    {
                    case PAD_SHAPE::CIRCLE:
                    case PAD_SHAPE::OVAL:
                        pad->SetSize( padPlotsSize );

                        if( aPlotOpt.GetSkipPlotNPTH_Pads() &&
                            ( aPlotOpt.GetDrillMarksType() == PCB_PLOT_PARAMS::NO_DRILL_SHAPE ) &&
                            ( pad->GetSize() == pad->GetDrillSize() ) &&
                            ( pad->GetAttribute() == PAD_ATTRIB::NPTH ) )
                        {
                            break;
                        }

                        itemplotter.PlotPad( pad, color, padPlotMode );
                        break;

                    case PAD_SHAPE::RECT:
                        pad->SetSize( padPlotsSize );

                        if( mask_clearance > 0 )
                        {
                            pad->SetShape( PAD_SHAPE::ROUNDRECT );
                            pad->SetRoundRectCornerRadius( mask_clearance );
                        }

                        itemplotter.PlotPad( pad, color, padPlotMode );
                        break;

                    case PAD_SHAPE::TRAPEZOID:
                        // inflate/deflate a trapezoid is a bit complex.
                        // so if the margin is not null, build a similar polygonal pad shape,
                        // and inflate/deflate the polygonal shape
                        // because inflating/deflating using different values for y and y
                        // we are using only margin.x as inflate/deflate value
                        if( mask_clearance == 0 )
                        {
                            itemplotter.PlotPad( pad, color, padPlotMode );
                        }
                        else
                        {
                            PAD dummy( *pad );
                            dummy.SetAnchorPadShape( PAD_SHAPE::CIRCLE );
                            dummy.SetShape( PAD_SHAPE::CUSTOM );
                            SHAPE_POLY_SET outline;
                            outline.NewOutline();
                            int dx = padSize.x / 2;
                            int dy = padSize.y / 2;
                            int ddx = padDelta.x / 2;
                            int ddy = padDelta.y / 2;

                            outline.Append( -dx - ddy,  dy + ddx );
                            outline.Append(  dx + ddy,  dy - ddx );
                            outline.Append(  dx - ddy, -dy + ddx );
                            outline.Append( -dx + ddy, -dy - ddx );

                            // Shape polygon can have holes so use InflateWithLinkedHoles(), not Inflate()
                            // which can create bad shapes if margin.x is < 0
                            int maxError = aBoard->GetDesignSettings().m_MaxError;
                            int numSegs = GetArcToSegmentCount( mask_clearance, maxError, 360.0 );
                            outline.InflateWithLinkedHoles( mask_clearance, numSegs,
                                                            SHAPE_POLY_SET::PM_FAST );
                            dummy.DeletePrimitivesList();
                            dummy.AddPrimitivePoly( outline, 0, true );

                            // Be sure the anchor pad is not bigger than the deflated shape because this
                            // anchor will be added to the pad shape when plotting the pad. So now the
                            // polygonal shape is built, we can clamp the anchor size
                            dummy.SetSize( wxSize( 0,0 ) );

                            itemplotter.PlotPad( &dummy, color, padPlotMode );
                        }

                        break;

                    case PAD_SHAPE::ROUNDRECT:
                    {
                        // rounding is stored as a percent, but we have to change the new radius
                        // to initial_radius + clearance to have a inflated/deflated similar shape
                        int initial_radius = pad->GetRoundRectCornerRadius();
                        pad->SetSize( padPlotsSize );
                        pad->SetRoundRectCornerRadius( std::max( initial_radius + mask_clearance, 0 ) );

                        itemplotter.PlotPad( pad, color, padPlotMode );
                        break;
                    }

                    case PAD_SHAPE::CHAMFERED_RECT:
                        if( mask_clearance == 0 )
                        {
                            // the size can be slightly inflated by width_adj (PS/PDF only)
                            pad->SetSize( padPlotsSize );
                            itemplotter.PlotPad( pad, color, padPlotMode );
                        }
                        else
                        {
                            // Due to the polygonal shape of a CHAMFERED_RECT pad, the best way is to
                            // convert the pad shape to a full polygon, inflate/deflate the polygon
                            // and use a dummy  CUSTOM pad to plot the final shape.
                            PAD dummy( *pad );
                            // Build the dummy pad outline with coordinates relative to the pad position
                            // and orientation 0. The actual pos and rotation will be taken in account
                            // later by the plot function
                            dummy.SetPosition( wxPoint( 0, 0 ) );
                            dummy.SetOrientation( 0 );
                            SHAPE_POLY_SET outline;
                            int maxError = aBoard->GetDesignSettings().m_MaxError;
                            int numSegs = GetArcToSegmentCount( mask_clearance, maxError, 360.0 );
                            dummy.TransformShapeWithClearanceToPolygon( outline, UNDEFINED_LAYER, 0,
                                                                        maxError, ERROR_INSIDE );
                            outline.InflateWithLinkedHoles( mask_clearance, numSegs,
                                                            SHAPE_POLY_SET::PM_FAST );

                            // Initialize the dummy pad shape:
                            dummy.SetAnchorPadShape( PAD_SHAPE::CIRCLE );
                            dummy.SetShape( PAD_SHAPE::CUSTOM );
                            dummy.DeletePrimitivesList();
                            dummy.AddPrimitivePoly( outline, 0, true );

                            // Be sure the anchor pad is not bigger than the deflated shape because this
                            // anchor will be added to the pad shape when plotting the pad.
                            // So we set the anchor size to 0
                            dummy.SetSize( wxSize( 0,0 ) );
                            dummy.SetPosition( pad->GetPosition() );
                            dummy.SetOrientation( pad->GetOrientation() );

                            itemplotter.PlotPad( &dummy, color, padPlotMode );
                        }

                        break;

                    case PAD_SHAPE::CUSTOM:
                    {
                        // inflate/deflate a custom shape is a bit complex.
                        // so build a similar pad shape, and inflate/deflate the polygonal shape
                        PAD dummy( *pad );
                        SHAPE_POLY_SET shape;
                        pad->MergePrimitivesAsPolygon( &shape );

                        // Shape polygon can have holes so use InflateWithLinkedHoles(), not Inflate()
                        // which can create bad shapes if margin.x is < 0
                        int maxError = aBoard->GetDesignSettings().m_MaxError;
                        int numSegs = GetArcToSegmentCount( mask_clearance, maxError, 360.0 );
                        shape.InflateWithLinkedHoles( mask_clearance, numSegs, SHAPE_POLY_SET::PM_FAST );
                        dummy.DeletePrimitivesList();
                        dummy.AddPrimitivePoly( shape, 0, true );

                        // Be sure the anchor pad is not bigger than the deflated shape because this
                        // anchor will be added to the pad shape when plotting the pad. So now the
                        // polygonal shape is built, we can clamp the anchor size
                        if( mask_clearance < 0 )  // we expect margin.x = margin.y for custom pads
                            dummy.SetSize( padPlotsSize );

                        itemplotter.PlotPad( &dummy, color, padPlotMode );
                        break;
                    }
                    }

                    // Restore the pad parameters modified by the plot code
                    pad->SetSize( padSize );
                    pad->SetDelta( padDelta );
                    pad->SetShape( padShape );
                    pad->SetRoundRectCornerRadius( padCornerRadius );
                }

                aPlotter->EndBlock( nullptr );
            } */

            // Plot vias on copper layers, and if aPlotOpt.GetPlotViaOnMaskLayer() is true,
            // plot them on solder mask

            /* GBR_METADATA gbr_metadata;

            bool isOnCopperLayer = ( aLayerMask & LSET::AllCuMask() ).any();

            if( isOnCopperLayer )
            {
                gbr_metadata.SetApertureAttrib( GBR_APERTURE_METADATA::GBR_APERTURE_ATTRIB_VIAPAD );
                gbr_metadata.SetNetAttribType( GBR_NETLIST_METADATA::GBR_NETINFO_NET );
            }

            aPlotter->StartBlock( nullptr );

            for( const PCB_TRACK* track : aBoard->Tracks() )
            {
                const PCB_VIA* via = dyn_cast<const PCB_VIA*>( track );

                if( !via )
                    continue;

                // vias are not plotted if not on selected layer, but if layer is SOLDERMASK_LAYER_BACK
                // or SOLDERMASK_LAYER_FRONT, vias are drawn only if they are on the corresponding
                // external copper layer
                LSET via_mask_layer = via->GetLayerSet();

                if( aPlotOpt.GetPlotViaOnMaskLayer() )
                {
                    if( via_mask_layer[B_Cu] )
                        via_mask_layer.set( B_Mask );

                    if( via_mask_layer[F_Cu] )
                        via_mask_layer.set( F_Mask );
                }

                if( !( via_mask_layer & aLayerMask ).any() )
                    continue;

                int via_margin = 0;
                double width_adj = 0;

                // If the current layer is a solder mask, use the global mask clearance for vias
                if( aLayerMask[B_Mask] || aLayerMask[F_Mask] )
                    via_margin = aBoard->GetDesignSettings().m_SolderMaskMargin;

                if( ( aLayerMask & LSET::AllCuMask() ).any() )
                    width_adj = itemplotter.getFineWidthAdj();

                int diameter = via->GetWidth() + 2 * via_margin + width_adj;

                /// Vias not connected to copper are optionally not drawn
                if( onCopperLayer && !via->FlashLayer( aLayerMask ) )
                    continue;

                // Don't draw a null size item :
                if( diameter <= 0 )
                    continue;

                // Some vias can be not connected (no net).
                // Set the m_NotInNet for these vias to force a empty net name in gerber file
                gbr_metadata.m_NetlistMetadata.m_NotInNet = via->GetNetname().IsEmpty();

                gbr_metadata.SetNetName( via->GetNetname() );

                COLOR4D color = aPlotOpt.ColorSettings()->GetColor(
                        LAYER_VIAS + static_cast<int>( via->GetViaType() ) );

                // Set plot color (change WHITE to LIGHTGRAY because the white items are not seen on a
                // white paper or screen
                aPlotter->SetColor( color != WHITE ? color : LIGHTGRAY );
                aPlotter->FlashPadCircle( via->GetStart(), diameter, plotMode, &gbr_metadata );
            }

            aPlotter->EndBlock( nullptr );
            aPlotter->StartBlock( nullptr );
            gbr_metadata.SetApertureAttrib( GBR_APERTURE_METADATA::GBR_APERTURE_ATTRIB_CONDUCTOR );
            */

            pcb.elements()?.filter_map(Pcb::segment).for_each(|s| {
                if s.layer == layer {
                    println!("\t\tSegment: {}x{}x{}x{}", s.start, s.end, s.net, s.width);
                    // aPlotter->ThickSegment( track->GetStart(), track->GetEnd(), width, plotMode,
                   //                         &gbr_metadata );
                }
            });

            /*
            // Plot tracks (not vias) :
            for( const PCB_TRACK* track : aBoard->Tracks() )
            {
                if( track->Type() == PCB_VIA_T )
                    continue;

                if( !aLayerMask[track->GetLayer()] )
                    continue;

                // Some track segments can be not connected (no net).
                // Set the m_NotInNet for these segments to force a empty net name in gerber file
                gbr_metadata.m_NetlistMetadata.m_NotInNet = track->GetNetname().IsEmpty();

                gbr_metadata.SetNetName( track->GetNetname() );
                int width = track->GetWidth() + itemplotter.getFineWidthAdj();
                aPlotter->SetColor( itemplotter.getColor( track->GetLayer() ) );

                if( track->Type() == PCB_ARC_T )
                {
                    const    PCB_ARC* arc = static_cast<const PCB_ARC*>( track );
                    VECTOR2D center( arc->GetCenter() );
                    int      radius = arc->GetRadius();
                    double   start_angle = arc->GetArcAngleStart();
                    double   end_angle = start_angle + arc->GetAngle();

                    aPlotter->ThickArc( wxPoint( center.x, center.y ), -end_angle, -start_angle,
                                        radius, width, plotMode, &gbr_metadata );
                }
                else
                {
                    aPlotter->ThickSegment( track->GetStart(), track->GetEnd(), width, plotMode,
                                            &gbr_metadata );
                }
            }

            aPlotter->EndBlock( nullptr );

            // Plot filled ares
            aPlotter->StartBlock( nullptr );

            NETINFO_ITEM nonet( aBoard );

            for( const ZONE* zone : aBoard->Zones() )
            {
                for( PCB_LAYER_ID layer : zone->GetLayerSet().Seq() )
                {
                    if( !aLayerMask[layer] )
                        continue;

                    SHAPE_POLY_SET mainArea = zone->GetFilledPolysList( layer );
                    SHAPE_POLY_SET islands;

                    for( int i = mainArea.OutlineCount() - 1; i >= 0; i-- )
                    {
                        if( zone->IsIsland( layer, i ) )
                        {
                            islands.AddOutline( mainArea.CPolygon( i )[0] );
                            mainArea.DeletePolygon( i );
                        }
                    }

                    itemplotter.PlotFilledAreas( zone, mainArea );

                    if( !islands.IsEmpty() )
                    {
                        ZONE dummy( *zone );
                        dummy.SetNet( &nonet );
                        itemplotter.PlotFilledAreas( &dummy, islands );
                    }
                }
            }

            aPlotter->EndBlock( nullptr );

            // Adding drill marks, if required and if the plotter is able to plot them:
            if( aPlotOpt.GetDrillMarksType() != PCB_PLOT_PARAMS::NO_DRILL_SHAPE )
                itemplotter.PlotDrillMarks(); */

        } else {
            println!("non copper layer: {}", layer);
        }
        Ok(())
    }
}

impl<'a> PlotterImpl<'a, Pcb> for GerberPlotter<'a> {
    fn plot<W: Write + 'static>(
        &self,
        pcb: &Pcb,
        out: &mut W,
        border: bool,
        scale: f64,
        _pages: Option<Vec<usize>>,
        _netlist: bool,
    ) -> Result<(), Error> {
        use super::pcb::PcbPlotIterator;

        //TODO:
        let m_gerberUnitFmt = 1;
        let leadingDigitCount = 2;
        let m_gerberUnitInch = false;

        writeln!(
            out,
            "%TF.GenerationSoftware,{},{}*%",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )?;

        let rev = if !pcb.title_block.date.is_empty() {
            pcb.title_block.date.replace(",", "_").to_string()
        } else {
            String::from("rev?")
        };

        writeln!(
            out,
            "%TF.ProjectId,{},{},{}*%",
            pcb.title_block.title.replace(",", "_"),
            Self::guid(&pcb.filename().unwrap()),
            rev
        )?;

        // Add the TF.SameCoordinates, that specify all gerber files uses the same
        // origin and orientation, and the registration between files is OK.
        // The parameter of TF.SameCoordinates is a string that is common
        // to all files using the same registration and has no special meaning:
        // this is just a key
        // Because there is no mirroring/rotation in Kicad, only the plot offset origin
        // can create incorrect registration.
        // So we create a key from plot offset options.
        // and therefore for a given board, all Gerber files having the same key have the same
        // plot origin and use the same registration
        //
        // Currently the key is "Original" when using absolute Pcbnew coordinates,
        // and the PY and PY position of auxiliary axis, when using it.
        // Please, if absolute Pcbnew coordinates, one day, are set by user, change the way
        // the key is built to ensure file only using the *same* axis have the same key.
        /* wxString registration_id = wxT( "Original" );
        wxPoint auxOrigin = aBoard->GetDesignSettings().GetAuxOrigin();

        if( aBoard->GetPlotOptions().GetUseAuxOrigin() && auxOrigin.x && auxOrigin.y )
            registration_id.Printf( wxT( "PX%xPY%x" ), auxOrigin.x, auxOrigin.y ); */

        writeln!(out, "%TF.SameCoordinates,PX{}xPY{}*%", 100, 100)?; //registration_id.GetData() );

        // Set coordinate format to 3.6 or 4.5 absolute, leading zero omitted
        // the number of digits for the integer part of coordinates is needed
        // in gerber format, but is not very important when omitting leading zeros
        // It is fixed here to 3 (inch) or 4 (mm), but is not actually used
        //TODO: int leadingDigitCount = m_gerberUnitInch ? 3 : 4;
        let leadingDigitCount = 3;

        writeln!(
            out,
            "%FSLAX{}{}Y{}{}*%",
            leadingDigitCount, m_gerberUnitFmt, leadingDigitCount, m_gerberUnitFmt
        )?;
        writeln!(
            out,
            "G04 Gerber Fmt {}.{}, Leading zero omitted, Abs format (unit {})*",
            leadingDigitCount,
            m_gerberUnitFmt,
            if m_gerberUnitInch { "inch" } else { "mm" }
        )?;


        //Add the gerber function attribute
    /* let layer = "FAdhes".to_string();
    let attrib;
 */


        // In gerber files, ASCII7 chars only are allowed.
        // So use a ISO date format (using a space as separator between date and time),
        // not a localized date format
        //TODO: wxDateTime date = wxDateTime::Now();
        let datetime = Local::now();
        writeln!(
            out,
            "G04 Created by {} ({}) date {}*",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            datetime
        )?;

        /* Mass parameter: unit = INCHES/MM */
        if m_gerberUnitInch {
            writeln!(out, "%MOIN*%")?;
        } else {
            writeln!(out, "%MOMM*%")?;
        }

        // Be sure the usual dark polarity is selected:
        writeln!(out, "%LPD*%")?;

        // Set initial interpolation mode: always G01 (linear):
        writeln!(out, "G01*")?;

        // Add aperture list start point
        writeln!(out, "G04 APERTURE LIST*")?;

        // Give a minimal value to the default pen size, used to plot items in sketch mode
        /* if( m_renderSettings )
        {
            const int pen_min = 0.1 * m_IUsPerDecimil * 10000 / 25.4;   // for min width = 0.1 mm
            m_renderSettings->SetDefaultPenWidth( std::max( m_renderSettings->GetDefaultPenWidth(),
                                                            pen_min ) );
        } */


        for layer in pcb.layers().unwrap() {
            println!("Plot Layer: {}:{}", layer.canonical_name, layer.id.is_copper());
            self.plot_layer(pcb, layer.id)?;

        }

        Ok(())
    }
}
impl Outline for GerberPlotter<'_> {}

impl<'a> Draw<Context> for GerberPlotter<'a> {
    fn draw(&self, items: &Vec<PlotItem>, document: &mut Context) {
        items
            .iter()
            .sorted_by(|a, b| {
                let za = match a {
                    PlotItem::Arc(z, _) => z,
                    PlotItem::Line(z, _) => z,
                    PlotItem::Text(z, _) => z,
                    PlotItem::Circle(z, _) => z,
                    PlotItem::Polyline(z, _) => z,
                    PlotItem::Rectangle(z, _) => z,
                };
                let zb = match b {
                    PlotItem::Arc(z, _) => z,
                    PlotItem::Line(z, _) => z,
                    PlotItem::Text(z, _) => z,
                    PlotItem::Circle(z, _) => z,
                    PlotItem::Polyline(z, _) => z,
                    PlotItem::Rectangle(z, _) => z,
                };

                Ord::cmp(&za, &zb)
            })
            .for_each(|item| match item {
                PlotItem::Arc(_, arc) => self.item(arc, document),
                PlotItem::Circle(_, circle) => self.item(circle, document),
                PlotItem::Line(_, line) => self.item(line, document),
                PlotItem::Rectangle(_, rectangle) => self.item(rectangle, document),
                PlotItem::Polyline(_, line) => self.item(line, document),
                PlotItem::Text(_, text) => self.item(text, document),
            });
    }
}

impl<'a> Drawer<Text, Context> for GerberPlotter<'a> {
    fn item(&self, text: &Text, context: &mut Context) {
        context.save().unwrap();
        let layout = create_layout(&self.context);
        let markup = format!(
            "<span face=\"{}\" foreground=\"{}\" size=\"{}\">{}</span>",
            self.themer.font(Some(text.font.to_string()), &text.class),
            rgba_color(text.color),
            (self.themer.font_size(Some(text.fontsize), &text.class) * 1024.0) as i32,
            text.text
        );
        layout.set_markup(markup.as_str());
        update_layout(context, &layout);

        let outline: (i32, i32) = layout.size();
        let outline = (
            outline.0 as f64 / SCALE as f64,
            outline.1 as f64 / SCALE as f64,
        );
        let mut x = text.pos[0];
        let mut y = text.pos[1];

        if !text.label {
            if text.angle == 0.0 || text.angle == 180.0 {
                if text.align.contains(&String::from("right")) {
                    x -= outline.0 as f64;
                } else if !text.align.contains(&String::from("left")) {
                    x -= outline.0 as f64 / 2.0;
                }
                if text.align.contains(&String::from("bottom")) {
                    y -= outline.1 as f64;
                } else if !text.align.contains(&String::from("top")) {
                    y -= outline.1 as f64 / 2.0;
                }
            } else if text.angle == 90.0 || text.angle == 270.0 {
                if text.align.contains(&String::from("right")) {
                    y += outline.0 as f64;
                } else if !text.align.contains(&String::from("left")) {
                    y += outline.0 as f64 / 2.0;
                }
                if text.align.contains(&String::from("bottom")) {
                    x -= outline.1 as f64;
                } else if !text.align.contains(&String::from("top")) {
                    x -= outline.1 as f64 / 2.0;
                }
            } else {
                println!("text angle is: {} ({})", text.angle, text.text);
            }
            context.move_to(x, y);
            let angle = if text.angle >= 180.0 {
                text.angle - 180.0
            } else {
                text.angle
            };
            context.rotate(-angle * std::f64::consts::PI / 180.0);
            show_layout(context, &layout);
            context.stroke().unwrap();
        } else {
            let label_left = 0.4;
            let label_up = 0.1;
            let contur = arr2(&[
                [0.0, 0.],
                [2.0 * label_left, -outline.1 / 2.0 - label_up],
                [3.0 * label_left + outline.0, -outline.1 / 2.0 - label_up],
                [3.0 * label_left + outline.0, outline.1 / 2.0 + label_up],
                [2.0 * label_left, outline.1 / 2.0 + label_up],
                [0.0, 0.0],
            ]);
            let theta = -text.angle.to_radians();
            let rot = arr2(&[[theta.cos(), -theta.sin()], [theta.sin(), theta.cos()]]);
            let verts: Array2<f64> = contur.dot(&rot);
            let verts = &text.pos + verts;
            context.move_to(text.pos[0], text.pos[1]);
            for row in verts.rows() {
                context.line_to(row[0], row[1]);
            }
            context.stroke().unwrap();

            //adjust the text
            if text.angle == 0.0 {
                x += 2.0 * label_left;
                y -= outline.1 / 2.0;
            } else if text.angle == 180.0 {
                x -= 2.0 * label_left + outline.0;
                y -= outline.1 / 2.0;
            } //TODO 90, 270
            context.move_to(x, y);
            let angle = if text.angle >= 180.0 {
                text.angle - 180.0
            } else {
                text.angle
            };
            context.rotate(-angle * std::f64::consts::PI / 180.0);
            show_layout(context, &layout);
            context.stroke().unwrap();
        }
        context.restore().unwrap();
    }
}

impl<'a> Drawer<Line, Context> for GerberPlotter<'a> {
    fn item(&self, line: &Line, context: &mut Context) {
        stroke!(context, line, self.themer);
        /*TODO: match line.linecap {
            LineCap::Butt => context.set_line_cap(cairo::LineCap::Butt),
            LineCap::Round => context.set_line_cap(cairo::LineCap::Round),
            LineCap::Square => context.set_line_cap(cairo::LineCap::Square),
        } */
        context.move_to(line.pts[[0, 0]], line.pts[[0, 1]]);
        context.line_to(line.pts[[1, 0]], line.pts[[1, 1]]);
        context.stroke().unwrap();
    }
}

impl<'a> Drawer<Polyline, Context> for GerberPlotter<'a> {
    fn item(&self, line: &Polyline, context: &mut Context) {
        stroke!(context, line, self.themer);
        let mut first: bool = true;
        for pos in line.pts.rows() {
            if first {
                context.move_to(pos[0], pos[1]);
                first = false;
            } else {
                context.line_to(pos[0], pos[1]);
                context.stroke_preserve().unwrap();
            }
        }
        fill!(context, line, self.themer);
        context.stroke().unwrap()
    }
}

impl<'a> Drawer<Rectangle, Context> for GerberPlotter<'a> {
    fn item(&self, rectangle: &Rectangle, context: &mut Context) {
        stroke!(context, rectangle, self.themer);
        context.rectangle(
            rectangle.pts[[0, 0]],
            rectangle.pts[[0, 1]],
            rectangle.pts[[1, 0]] - rectangle.pts[[0, 0]],
            rectangle.pts[[1, 1]] - rectangle.pts[[0, 1]],
        );
        context.stroke_preserve().unwrap();
        fill!(context, rectangle, self.themer);
        context.stroke().unwrap()
    }
}

impl<'a> Drawer<Circle, Context> for GerberPlotter<'a> {
    fn item(&self, circle: &Circle, context: &mut Context) {
        stroke!(context, circle, self.themer);
        context.arc(circle.pos[0], circle.pos[1], circle.radius, 0., 10.);
        context.stroke_preserve().unwrap();
        fill!(context, circle, self.themer);
        context.stroke().unwrap()
    }
}

impl<'a> Drawer<Arc, Context> for GerberPlotter<'a> {
    fn item(&self, arc: &Arc, context: &mut Context) {
        /* TODO: stroke!(context, arc, self.themer);
        context.arc(arc.start[0], arc.start[1], arc.mid[1], 0., 10.);
        context.stroke_preserve().unwrap();
        fill!(context, arc, self.themer);
        context.stroke().unwrap() */
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::sexp::Schema;

    use super::super::plotter::{PlotterImpl, Theme};
    use super::{rgba_color, GerberPlotter};

    #[test]
    fn test_guid() {
        assert_eq!(
            "2f746d70-2f6d-4795-9f70-63622e6b6963",
            GerberPlotter::guid("/tmp/my_pcb.kicad.pcb")
        );
        assert_eq!(
            "6d792e6b-6963-4616-942e-706362585858",
            GerberPlotter::guid("my.kicad.pcb")
        );
    }
    /* #[test]
    fn plt_jfet() {
        let doc = Schema::load("files/jfet.kicad_sch").unwrap();
        let png = CairoPlotter::new(crate::plot::plotter::ImageType::Png, Theme::Kicad2020);

        let mut buffer = Vec::<u8>::new();
        let mut buffer = File::create("target/jfet.png").unwrap();
        png.plot(&doc, &mut buffer, true, 1.0, None, false).unwrap();

        // assert!(!buffer.is_empty());
    } */
}
