/*
 * Plot a solder mask layer.  Solder mask layers have a minimum thickness value and cannot be
 * drawn like standard layers, unless the minimum thickness is 0.
 */
/* static void PlotSolderMaskLayer( BOARD *aBoard, PLOTTER* aPlotter, LSET aLayerMask,
const PCB_PLOT_PARAMS& aPlotOpt, int aMinThickness ); */

use std::{cell::RefCell, rc::Rc};

use ndarray::arr1;

use super::{
    color4d::{COLOR4D, EDA_COLOR_T},
    gerber_plotter::{GERBER_PLOTTER, AddGerberX2Attribute},
    pcb_render_settings::PCB_RENDER_SETTINGS,
    plot_items::BRDITEMS_PLOTTER,
    plot_params::{OUTLINE_MODE, PCB_PLOT_PARAMS}, gerber_metadata::GBR_METADATA,
};
use crate::{
    error::Error,
    gerber::{plot_params::DrillMarksType, PlotFormat},
    sexp::{
        math_utils::GetArcToSegmentCount,
        shape_poly_set::{POLYGON_MODE, SHAPE_POLY_SET},
        LayerId, LayerSet, PadShape, Pcb, ERROR_LOC,
    },
};

pub fn PlotOneBoardLayer(
    aBoard: &mut Pcb,
    aPlotter: Rc<RefCell<GERBER_PLOTTER>>,
    aLayer: LayerId,
    aPlotOpt: &PCB_PLOT_PARAMS,
) {
    let mut plotOpt = PCB_PLOT_PARAMS::new(); //OLD: plotOpt = aPlotOpt;

    let soldermask_min_thickness = 0; //TODO: aBoard.GetDesignSettings().m_SolderMaskMinWidth;

    // Set a default color and the text mode for this layer
    /*TODO: aPlotter.SetColor( BLACK );
    aPlotter.SetTextMode( aPlotOpt.GetTextMode() ); */

    // Specify that the contents of the "Edges Pcb" layer are to be plotted in addition to the
    // contents of the currently specified layer.
    let mut layer_mask = LayerSet::from(aLayer);

    if !aPlotOpt.GetExcludeEdgeLayer() {
        layer_mask.set(LayerId::EdgeCuts);
    }

    if aLayer.is_copper() {
        plotOpt.SetSkipPlotNPTH_Pads(true);
        PlotStandardLayer(aBoard, aPlotter.clone(), layer_mask, &plotOpt);
    } else {
        use LayerId::*;
        match aLayer {
            BMask | FMask => {
                plotOpt.SetSkipPlotNPTH_Pads(false);
                // Disable plot pad holes
                plotOpt.SetDrillMarksType(DrillMarksType::NO_DRILL_SHAPE);

                // Plot solder mask:
                if soldermask_min_thickness == 0 {
                    PlotStandardLayer(aBoard, aPlotter.clone(), layer_mask, &plotOpt);
                } else {
                    PlotSolderMaskLayer(
                        aBoard,
                        aPlotter,
                        layer_mask,
                        &plotOpt,
                        soldermask_min_thickness,
                    );
                }
            }

            BAdhes | FAdhes | BPaste | FPaste => {
                plotOpt.SetSkipPlotNPTH_Pads(false);
                // Disable plot pad holes
                plotOpt.SetDrillMarksType(DrillMarksType::NO_DRILL_SHAPE);
                PlotStandardLayer(aBoard, aPlotter.clone(), layer_mask, &plotOpt);
            }

            FSilkS | BSilkS => {
                PlotStandardLayer(aBoard, aPlotter.clone(), layer_mask, &plotOpt);

                // Gerber: Subtract soldermask from silkscreen if enabled
                if aPlotter.borrow().GetPlotterType() == PlotFormat::GERBER
                    && plotOpt.GetSubtractMaskFromSilk() {
                    if aLayer == LayerId::FSilkS {
                        layer_mask = LayerSet::from(FMask);
                    } else {
                        layer_mask = LayerSet::from(BMask);
                    }

                    // Create the mask to subtract by creating a negative layer polarity
                    aPlotter.borrow_mut().SetLayerPolarity(false);

                    // Disable plot pad holes
                    plotOpt.SetDrillMarksType(DrillMarksType::NO_DRILL_SHAPE);

                    // Plot the mask
                    PlotStandardLayer(aBoard, aPlotter.clone(), layer_mask, &plotOpt);
                }
            }

            // These layers are plotted like silk screen layers.
            // Mainly, pads on these layers are not filled.
            // This is not necessary the best choice.
            DwgsUser | CmtsUser | Eco1User | Eco2User | EdgeCuts | Margin | FCrtYd | BCrtYd
            | FFab | BFab => {
                plotOpt.SetSkipPlotNPTH_Pads(false);
                plotOpt.SetDrillMarksType(DrillMarksType::NO_DRILL_SHAPE);
                PlotStandardLayer(aBoard, aPlotter.clone(), layer_mask, &plotOpt);
            }

            _ => {
                plotOpt.SetSkipPlotNPTH_Pads(false);
                plotOpt.SetDrillMarksType(DrillMarksType::NO_DRILL_SHAPE);
                PlotStandardLayer(aBoard, aPlotter.clone(), layer_mask, &plotOpt);
            }
        }
    }
}

/**
 * Plot a copper layer or mask.
 *
 * Silk screen layers are not plotted here.
 */
