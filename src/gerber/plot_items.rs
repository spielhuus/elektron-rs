// Define min and max reasonable values for plot/print scale
/* #define PLOT_MIN_SCALE 0.01
#define PLOT_MAX_SCALE 100.0 */

use std::{rc::Rc, cell::RefCell};

use crate::sexp::{LayerId, Pcb, Text, Footprint, Shape, LayerSet, Pad, Zone, shape_poly_set::SHAPE_POLY_SET};

use super::{plotter::Plotter, plot_params::{PCB_PLOT_PARAMS, OUTLINE_MODE}, color4d::COLOR4D, gerber_plotter::GERBER_PLOTTER, gerber_metadata::GBR_METADATA};

// A helper class to plot board items
pub struct BRDITEMS_PLOTTER<'a> {
    aBoard: &'a Pcb, 
    aPlotter: Rc<RefCell<GERBER_PLOTTER>>, 
    aPlotOpt: &'a PCB_PLOT_PARAMS,
    m_layerMask: LayerSet,

 }

impl<'a> BRDITEMS_PLOTTER<'a> {
    pub fn new(aBoard: &'a Pcb, aPlotter: Rc<RefCell<GERBER_PLOTTER>>, aPlotOpt: &'a PCB_PLOT_PARAMS) -> Self {
        Self {
            aBoard,
            aPlotter,
            aPlotOpt,
            m_layerMask: LayerSet::new(),
        }
    }
    pub fn SetLayerSet(&mut self, layers: LayerSet) { //OLD: LSET aLayerMask ) { 
        self.m_layerMask = layers; 
    }

    /**
     * Plot items like text and graphics but not tracks and footprints.
     */
    pub fn PlotBoardGraphicItems(&self) {
        /* for item in m_board.drawings() {
            match item {
                Text => {
                    //TODO: PlotPcbText( (PCB_TEXT*) item );
                }
                /* TODO: PCB_SHAPE_T => {
                    PlotPcbShape( (PCB_SHAPE*) item );
                },
                PCB_TEXT_T => {
                },
                PCB_DIM_ALIGNED_T|PCB_DIM_CENTER_T|PCB_DIM_ORTHOGONAL_T|PCB_DIM_LEADER_T => {
                    PlotDimension( (PCB_DIMENSION_BASE*) item );
                },
                PCB_TARGET_T => {
                    PlotPcbTarget( (PCB_TARGET*) item );
                }, */
            }
        } */
    }
    
    // Basic functions to plot a board item
    pub fn PlotFootprintGraphicItems(&self, aFootprint: &Footprint ) {

    }
    pub fn PlotFootprintGraphicItem( aShape: &Shape ) {

    }
    /*
     * Reference, Value, and other fields are plotted only if the corresponding option is enabled.
     * Invisible text fields are plotted only if PlotInvisibleText option is set.
     */
    pub fn PlotFootprintTextItems(&self, aFootprint: &Footprint) {

    }
    /* pub fn PlotDimension(&self, aDim: PCB_DIMENSION_BASE) {

    }
    pub fn PlotPcbTarget( const PCB_TARGET* aMire ); */
    pub fn PlotFilledAreas(&mut self, aZone: &Zone, polysList: &SHAPE_POLY_SET ) {
        /* if polysList.IsEmpty() {
            return;
        }

        let gbr_metadata = GBR_METADATA::new();

        bool isOnCopperLayer = aZone->IsOnCopperLayer();

        if( isOnCopperLayer ) {
            gbr_metadata.SetNetName( aZone->GetNetname() );
            gbr_metadata.SetCopper( true );

            // Zones with no net name can exist.
            // they are not used to connect items, so the aperture attribute cannot
            // be set as conductor
            if( aZone->GetNetname().IsEmpty() )
            {
                gbr_metadata.SetApertureAttrib(
                        GBR_APERTURE_METADATA::GBR_APERTURE_ATTRIB_NONCONDUCTOR );
            }
            else
            {
                gbr_metadata.SetApertureAttrib( GBR_APERTURE_METADATA::GBR_APERTURE_ATTRIB_CONDUCTOR );
                gbr_metadata.SetNetAttribType( GBR_NETLIST_METADATA::GBR_NETINFO_NET );
            }
        }

        m_plotter->SetColor( getColor( aZone->GetLayer() ) );

        m_plotter->StartBlock( nullptr );    // Clean current object attributes

        /* Plot all filled areas: filled areas have a filled area and a thick
         * outline (depending on the fill area option we must plot the filled area itself
         * and plot the thick outline itself, if the thickness has meaning (at least is > 1)
         *
         * in non filled mode the outline is plotted, but not the filling items
         */
        int outline_thickness = aZone->GetFilledPolysUseThickness() ? aZone->GetMinThickness() : 0;

        for( int idx = 0; idx < polysList.OutlineCount(); ++idx )
        {
            const SHAPE_LINE_CHAIN& outline = polysList.Outline( idx );

            // Plot the current filled area (as region for Gerber plotter
            // to manage attributes) and its outline for thick outline
            if( GetPlotMode() == FILLED )
            {
                if( m_plotter->GetPlotterType() == PLOT_FORMAT::GERBER )
                {
                    if( outline_thickness > 0 )
                    {
                        m_plotter->PlotPoly( outline, FILL_T::NO_FILL, outline_thickness,
                                             &gbr_metadata );

                        // Ensure the outline is closed:
                        int last_idx = outline.PointCount() - 1;

                        if( outline.CPoint( 0 ) != outline.CPoint( last_idx ) )
                        {
                            m_plotter->ThickSegment( wxPoint( outline.CPoint( 0 ) ),
                                                     wxPoint( outline.CPoint( last_idx ) ),
                                                     outline_thickness, GetPlotMode(), &gbr_metadata );
                        }
                    }

                    static_cast<GERBER_PLOTTER*>( m_plotter )->PlotGerberRegion( outline,
                                                                                 &gbr_metadata );
                }
                else
                {
                    m_plotter->PlotPoly( outline, FILL_T::FILLED_SHAPE, outline_thickness,
                                         &gbr_metadata );
                }
            }
            else
            {
                if( outline_thickness )
                {
                    int last_idx = outline.PointCount() - 1;

                    for( int jj = 1; jj <= last_idx; jj++ )
                    {
                        m_plotter->ThickSegment( wxPoint( outline.CPoint( jj - 1) ),
                                                 wxPoint( outline.CPoint( jj ) ),
                                                 outline_thickness,
                                                 GetPlotMode(), &gbr_metadata );
                    }

                    // Ensure the outline is closed:
                    if( outline.CPoint( 0 ) != outline.CPoint( last_idx ) )
                    {
                        m_plotter->ThickSegment( wxPoint( outline.CPoint( 0 ) ),
                                                 wxPoint( outline.CPoint( last_idx ) ),
                                                 outline_thickness,
                                                 GetPlotMode(), &gbr_metadata );
                    }
                }

                m_plotter->SetCurrentLineWidth( -1 );
            }
        }

        m_plotter->EndBlock( nullptr );    // Clear object attributes */
    }

    /*
    pub fn PlotPcbText( const PCB_TEXT* aText );
    pub fn PlotPcbShape( const PCB_SHAPE* aShape ); */

    /*
     * Plot a pad.
     *
     * Unlike other items, a pad had not a specific color and be drawn as a non filled item
     * although the plot mode is filled color and plot mode are needed by this function.
     */
     pub fn PlotPad(&mut self, aPad: &Pad, aColor: &COLOR4D, aPlotMode: OUTLINE_MODE) {
    }


    /*
     * Draw a drill mark for pads and vias.
     *
     * Must be called after all drawings, because it redraws the drill mark on a pad or via, as
     * a negative (i.e. white) shape in FILLED plot mode (for PS and PDF outputs).
     */
    // void PlotDrillMarks();
}