fn PlotStandardLayer(
    aBoard: &mut Pcb,
    aPlotter: Rc<RefCell<GERBER_PLOTTER>>,
    aLayerMask: LayerSet,
    aPlotOpt: &PCB_PLOT_PARAMS,
) {
    let mut itemplotter = BRDITEMS_PLOTTER::new(aBoard, aPlotter.clone(), aPlotOpt);

    itemplotter.SetLayerSet(aLayerMask.clone());

    let plotMode = aPlotOpt.GetPlotMode();
    /* bool onCopperLayer = ( LSET::AllCuMask() & aLayerMask ).any();
    bool onSolderMaskLayer = ( LSET( 2, F_Mask, B_Mask ) & aLayerMask ).any();
    bool onSolderPasteLayer = ( LSET( 2, F_Paste, B_Paste ) & aLayerMask ).any();
    bool onFrontFab = ( LSET(  F_Fab ) & aLayerMask ).any();
    bool onBackFab  = ( LSET( B_Fab ) & aLayerMask ).any();
    bool sketchPads = ( onFrontFab || onBackFab ) && aPlotOpt.GetSketchPadsOnFabLayers(); */

    // Plot edge layer and graphic items
    itemplotter.PlotBoardGraphicItems();

    // Draw footprint texts:
    for fp in aBoard.footprints() {
        itemplotter.PlotFootprintTextItems(fp);
    }

    // Draw footprint other graphic items:
    for fp in aBoard.footprints() {
        itemplotter.PlotFootprintGraphicItems(fp);
    }

    // Plot footprint pads
    for fp in aBoard.footprints() {
        aPlotter.borrow_mut().StartBlock();

        // for( PAD* pad : footprint->Pads() ) {
        for mut pad in &mut fp.pads.iter().cloned() {
            let padPlotMode: OUTLINE_MODE = plotMode;

            /* if( !( pad.GetLayerSet() & aLayerMask ).any() ) {
                if( sketchPads &&
                        ( ( onFrontFab && pad->GetLayerSet().Contains( F_Cu ) ) ||
                          ( onBackFab && pad->GetLayerSet().Contains( B_Cu ) ) ) )
                {
                    padPlotMode = SKETCH;
                }
                else {
                    continue;
                }
            } */

            // pads not connected to copper are optionally not drawn
            /* if onCopperLayer && !pad.FlashLayer( aLayerMask ) {
                continue;
            } */

            let color = COLOR4D::from_type(EDA_COLOR_T::BLACK);

            /* if( ( pad->GetLayerSet() & aLayerMask )[B_Cu] )
               color = aPlotOpt.ColorSettings()->GetColor( B_Cu );

            if( ( pad->GetLayerSet() & aLayerMask )[F_Cu] )
                color = color.LegacyMix( aPlotOpt.ColorSettings()->GetColor( F_Cu ) );

            if( sketchPads && aLayerMask[F_Fab] )
                color = aPlotOpt.ColorSettings()->GetColor( F_Fab );
            else if( sketchPads && aLayerMask[B_Fab] )
                color = aPlotOpt.ColorSettings()->GetColor( B_Fab );
            */

            let width_adj = 0.0;

            /* if( onCopperLayer ) {
                width_adj = itemplotter.getFineWidthAdj();
            } */

            let mut margin = arr1(&[0.0, 0.0]);
            if aLayerMask.on_solder_mask_layer() {
                let m = pad.GetSolderMaskMargin();
                margin = arr1(&[m, m]);
            }

            if aLayerMask.on_solder_paste_layer() {
                margin = pad.GetSolderPasteMargin();
            }

            // not all shapes can have a different margin for x and y axis
            // in fact only oval and rect shapes can have different values.
            // Round shape have always the same x,y margin
            // so define a unique value for other shapes that do not support different values
            let mask_clearance = margin[0]; //x

            // Now offset the pad size by margin + width_adj
            let padPlotsSize = pad.size.clone() + margin * 2.0 + arr1(&[width_adj, width_adj]);

            // Store these parameters that can be modified to plot inflated/deflated pads shape
            let padShape = pad.padshape.clone();
            let padSize = pad.size.clone();
            let padDelta = arr1(&[0.0, 0.0]); //TODO: pad->GetDelta(); // has meaning only for trapezoidal pads
                                              // let padCornerRadius = pad->GetRoundRectCornerRadius();

            // Don't draw a 0 sized pad.
            // Note: a custom pad can have its pad anchor with size = 0
            if padShape != PadShape::Custom && (padPlotsSize[0] <= 0.0 || padPlotsSize[1] <= 0.0) {
                continue;
            }

            match padShape {
                PadShape::Circle | PadShape::Oval => {
                    pad.size = padPlotsSize;

                    /* if( aPlotOpt.GetSkipPlotNPTH_Pads() &&
                        ( aPlotOpt.GetDrillMarksType() == PCB_PLOT_PARAMS::NO_DRILL_SHAPE ) &&
                        ( pad->GetSize() == pad->GetDrillSize() ) &&
                        ( pad->GetAttribute() == PAD_ATTRIB::NPTH ) )
                    {
                        break;
                    } */

                    itemplotter.PlotPad(&pad, &color, padPlotMode);
                }

                PadShape::Rect => {
                    pad.size = padPlotsSize;

                    if mask_clearance > 0.0 {
                        pad.padshape = PadShape::RoundRect;
                        pad.SetRoundRectCornerRadius(mask_clearance);
                    }

                    itemplotter.PlotPad(&pad, &color, padPlotMode);
                }

                PadShape::Trapezoid => {
                    // inflate/deflate a trapezoid is a bit complex.
                    // so if the margin is not null, build a similar polygonal pad shape,
                    // and inflate/deflate the polygonal shape
                    // because inflating/deflating using different values for y and y
                    // we are using only margin.x as inflate/deflate value
                    if mask_clearance == 0.0 {
                        itemplotter.PlotPad(&pad, &color, padPlotMode);
                    } else {
                        let mut dummy = pad.clone();
                        dummy.SetAnchorPadShape(PadShape::Circle);
                        dummy.padshape = PadShape::Custom;
                        let mut outline = SHAPE_POLY_SET{};
                        outline.NewOutline();
                        let dx = padSize[0] / 2.0;
                        let dy = padSize[1] / 2.0;
                        let ddx = padDelta[0] / 2.0;
                        let ddy = padDelta[1] / 2.0;

                        outline.Append(-dx - ddy, dy + ddx);
                        outline.Append(dx + ddy, dy - ddx);
                        outline.Append(dx - ddy, -dy + ddx);
                        outline.Append(-dx + ddy, -dy - ddx);

                        // Shape polygon can have holes so use InflateWithLinkedHoles(), not Inflate()
                        // which can create bad shapes if margin.x is < 0
                        let maxError = 0; //TODO: aBoard->GetDesignSettings().m_MaxError;
                        let numSegs = GetArcToSegmentCount(mask_clearance, maxError, 360.0);
                        outline.InflateWithLinkedHoles(
                            mask_clearance,
                            numSegs,
                            POLYGON_MODE::PM_FAST,
                        );
                        dummy.DeletePrimitivesList();
                        dummy.AddPrimitivePoly(&outline, 0, true);

                        // Be sure the anchor pad is not bigger than the deflated shape because this
                        // anchor will be added to the pad shape when plotting the pad. So now the
                        // polygonal shape is built, we can clamp the anchor size
                        dummy.size = arr1(&[0.0, 0.0]);

                        itemplotter.PlotPad(&dummy, &color, padPlotMode);
                    }
                }

                PadShape::RoundRect => {
                    // rounding is stored as a percent, but we have to change the new radius
                    // to initial_radius + clearance to have a inflated/deflated similar shape
                    let initial_radius = pad.GetRoundRectCornerRadius();
                    pad.size = padPlotsSize;
                    pad.SetRoundRectCornerRadius(if initial_radius + mask_clearance > 0.0 {
                        initial_radius + mask_clearance
                    } else {
                        0.0
                    });

                    itemplotter.PlotPad(&pad, &color, padPlotMode);
                }

                PadShape::ChamferedRect => {
                    if mask_clearance == 0.0 {
                        // the size can be slightly inflated by width_adj (PS/PDF only)
                        pad.size = padPlotsSize;
                        itemplotter.PlotPad(&pad, &color, padPlotMode);
                    } else {
                        // Due to the polygonal shape of a CHAMFERED_RECT pad, the best way is to
                        // convert the pad shape to a full polygon, inflate/deflate the polygon
                        // and use a dummy  CUSTOM pad to plot the final shape.
                        let mut dummy = pad.clone();
                        // Build the dummy pad outline with coordinates relative to the pad position
                        // and orientation 0. The actual pos and rotation will be taken in account
                        // later by the plot function
                        dummy.at = arr1(&[0.0, 0.0]);
                        dummy.angle = 0.0;
                        let outline = SHAPE_POLY_SET {};
                        let maxError = 0; //TODO: aBoard->GetDesignSettings().m_MaxError;
                        let numSegs = GetArcToSegmentCount(mask_clearance, maxError, 360.0);
                        dummy.TransformShapeWithClearanceToPolygon(
                            &outline,
                            LayerId::Undefined,
                            0,
                            maxError,
                            ERROR_LOC::ERROR_INSIDE,
                            false,
                        );
                        outline.InflateWithLinkedHoles(
                            mask_clearance,
                            numSegs,
                            POLYGON_MODE::PM_FAST,
                        );

                        // Initialize the dummy pad shape:
                        dummy.anchor_padshape = PadShape::Circle;
                        dummy.padshape = PadShape::Custom;
                        dummy.DeletePrimitivesList();
                        dummy.AddPrimitivePoly(&outline, 0, true);

                        // Be sure the anchor pad is not bigger than the deflated shape because this
                        // anchor will be added to the pad shape when plotting the pad.
                        // So we set the anchor size to 0
                        dummy.size = arr1(&[0.0, 0.0]);
                        dummy.at = pad.at;
                        dummy.angle = pad.angle;

                        itemplotter.PlotPad(&dummy, &color, padPlotMode);
                    }
                }

                PadShape::Custom => {
                    // inflate/deflate a custom shape is a bit complex.
                    // so build a similar pad shape, and inflate/deflate the polygonal shape
                    let mut dummy = pad.clone();
                    let shape = SHAPE_POLY_SET {};
                    pad.MergePrimitivesAsPolygon(&shape, ERROR_LOC::ERROR_INSIDE);

                    // Shape polygon can have holes so use InflateWithLinkedHoles(), not Inflate()
                    // which can create bad shapes if margin.x is < 0
                    let maxError = 0; //TODO: aBoard->GetDesignSettings().m_MaxError;
                    let numSegs = GetArcToSegmentCount(mask_clearance, maxError, 360.0);
                    shape.InflateWithLinkedHoles(mask_clearance, numSegs, POLYGON_MODE::PM_FAST);
                    dummy.DeletePrimitivesList();
                    dummy.AddPrimitivePoly(&shape, 0, true);

                    // Be sure the anchor pad is not bigger than the deflated shape because this
                    // anchor will be added to the pad shape when plotting the pad. So now the
                    // polygonal shape is built, we can clamp the anchor size
                    if mask_clearance < 0.0 {
                        // we expect margin.x = margin.y for custom pads
                        dummy.size = padPlotsSize;
                    }

                    itemplotter.PlotPad(&dummy, &color, padPlotMode);
                }
            }

            // Restore the pad parameters modified by the plot code
            /* pad->SetSize( padSize );
            pad->SetDelta( padDelta );
            pad->SetShape( padShape );
            pad->SetRoundRectCornerRadius( padCornerRadius ); */
        }

        aPlotter.borrow_mut().EndBlock();
    }

    // Plot vias on copper layers, and if aPlotOpt.GetPlotViaOnMaskLayer() is true,
    // plot them on solder mask

    let mut gbr_metadata = GBR_METADATA::new();

    /* bool isOnCopperLayer = ( aLayerMask & LSET::AllCuMask() ).any();

    if( isOnCopperLayer )
    {
        gbr_metadata.SetApertureAttrib( GBR_APERTURE_METADATA::GBR_APERTURE_ATTRIB_VIAPAD );
        gbr_metadata.SetNetAttribType( GBR_NETLIST_METADATA::GBR_NETINFO_NET );
    } */

    aPlotter.borrow_mut().StartBlock();

    /*
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
    } */

    aPlotter.borrow_mut().EndBlock();
    aPlotter.borrow_mut().StartBlock();
    // gbr_metadata.SetApertureAttrib( GBR_APERTURE_METADATA::GBR_APERTURE_ATTRIB_CONDUCTOR );

    // Plot tracks (not vias) :
    for track in aBoard.segements() {
        /* if( track->Type() == PCB_VIA_T )
        continue; */

        /* if( !aLayerMask[track->GetLayer()] )
        continue; */

        // Some track segments can be not connected (no net).
        // Set the m_NotInNet for these segments to force a empty net name in gerber file
        // gbr_metadata.m_NetlistMetadata.m_NotInNet = track->GetNetname().IsEmpty();

        // gbr_metadata.SetNetName( track->GetNetname() );
        let width = track.width; //TODO: only used for PS + itemplotter.getFineWidthAdj();
        // aPlotter.SetColor( itemplotter.getColor( track.layer ) );

        /* if( track->Type() == PCB_ARC_T )
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
        { */
            aPlotter.borrow_mut().ThickSegment( track.start.clone(), track.end.clone(), width, plotMode,
                                    &gbr_metadata );
        // }
    }

    aPlotter.borrow_mut().EndBlock();

    // Plot filled ares
    aPlotter.borrow_mut().StartBlock();

    //TODO: NETINFO_ITEM nonet( aBoard );

    // for( const ZONE* zone : aBoard->Zones() )
    for zone in aBoard.zones() {

        for layer in &zone.layers { //.->GetLayerSet().Seq() ) {
            //if !aLayerMask[layer] )
            if !aLayerMask.contains(*layer) {
                continue; 
            }

            let mut mainArea = zone.GetFilledPolysList( *layer );
            let mut islands = SHAPE_POLY_SET{};

            //for( int i = mainArea.OutlineCount() - 1; i >= 0; i-- )
            for i in 0..mainArea.OutlineCount() {
                if zone.IsIsland( layer, i ) {
                    islands.AddOutline( &mainArea.CPolygon( i )[0] );
                    mainArea.DeletePolygon( i );
                }
            }

            itemplotter.PlotFilledAreas( zone, &mainArea );

            if !islands.IsEmpty() {
                let dummy = zone.clone();
                //TODO: dummy.net = &nonet;
                itemplotter.PlotFilledAreas( &dummy, &islands );
            }
        }
    }

    aPlotter.borrow_mut().EndBlock();

/*
    // Adding drill marks, if required and if the plotter is able to plot them:
    if( aPlotOpt.GetDrillMarksType() != PCB_PLOT_PARAMS::NO_DRILL_SHAPE )
        itemplotter.PlotDrillMarks(); */
}

/*
// Seems like we want to plot from back to front?
static const PCB_LAYER_ID plot_seq[] = {

    User_9,
    User_8,
    User_7,
    User_6,
    User_5,
    User_4,
    User_3,
    User_2,
    User_1,
    B_Adhes,
    F_Adhes,
    B_Paste,
    F_Paste,
    B_SilkS,
    B_Mask,
    F_Mask,
    Dwgs_User,
    Cmts_User,
    Eco1_User,
    Eco2_User,
    Edge_Cuts,
    Margin,

    F_CrtYd,        // CrtYd & Body are footprint only
    B_CrtYd,
    F_Fab,
    B_Fab,

    B_Cu,
    In30_Cu,
    In29_Cu,
    In28_Cu,
    In27_Cu,
    In26_Cu,
    In25_Cu,
    In24_Cu,
    In23_Cu,
    In22_Cu,
    In21_Cu,
    In20_Cu,
    In19_Cu,
    In18_Cu,
    In17_Cu,
    In16_Cu,
    In15_Cu,
    In14_Cu,
    In13_Cu,
    In12_Cu,
    In11_Cu,
    In10_Cu,
    In9_Cu,
    In8_Cu,
    In7_Cu,
    In6_Cu,
    In5_Cu,
    In4_Cu,
    In3_Cu,
    In2_Cu,
    In1_Cu,
    F_Cu,

    F_SilkS,
};
*/