/* {
public:
    BRDITEMS_PLOTTER( PLOTTER* aPlotter, BOARD* aBoard, const PCB_PLOT_PARAMS& aPlotOpts )
            : PCB_PLOT_PARAMS( aPlotOpts )
    {
        m_plotter = aPlotter;
        m_board = aBoard;
    }

    /**
     * @return a 'width adjustment' for the postscript engine
     * (useful for controlling toner bleeding during direct transfer)
     * added to track width and via/pads size
     */
    int getFineWidthAdj() const
    {
        if( GetFormat() == PLOT_FORMAT::POST )
            return GetWidthAdjust();
        else
            return 0;
    }
    


    /**
     * White color is special because it cannot be seen on a white paper in B&W mode. It is
     * plotted as white but other colors are plotted in BLACK so the returned color is LIGHTGRAY
     * when the layer color is WHITE.
     *
     * @param aLayer is the layer id.
     * @return the layer color.
     */
    COLOR4D getColor( LAYER_NUM aLayer ) const;

private:
    /**
     * Helper function to plot a single drill mark.
     *
     * It compensate and clamp the drill mark size depending on the current plot options.
     */
    void plotOneDrillMark( PAD_DRILL_SHAPE_T aDrillShape, const wxPoint& aDrillPos,
                           const wxSize& aDrillSize, const wxSize& aPadSize,
                           double aOrientation, int aSmallDrill );

    PLOTTER*    m_plotter;
    BOARD*      m_board;
    LSET        m_layerMask;
};

PLOTTER* StartPlotBoard( BOARD* aBoard,
                         const PCB_PLOT_PARAMS* aPlotOpts,
                         int aLayer,
                         const wxString& aFullFileName,
                         const wxString& aSheetDesc );