/**
 * Plot outlines of copper layer.
 */
/* pub fn PlotLayerOutlines(aBoard: &mut Pcb, aPlotter: Rc<RefCell<GERBER_PLOTTER>>, aLayerMask: LayerSet,
                        aPlotOpt: &PCB_PLOT_PARAMS ) {

    let mut itemplotter = BRDITEMS_PLOTTER::new(aBoard, aPlotter.clone(), aPlotOpt);
    itemplotter.SetLayerSet(aLayerMask.clone());

    let mut outlines = SHAPE_POLY_SET::new();

    // for( LSEQ seq = aLayerMask.Seq( plot_seq, arrayDim( plot_seq ) );  seq;  ++seq )
    for( LSEQ seq = aLayerMask.Seq( plot_seq, arrayDim( plot_seq ) );  seq;  ++seq )
    {
        PCB_LAYER_ID layer = *seq;

        outlines.RemoveAllContours();
        aBoard->ConvertBrdLayerToPolygonalContours( layer, outlines );

        outlines.Simplify( SHAPE_POLY_SET::PM_FAST );

        // Plot outlines
        std::vector<wxPoint> cornerList;

        // Now we have one or more basic polygons: plot each polygon
        for( int ii = 0; ii < outlines.OutlineCount(); ii++ )
        {
            for( int kk = 0; kk <= outlines.HoleCount(ii); kk++ )
            {
                cornerList.clear();
                const SHAPE_LINE_CHAIN& path =
                        ( kk == 0 ) ? outlines.COutline( ii ) : outlines.CHole( ii, kk - 1 );

                aPlotter->PlotPoly( path, FILL_T::NO_FILL );
            }
        }

        // Plot pad holes
        if( aPlotOpt.GetDrillMarksType() != PCB_PLOT_PARAMS::NO_DRILL_SHAPE )
        {
            int smallDrill = (aPlotOpt.GetDrillMarksType() == PCB_PLOT_PARAMS::SMALL_DRILL_SHAPE)
                                  ? ADVANCED_CFG::GetCfg().m_SmallDrillMarkSize : INT_MAX;

            for( FOOTPRINT* footprint : aBoard->Footprints() )
            {
                for( PAD* pad : footprint->Pads() )
                {
                    wxSize hole = pad->GetDrillSize();

                    if( hole.x == 0 || hole.y == 0 )
                        continue;

                    if( hole.x == hole.y )
                    {
                        hole.x = std::min( smallDrill, hole.x );
                        aPlotter->Circle( pad->GetPosition(), hole.x, FILL_T::NO_FILL );
                    }
                    else
                    {
                        // Note: small drill marks have no significance when applied to slots
                        const SHAPE_SEGMENT* seg = pad->GetEffectiveHoleShape();
                        aPlotter->ThickSegment( (wxPoint) seg->GetSeg().A,
                                                (wxPoint) seg->GetSeg().B,
                                                seg->GetWidth(), SKETCH, nullptr );
                    }
                }
            }
        }

        // Plot vias holes
        for( PCB_TRACK* track : aBoard->Tracks() )
        {
            const PCB_VIA* via = dyn_cast<const PCB_VIA*>( track );

            if( via && via->IsOnLayer( layer ) )    // via holes can be not through holes
                aPlotter->Circle( via->GetPosition(), via->GetDrillValue(), FILL_T::NO_FILL );
        }
    }
} */

/**
 * Plot a solder mask layer.
 *
 * Solder mask layers have a minimum thickness value and cannot be drawn like standard layers,
 * unless the minimum thickness is 0.
 * Currently the algo is:
 * 1 - build all pad shapes as polygons with a size inflated by
 *      mask clearance + (min width solder mask /2)
 * 2 - Merge shapes
 * 3 - deflate result by (min width solder mask /2)
 * 4 - ORing result by all pad shapes as polygons with a size inflated by
 *      mask clearance only (because deflate sometimes creates shape artifacts)
 * 5 - draw result as polygons
 *
 * We have 2 algos:
 * the initial algo, that create polygons for every shape, inflate and deflate polygons
 * with Min Thickness/2, and merges the result.
 * Drawback: pads attributes are lost (annoying in Gerber)
 * the new algo:
 * create initial polygons for every shape (pad or polygon),
 * inflate and deflate polygons
 * with Min Thickness/2, and merges the result (like initial algo)
 * remove all initial polygons.
 * The remaining polygons are areas with thickness < min thickness
 * plot all initial shapes by flashing (or using regions) for pad and polygons
 * (shapes will be better) and remaining polygons to
 * remove areas with thickness < min thickness from final mask
 *
 * TODO: remove old code after more testing.
 */
// #define NEW_ALGO 1