/**
 * Plot one copper or technical layer.
 *
 * It prepares options and calls the specialized plot function according to the layer type.
 *
 * @param aBoard is the board to plot.
 * @param aPlotter is the plotter to use.
 * @param aLayer is the layer id to plot.
 * @param aPlotOpt is the plot options (files, sketch). Has meaning for some formats only.
 */
void PlotOneBoardLayer( BOARD* aBoard, PLOTTER* aPlotter, PCB_LAYER_ID aLayer,
                        const PCB_PLOT_PARAMS& aPlotOpt );

/**
 * Plot copper or technical layers.
 *
 * This is not used for silk screen layers because these layers have specific requirements.
 * This is  mainly for pads.
 *
 * @param aBoard is the board to plot.
 * @param aPlotter is the plotter to use.
 * @param aLayerMask is the mask to define the layers to plot.
 * @param aPlotOpt is the plot options (files, sketch). Has meaning for some formats only.
 *
 * aPlotOpt has 3 important options to control this plot,
 * which are set, depending on the layer type to plot
 *      SetEnablePlotVia( bool aEnable )
 *          aEnable = true to plot vias, false to skip vias (has meaning
 *                      only for solder mask layers).
 *      SetSkipPlotNPTH_Pads( bool aSkip )
 *          aSkip = true to skip NPTH Pads, when the pad size and the pad hole
 *                  have the same size. Used in GERBER format only.
 *      SetDrillMarksType( DrillMarksType aVal ) control the actual hole:
 *              no hole, small hole, actual hole
 */
void PlotStandardLayer( BOARD* aBoard, PLOTTER* aPlotter, LSET aLayerMask,
                        const PCB_PLOT_PARAMS& aPlotOpt );

/**
 * Plot copper outline of a copper layer.
 *
 * @param aBoard is the board to plot.
 * @param aPlotter is the plotter to use.
 * @param aLayerMask is the mask to define the layers to plot.
 * @param aPlotOpt is the plot options. Has meaning for some formats only.
 */
void PlotLayerOutlines( BOARD* aBoard, PLOTTER* aPlotter,
                        LSET aLayerMask, const PCB_PLOT_PARAMS& aPlotOpt );

/**
 * Complete a plot filename.
 *
 * It forces the output directory, adds a suffix to the name, and sets the specified extension.
 * The suffix is usually the layer name and replaces illegal file name character in the suffix
 * with an underscore character.
 *
 * @param aFilename is the file name to initialize that contains the base filename.
 * @param aOutputDir is the path.
 * @param aSuffix is the suffix to add to the base filename.
 * @param aExtension is the file extension.
 */
void BuildPlotFileName( wxFileName*     aFilename,
                        const wxString& aOutputDir,
                        const wxString& aSuffix,
                        const wxString& aExtension );


/**
 * @return the appropriate Gerber file extension for \a aLayer
 */
const wxString GetGerberProtelExtension( LAYER_NUM aLayer );

/**
 * Return the "file function" attribute for \a aLayer, as defined in the
 * Gerber file format specification J1 (chapter 5).
 *
 * The returned string includes the "%TF.FileFunction" attribute prefix and the "*%" suffix.
 *
 * @param aBoard is the board, needed to get the total count of copper layers.
 * @param aLayer is the layer number to create the attribute for.
 * @return The attribute, as a text string
 */
const wxString GetGerberFileFunctionAttribute( const BOARD* aBoard, LAYER_NUM aLayer );

/**
 * Calculate some X2 attributes as defined in the Gerber file format specification J4
 * (chapter 5) and add them the to the gerber file header.
 *
 * TF.GenerationSoftware
 * TF.CreationDate
 * TF.ProjectId
 * file format attribute is not added
 *
 * @param aPlotter is the current plotter.
 * @param aBoard is the board, needed to extract some info.
 * @param aUseX1CompatibilityMode set to false to generate X2 attributes, true to
 *        use X1 compatibility (X2 attributes added as structured comments,
 *        starting by "G04 #@! " followed by the X2 attribute
 */
void AddGerberX2Header( PLOTTER* aPlotter, const BOARD* aBoard,
                        bool aUseX1CompatibilityMode = false );

/**
 * Calculate some X2 attributes as defined in the Gerber file format specification and add them
 * to the gerber file header.
 *
 * TF.GenerationSoftware
 * TF.CreationDate
 * TF.ProjectId
 * TF.FileFunction
 * TF.FilePolarity
 *
 * @param aPlotter is the current plotter.
 * @param aBoard is the board, needed to extract some info.
 * @param aLayer is the layer number to create the attribute for.
 * @param aUseX1CompatibilityMode set to false to generate X2 attributes, true to use X1
 *        compatibility (X2 attributes added as structured comments, starting by "G04 #@! "
 *        followed by the X2 attribute.
 */
void AddGerberX2Attribute( PLOTTER* aPlotter, const BOARD* aBoard,
                           LAYER_NUM aLayer, bool aUseX1CompatibilityMode );

#endif // PCBPLOT_H_ */