pub fn PlotSolderMaskLayer(
    aBoard: &Pcb,
    aPlotter: Rc<RefCell<GERBER_PLOTTER>>,
    aLayerMask: LayerSet,
    aPlotOpt: &PCB_PLOT_PARAMS,
    aMinThickness: u32,
) {
    let maxError = 0; //TODO: aBoard->GetDesignSettings().m_MaxError;
    let layer = if aLayerMask.contains(LayerId::BMask) {
        LayerId::BMask
    } else {
        LayerId::FMask
    };
    /* SHAPE_POLY_SET  buffer;
    SHAPE_POLY_SET* boardOutline = nullptr; */

    /* if aBoard.GetBoardPolygonOutlines( buffer ) {
        boardOutline = &buffer;
    } */

    // We remove 1nm as we expand both sides of the shapes, so allowing for
    // a strictly greater than or equal comparison in the shape separation (boolean add)
    // means that we will end up with separate shapes that then are shrunk
    let inflate = aMinThickness / 2 - 1;

    let mut itemplotter = BRDITEMS_PLOTTER::new(aBoard, aPlotter.clone(), aPlotOpt);
    itemplotter.SetLayerSet(aLayerMask);

    // Plot edge layer and graphic items.
    // They do not have a solder Mask margin, because they are graphic items
    // on this layer (like logos), not actually areas around pads.

    itemplotter.PlotBoardGraphicItems();

    /* // for( FOOTPRINT* footprint : aBoard->Footprints() )
        for footprint in aBoard.footprint() {
            itemplotter.PlotFootprintTextItems( footprint );

            /*TODO: for( BOARD_ITEM* item : footprint->GraphicalItems() )
            {
                if( item->Type() == PCB_FP_SHAPE_T && item->GetLayer() == layer )
                    itemplotter.PlotFootprintGraphicItem( (FP_SHAPE*) item );
            } */
        }

        // Build polygons for each pad shape.  The size of the shape on solder mask should be size
        // of pad + clearance around the pad, where clearance = solder mask clearance + extra margin.
        // Extra margin is half the min width for solder mask, which is used to merge too-close shapes
        // (distance < aMinThickness), and will be removed when creating the actual shapes.

        // Will contain shapes inflated by inflate value that will be merged and deflated by
        // inflate value to build final polygons
        // After calculations the remaining polygons are polygons to plot
        SHAPE_POLY_SET areas;

        // Will contain exact shapes of all items on solder mask
        SHAPE_POLY_SET initialPolys;

    #if NEW_ALGO
        // Generate polygons with arcs inside the shape or exact shape
        // to minimize shape changes created by arc to segment size correction.
        DISABLE_ARC_RADIUS_CORRECTION disabler;
    #endif
        {
            // Plot pads
            for( FOOTPRINT* footprint : aBoard->Footprints() )
            {
                // add shapes with their exact mask layer size in initialPolys
                footprint->TransformPadsWithClearanceToPolygon( initialPolys, layer, 0, maxError,
                                                                ERROR_OUTSIDE );
                // add shapes inflated by aMinThickness/2 in areas
                footprint->TransformPadsWithClearanceToPolygon( areas, layer, inflate, maxError,
                                                                ERROR_OUTSIDE );
            }

            // Plot vias on solder masks, if aPlotOpt.GetPlotViaOnMaskLayer() is true,
            if( aPlotOpt.GetPlotViaOnMaskLayer() )
            {
                // The current layer is a solder mask, use the global mask clearance for vias
                int via_clearance = aBoard->GetDesignSettings().m_SolderMaskMargin;
                int via_margin = via_clearance + inflate;

                for( PCB_TRACK* track : aBoard->Tracks() )
                {
                    const PCB_VIA* via = dyn_cast<const PCB_VIA*>( track );

                    if( !via )
                        continue;

                    // vias are plotted only if they are on the corresponding external copper layer
                    LSET via_set = via->GetLayerSet();

                    if( via_set[B_Cu] )
                        via_set.set( B_Mask );

                    if( via_set[F_Cu] )
                        via_set.set( F_Mask );

                    if( !( via_set & aLayerMask ).any() )
                        continue;

                    // add shapes with their exact mask layer size in initialPolys
                    via->TransformShapeWithClearanceToPolygon( initialPolys, layer, via_clearance,
                                                               maxError, ERROR_OUTSIDE );
                    // add shapes inflated by aMinThickness/2 in areas
                    via->TransformShapeWithClearanceToPolygon( areas, layer, via_margin, maxError,
                                                               ERROR_OUTSIDE );
                }
            }

            // Add filled zone areas.
    #if 0   // Set to 1 if a solder mask margin must be applied to zones on solder mask
            int zone_margin = aBoard->GetDesignSettings().m_SolderMaskMargin;
    #else
            int zone_margin = 0;
    #endif

            for( ZONE* zone : aBoard->Zones() )
            {
                if( !zone->IsOnLayer( layer ) )
                    continue;

                // add shapes inflated by aMinThickness/2 in areas
                zone->TransformSmoothedOutlineToPolygon( areas, inflate + zone_margin, maxError,
                                                         ERROR_OUTSIDE, boardOutline );

                // add shapes with their exact mask layer size in initialPolys
                zone->TransformSmoothedOutlineToPolygon( initialPolys, zone_margin, maxError,
                                                         ERROR_OUTSIDE, boardOutline );
            }
        }

        int numSegs = GetArcToSegmentCount( inflate, maxError, 360.0 );

        // Merge all polygons: After deflating, not merged (not overlapping) polygons
        // will have the initial shape (with perhaps small changes due to deflating transform)
        areas.Simplify( SHAPE_POLY_SET::PM_STRICTLY_SIMPLE );
        areas.Deflate( inflate, numSegs );

    #if !NEW_ALGO
        // To avoid a lot of code, use a ZONE to handle and plot polygons, because our polygons look
        // exactly like filled areas in zones.
        // Note, also this code is not optimized: it creates a lot of copy/duplicate data.
        // However it is not complex, and fast enough for plot purposes (copy/convert data is only a
        // very small calculation time for these calculations).
        ZONE zone( aBoard );
        zone.SetMinThickness( 0 );      // trace polygons only
        zone.SetLayer( layer );

        // Combine the current areas to initial areas. This is mandatory because inflate/deflate
        // transform is not perfect, and we want the initial areas perfectly kept
        areas.BooleanAdd( initialPolys, SHAPE_POLY_SET::PM_FAST );
        areas.Fracture( SHAPE_POLY_SET::PM_STRICTLY_SIMPLE );

        itemplotter.PlotFilledAreas( &zone, areas );
    #else

        // Remove initial shapes: each shape will be added later, as flashed item or region
        // with a suitable attribute.
        // Do not merge pads is mandatory in Gerber files: They must be identified as pads

        // we deflate areas in polygons, to avoid after subtracting initial shapes
        // having small artifacts due to approximations during polygon transforms
        areas.BooleanSubtract( initialPolys, SHAPE_POLY_SET::PM_STRICTLY_SIMPLE );

        // Slightly inflate polygons to avoid any gap between them and other shapes,
        // These gaps are created by arc to segments approximations
        areas.Inflate( Millimeter2iu( 0.002 ), 6 );

        // Now, only polygons with a too small thickness are stored in areas.
        areas.Fracture( SHAPE_POLY_SET::PM_STRICTLY_SIMPLE );

        // Plot each initial shape (pads and polygons on mask layer), with suitable attributes:
        PlotStandardLayer( aBoard, aPlotter, aLayerMask, aPlotOpt );

        for( int ii = 0; ii < areas.OutlineCount(); ii++ )
        {
            const SHAPE_LINE_CHAIN& path = areas.COutline( ii );

            // polygon area in mm^2 :
            double curr_area = path.Area() / ( IU_PER_MM * IU_PER_MM );

            // Skip very small polygons: they are certainly artifacts created by
            // arc approximations and polygon transforms
            // (inflate/deflate transforms)
            constexpr double poly_min_area_mm2 = 0.01;     // 0.01 mm^2 gives a good filtering

            if( curr_area < poly_min_area_mm2 )
                continue;

            aPlotter->PlotPoly( path, FILL_T::FILLED_SHAPE );
        }
    #endif
        */
}

/**
 * Set up most plot options for plotting a board (especially the viewport)
 * Important thing:
 *      page size is the 'drawing' page size,
 *      paper size is the physical page size
 */
pub fn initializePlotter(aPlotter: Rc<RefCell<GERBER_PLOTTER>>, aBoard: &Pcb, aPlotOpts: &PCB_PLOT_PARAMS) {
    /* PAGE_INFO pageA4( wxT( "A4" ) );
    const PAGE_INFO& pageInfo = aBoard->GetPageSettings();
    const PAGE_INFO* sheet_info;
    double paperscale; // Page-to-paper ratio
    wxSize paperSizeIU;
    wxSize pageSizeIU( pageInfo.GetSizeIU() );
    bool autocenter = false;

    // Special options: to fit the sheet to an A4 sheet replace the paper size. However there
    // is a difference between the autoscale and the a4paper option:
    //  - Autoscale fits the board to the paper size
    //  - A4paper fits the original paper size to an A4 sheet
    //  - Both of them fit the board to an A4 sheet
    if( aPlotOpts->GetA4Output() )
    {
        sheet_info  = &pageA4;
        paperSizeIU = pageA4.GetSizeIU();
        paperscale  = (double) paperSizeIU.x / pageSizeIU.x;
        autocenter  = true;
    }
    else
    {
        sheet_info  = &pageInfo;
        paperSizeIU = pageSizeIU;
        paperscale  = 1;

        // Need autocentering only if scale is not 1:1
        autocenter  = (aPlotOpts->GetScale() != 1.0);
    }

    EDA_RECT bbox = aBoard->ComputeBoundingBox();
    wxPoint boardCenter = bbox.Centre();
    wxSize boardSize = bbox.GetSize();

    double compound_scale;

    // Fit to 80% of the page if asked; it could be that the board is empty, in this case
    // regress to 1:1 scale
    if( aPlotOpts->GetAutoScale() && boardSize.x > 0 && boardSize.y > 0 )
    {
        double xscale = (paperSizeIU.x * 0.8) / boardSize.x;
        double yscale = (paperSizeIU.y * 0.8) / boardSize.y;

        compound_scale = std::min( xscale, yscale ) * paperscale;
    }
    else
    {
        compound_scale = aPlotOpts->GetScale() * paperscale;
    }

    // For the plot offset we have to keep in mind the auxiliary origin too: if autoscaling is
    // off we check that plot option (i.e. autoscaling overrides auxiliary origin)
    wxPoint offset( 0, 0);

    if( autocenter )
    {
        offset.x = KiROUND( boardCenter.x - ( paperSizeIU.x / 2.0 ) / compound_scale );
        offset.y = KiROUND( boardCenter.y - ( paperSizeIU.y / 2.0 ) / compound_scale );
    }
    else
    {
        if( aPlotOpts->GetUseAuxOrigin() )
            offset = aBoard->GetDesignSettings().GetAuxOrigin();
    }

    aPlotter->SetPageSettings( *sheet_info );

    aPlotter->SetViewport( offset, IU_PER_MILS/10, compound_scale, aPlotOpts->GetMirror() );

    // Has meaning only for gerber plotter. Must be called only after SetViewport
    aPlotter->SetGerberCoordinatesFormat( aPlotOpts->GetGerberPrecision() );

    // Has meaning only for SVG plotter. Must be called only after SetViewport
    aPlotter->SetSvgCoordinatesFormat( aPlotOpts->GetSvgPrecision(), aPlotOpts->GetSvgUseInch() );

    aPlotter->SetCreator( wxT( "PCBNEW" ) );
    aPlotter->SetColorMode( false );        // default is plot in Black and White.
    aPlotter->SetTextMode( aPlotOpts->GetTextMode() ); */
}

/**
 * Prefill in black an area a little bigger than the board to prepare for the negative plot
 */
/* static void FillNegativeKnockout( PLOTTER *aPlotter, const EDA_RECT &aBbbox )
{
    const int margin = 5 * IU_PER_MM;   // Add a 5 mm margin around the board
    aPlotter->SetNegative( true );
    aPlotter->SetColor( WHITE );        // Which will be plotted as black
    EDA_RECT area = aBbbox;
    area.Inflate( margin );
    aPlotter->Rect( area.GetOrigin(), area.GetEnd(), FILL_T::FILLED_SHAPE );
    aPlotter->SetColor( BLACK );
} */


/**
 * Calculate the effective size of HPGL pens and set them in the plotter object
 */
/* static void ConfigureHPGLPenSizes( HPGL_PLOTTER *aPlotter, const PCB_PLOT_PARAMS *aPlotOpts )
{
    // Compute penDiam (the value is given in mils) in pcb units, with plot scale (if Scale is 2,
    // penDiam value is always m_HPGLPenDiam so apparent penDiam is actually penDiam / Scale
    int penDiam = KiROUND( aPlotOpts->GetHPGLPenDiameter() * IU_PER_MILS / aPlotOpts->GetScale() );

    // Set HPGL-specific options and start
    aPlotter->SetPenSpeed( aPlotOpts->GetHPGLPenSpeed() );
    aPlotter->SetPenNumber( aPlotOpts->GetHPGLPenNum() );
    aPlotter->SetPenDiameter( penDiam );
}  */

/**
 * Open a new plotfile using the options (and especially the format) specified in the options
 * and prepare the page for plotting.
 *
 * @return the plotter object if OK, NULL if the file is not created (or has a problem).
 */
pub fn StartPlotBoard(
    aBoard: &Pcb,
    aPlotOpts: &PCB_PLOT_PARAMS,
    aLayer: LayerId,
    aFullFileName: String,
    aSheetDesc: String,
) -> Result<Rc<RefCell<GERBER_PLOTTER>>, Error> {
    // Create the plotter driver and set the few plotter specific options
    // PLOTTER*    plotter = nullptr;

    let mut writer = Box::new(std::fs::File::create(aFullFileName)?);
    let mut plotter = match aPlotOpts.GetFormat() {
        /* case PLOT_FORMAT::DXF:
            DXF_PLOTTER* DXF_plotter;
            DXF_plotter = new DXF_PLOTTER();
            DXF_plotter->SetUnits( aPlotOpts->GetDXFPlotUnits() );

            plotter = DXF_plotter;
            break;

        case PLOT_FORMAT::POST:
            PS_PLOTTER* PS_plotter;
            PS_plotter = new PS_PLOTTER();
            PS_plotter->SetScaleAdjust( aPlotOpts->GetFineScaleAdjustX(),
                                        aPlotOpts->GetFineScaleAdjustY() );
            plotter = PS_plotter;
            break; */
        /* PlotFormat::PDF => {
            // plotter = new PDF_PLOTTER();
        } */

        /* case PLOT_FORMAT::HPGL:
        HPGL_PLOTTER* HPGL_plotter;
        HPGL_plotter = new HPGL_PLOTTER();

        // HPGL options are a little more convoluted to compute, so they get their own function
        ConfigureHPGLPenSizes( HPGL_plotter, aPlotOpts );
        plotter = HPGL_plotter;
        break; */
        PlotFormat::GERBER => {
            // plotter = new GERBER_PLOTTER();
            Rc::new(RefCell::new(GERBER_PLOTTER::new(writer)))
        }

        /* PlotFormat::SVG => {
            plotter = new SVG_PLOTTER();
        } */

        _ => {
            todo!("unsupported plotter type");
        }
    };

    let mut renderSettings = PCB_RENDER_SETTINGS::new();
    // renderSettings.LoadColors( aPlotOpts.ColorSettings() );
    // renderSettings.SetDefaultPenWidth( Millimeter2iu( 0.0212 ) );  // Hairline at 1200dpi

    /* if( aLayer < GAL_LAYER_ID_END )
    renderSettings->SetLayerName( aBoard->GetLayerName( ToLAYER_ID( aLayer ) ) ); */

    plotter.borrow_mut().SetRenderSettings(renderSettings);

    // Compute the viewport and set the other options

    // page layout is not mirrored, so temporarily change mirror option for the page layout
    let plotOpts: PCB_PLOT_PARAMS = aPlotOpts.clone();

    /* if( plotOpts.GetPlotFrameRef() && plotOpts.GetMirror() )
    plotOpts.SetMirror( false ); */

    initializePlotter(plotter.clone(), aBoard, &plotOpts);

    // if plotter.borrow_mut().OpenFile(aFullFileName).is_ok() {
        plotter.borrow_mut().ClearHeaderLinesList();

        // For the Gerber "file function" attribute, set the layer number
        /* if plotter->GetPlotterType() == PLOT_FORMAT::GERBER )
        { */
        let useX2mode = plotOpts.GetUseGerberX2format();

        //TODO casted here: let mut gbrplotter: GERBER_PLOTTER = GERBER_PLOTTER{};
        plotter.borrow_mut().DisableApertMacros(plotOpts.GetDisableGerberMacros());
        plotter.borrow_mut().UseX2format(useX2mode);
        plotter.borrow_mut().UseX2NetAttributes(plotOpts.GetIncludeGerberNetlistInfo());

        // Attributes can be added using X2 format or as comment (X1 format)
        AddGerberX2Attribute(plotter.clone(), aBoard, aLayer, !useX2mode );
        // }

        plotter.borrow_mut().StartPlot()?;

        // Plot the frame reference if requested
        /* if( aPlotOpts->GetPlotFrameRef() )
        {
            PlotDrawingSheet( plotter, aBoard->GetProject(), aBoard->GetTitleBlock(),
                              aBoard->GetPageSettings(), wxT( "1" ), 1, aSheetDesc,
                              aBoard->GetFileName() );

            if( aPlotOpts->GetMirror() )
                initializePlotter( plotter, aBoard, aPlotOpts );
        } */

        // When plotting a negative board: draw a black rectangle (background for plot board
        // in white) and switch the current color to WHITE; note the color inversion is actually
        // done in the driver (if supported)
        /* if( aPlotOpts->GetNegative() )
        {
            EDA_RECT bbox = aBoard->ComputeBoundingBox();
            FillNegativeKnockout( plotter, bbox );
        } */

        return Ok(plotter);
    // }

    /* delete plotter->RenderSettings();
    delete plotter;
    return nullptr; */
}
