use super::PlotFormat;


#[derive(Debug, Clone)]
pub struct COLOR_SETTINGS;
struct PCB_PLOT_PARAMS_PARSER;

#[derive(Clone, Copy, Debug)]
pub enum DrillMarksType {
    NO_DRILL_SHAPE,
    SMALL_DRILL_SHAPE,
    FULL_DRILL_SHAPE,
}

#[derive(Clone, Copy, Debug)]
pub enum OUTLINE_MODE {
    Sketch,     // sketch mode: draw segments outlines only
    Filled,     // normal mode: solid segments
}


/**
 * Parameters and options when plotting/printing a board.
 */
#[derive(Debug, Clone)]
pub struct PCB_PLOT_PARAMS {
    m_gerberDisableApertMacros: bool,
    m_includeGerberNetlistInfo: bool,
    m_useGerberX2format: bool,
    m_format: PlotFormat,
    m_colors: COLOR_SETTINGS,
    m_excludeEdgeLayer: bool,
    m_drillMarks: DrillMarksType,
    m_skipNPTH_Pads: bool,
    m_plotMode: OUTLINE_MODE,
    m_subtractMaskFromSilk: bool,
}

impl PCB_PLOT_PARAMS {
    pub fn new() -> Self {
        Self {
            m_gerberDisableApertMacros: false,
            m_includeGerberNetlistInfo: false,
            m_useGerberX2format: true,
            m_format: PlotFormat::SVG,
            m_colors: COLOR_SETTINGS{},
            m_excludeEdgeLayer: false,
            m_drillMarks: DrillMarksType::NO_DRILL_SHAPE,
            m_skipNPTH_Pads: false,
            m_plotMode: OUTLINE_MODE::Filled,
            m_subtractMaskFromSilk: false,
        }
    }

    pub fn SetDisableGerberMacros(&mut self,  aDisable: bool ) { self.m_gerberDisableApertMacros = aDisable; }
    pub fn GetDisableGerberMacros(&self) -> bool { self.m_gerberDisableApertMacros }
    pub fn SetIncludeGerberNetlistInfo(&mut self, aUse: bool ) { self.m_includeGerberNetlistInfo = aUse; }
    pub fn GetIncludeGerberNetlistInfo(&self) -> bool { self.m_includeGerberNetlistInfo }
    pub fn SetUseGerberX2format(&mut self, aUse: bool ) { self.m_useGerberX2format = aUse; }
    pub fn GetUseGerberX2format(&self) -> bool { self.m_useGerberX2format }
    pub fn SetFormat(&mut self, aFormat: PlotFormat ) { self.m_format = aFormat }
    pub fn GetFormat(&self) -> PlotFormat { self.m_format }

    pub fn SetColorSettings(&mut self, aSettings: COLOR_SETTINGS ) { self.m_colors = aSettings; }
    pub fn ColorSettings(&self) -> COLOR_SETTINGS { self.m_colors.clone() } //TODO:

    pub fn SetExcludeEdgeLayer(&mut self, aFlag: bool) { 
        self.m_excludeEdgeLayer = aFlag; 
    }
    pub fn GetExcludeEdgeLayer(&self) -> bool { self.m_excludeEdgeLayer }
    pub fn SetDrillMarksType(&mut self, aVal: DrillMarksType) { self.m_drillMarks = aVal; }
    pub fn GetDrillMarksType(&self) -> DrillMarksType { self.m_drillMarks }
    pub fn SetSkipPlotNPTH_Pads(&mut self, aSkip: bool ) { 
        self.m_skipNPTH_Pads = aSkip; 
    }
    pub fn GetSkipPlotNPTH_Pads(&self) -> bool { self.m_skipNPTH_Pads }

    pub fn SetPlotMode(&mut self, aPlotMode: OUTLINE_MODE ) { self.m_plotMode = aPlotMode; }
    pub fn GetPlotMode(&self) -> OUTLINE_MODE { return self.m_plotMode; }
    pub fn SetSubtractMaskFromSilk(&mut self, aSubtract: bool ) { self.m_subtractMaskFromSilk = aSubtract }
    pub fn GetSubtractMaskFromSilk(&self) -> bool { self.m_subtractMaskFromSilk }
}


/* {
public:
    enum DrillMarksType {
        NO_DRILL_SHAPE    = 0,
        SMALL_DRILL_SHAPE = 1,
        FULL_DRILL_SHAPE  = 2
    };

    PCB_PLOT_PARAMS();


    void        Format( OUTPUTFORMATTER* aFormatter, int aNestLevel, int aControl=0 ) const;
    void        Parse( PCB_PLOT_PARAMS_PARSER* aParser );

    /**
     * Compare current settings to aPcbPlotParams, including not saved parameters in brd file.
     *
     * @param aPcbPlotParams is the #PCB_PLOT_PARAMS to compare/
     * @param aCompareOnlySavedPrms set to true to compare only saved in file parameters,
     *        or false to compare the full set of parameters.
     * @return true is parameters are same, false if one (or more) parameter does not match.
     */
    bool        IsSameAs( const PCB_PLOT_PARAMS &aPcbPlotParams ) const;

    void SetColorSettings( COLOR_SETTINGS* aSettings ) { m_colors = aSettings; }

    COLOR_SETTINGS* ColorSettings() const { return m_colors; }

    void SetTextMode( PLOT_TEXT_MODE aVal )
    {
        m_textMode = aVal;
    }

    PLOT_TEXT_MODE GetTextMode() const
    {
        return m_textMode;
    }


    void        SetDXFPlotPolygonMode( bool aFlag ) { m_DXFplotPolygonMode = aFlag; }
    bool        GetDXFPlotPolygonMode() const { return m_DXFplotPolygonMode; }

    void SetDXFPlotUnits( DXF_UNITS aUnit )
    {
        m_DXFplotUnits = aUnit;
    }

    DXF_UNITS GetDXFPlotUnits() const
    {
        return m_DXFplotUnits;
    }


    void        SetScale( double aVal ) { m_scale = aVal; }
    double      GetScale() const { return m_scale; }

    void        SetFineScaleAdjustX( double aVal ) { m_fineScaleAdjustX = aVal; }
    double      GetFineScaleAdjustX() const { return m_fineScaleAdjustX; }
    void        SetFineScaleAdjustY( double aVal ) { m_fineScaleAdjustY = aVal; }
    double      GetFineScaleAdjustY() const { return m_fineScaleAdjustY; }
    void        SetWidthAdjust( int aVal ) { m_widthAdjust = aVal; }
    int         GetWidthAdjust() const { return m_widthAdjust; }

    void        SetAutoScale( bool aFlag ) { m_autoScale = aFlag; }
    bool        GetAutoScale() const { return m_autoScale; }

    void        SetMirror( bool aFlag ) { m_mirror = aFlag; }
    bool        GetMirror() const { return m_mirror; }

    void        SetSketchPadsOnFabLayers( bool aFlag ) { m_sketchPadsOnFabLayers = aFlag; }
    bool        GetSketchPadsOnFabLayers() const { return m_sketchPadsOnFabLayers; }
    void        SetSketchPadLineWidth( int aWidth ) { m_sketchPadLineWidth = aWidth; }
    int         GetSketchPadLineWidth() const { return m_sketchPadLineWidth; }

    void        SetPlotInvisibleText( bool aFlag ) { m_plotInvisibleText = aFlag; }
    bool        GetPlotInvisibleText() const { return m_plotInvisibleText; }
    void        SetPlotValue( bool aFlag ) { m_plotValue = aFlag; }
    bool        GetPlotValue() const { return m_plotValue; }
    void        SetPlotReference( bool aFlag ) { m_plotReference = aFlag; }
    bool        GetPlotReference() const { return m_plotReference; }

    void        SetNegative( bool aFlag ) { m_negative = aFlag; }
    bool        GetNegative() const { return m_negative; }

    void        SetPlotViaOnMaskLayer( bool aFlag ) { m_plotViaOnMaskLayer = aFlag; }
    bool        GetPlotViaOnMaskLayer() const { return m_plotViaOnMaskLayer; }

    void        SetPlotFrameRef( bool aFlag ) { m_plotFrameRef = aFlag; }
    bool        GetPlotFrameRef() const { return m_plotFrameRef; }



    void        SetOutputDirectory( const wxString& aDir ) { m_outputDirectory = aDir; }
    wxString    GetOutputDirectory() const { return m_outputDirectory; }

    void        SetDisableGerberMacros( bool aDisable ) { m_gerberDisableApertMacros = aDisable; }
    bool        GetDisableGerberMacros() const { return m_gerberDisableApertMacros; }

    void        SetUseGerberX2format( bool aUse ) { m_useGerberX2format = aUse; }
    bool        GetUseGerberX2format() const { return m_useGerberX2format; }


    void        SetCreateGerberJobFile( bool aCreate ) { m_createGerberJobFile = aCreate; }
    bool        GetCreateGerberJobFile() const { return m_createGerberJobFile; }

    void        SetUseGerberProtelExtensions( bool aUse ) { m_useGerberProtelExtensions = aUse; }
    bool        GetUseGerberProtelExtensions() const { return m_useGerberProtelExtensions; }

    void        SetGerberPrecision( int aPrecision );
    int         GetGerberPrecision() const { return m_gerberPrecision; }

    void        SetSvgPrecision( unsigned aPrecision, bool aUseInch );
    unsigned    GetSvgPrecision() const { return m_svgPrecision; }
    bool        GetSvgUseInch() const { return m_svgUseInch; }

    /**
     * Default precision of coordinates in Gerber files.
     *
     * When units are in mm (7 in inches, but Pcbnew uses mm).
     * 6 is the internal resolution of Pcbnew, so the default is 6.
     */
    static int  GetGerberDefaultPrecision() { return 6; }


    void        SetLayerSelection( LSET aSelection )    { m_layerSelection = aSelection; };
    LSET        GetLayerSelection() const               { return m_layerSelection; };

    void        SetUseAuxOrigin( bool aAux ) { m_useAuxOrigin = aAux; };
    bool        GetUseAuxOrigin() const { return m_useAuxOrigin; };

    void        SetScaleSelection( int aSelection ) { m_scaleSelection = aSelection; };
    int         GetScaleSelection() const { return m_scaleSelection; };

    void        SetA4Output( int aForce ) { m_A4Output = aForce; };
    bool        GetA4Output() const { return m_A4Output; };

    // For historical reasons, this parameter is stored in mils
    // (but is in mm in hpgl files...)
    double      GetHPGLPenDiameter() const { return m_HPGLPenDiam; };
    bool        SetHPGLPenDiameter( double aValue );

    // This parameter is always in cm, due to hpgl file format constraint
    int         GetHPGLPenSpeed() const { return m_HPGLPenSpeed; };
    bool        SetHPGLPenSpeed( int aValue );

    void        SetHPGLPenNum( int aVal ) { m_HPGLPenNum = aVal; }
    int         GetHPGLPenNum() const { return m_HPGLPenNum; }

private:
    friend class PCB_PLOT_PARAMS_PARSER;

    // If true, do not plot NPTH pads
    // (mainly used to disable NPTH pads plotting on copper layers)

    /**
     * FILLED or SKETCH selects how to plot filled objects.
     *
     * FILLED or SKETCH not available with all drivers: some have fixed mode
     */
    OUTLINE_MODE m_plotMode;

    /**
     * DXF format: Plot items in outline (polygon) mode.
     *
     * In polygon mode, each item to plot is converted to a polygon and all polygons are merged.
     */
    bool        m_DXFplotPolygonMode;

    /**
     * DXF format: Units to use when plotting the DXF
     */
    DXF_UNITS m_DXFplotUnits;

    /// Plot format type (chooses the driver to be used)
    PLOT_FORMAT m_format;

    /// Holes can be not plotted, have a small mark or plotted in actual size
    DrillMarksType m_drillMarks;

    /// Choose how represent text with PS, PDF and DXF drivers
    PLOT_TEXT_MODE m_textMode;

    /// When true set the scale to fit the board in the page
    bool        m_autoScale;

    /// Global scale factor, 1.0 plots a board with its actual size.
    double      m_scale;

    /// Mirror the plot around the X axis
    bool        m_mirror;

    /// Plot in negative color (supported only by some drivers)
    bool        m_negative;

    /// True if vias are drawn on Mask layer (ie untented, *exposed* by mask)
    bool        m_plotViaOnMaskLayer;

    /// True to plot/print frame references
    bool        m_plotFrameRef;

    /// If false always plot (merge) the pcb edge layer on other layers
    bool        m_excludeEdgeLayer;

    /// Set of layers to plot
    LSET        m_layerSelection;

    /** When plotting gerber files, use a conventional set of Protel extensions
     * instead of .gbr, that is now the official gerber file extension
     * this is a deprecated feature
     */
    bool        m_useGerberProtelExtensions;

    /// Include attributes from the Gerber X2 format (chapter 5 in revision J2)
    bool        m_useGerberX2format;

    /// Disable aperture macros in Gerber format (only for broken Gerber readers)
    /// Ideally, should be never selected.
    bool        m_gerberDisableApertMacros;

    /// Include netlist info (only in Gerber X2 format) (chapter ? in revision ?)
    bool        m_includeGerberNetlistInfo;

    /// generate the auxiliary "job file" in gerber format
    bool        m_createGerberJobFile;

    /// precision of coordinates in Gerber files: accepted 5 or 6
    /// when units are in mm (6 or 7 in inches, but Pcbnew uses mm).
    /// 6 is the internal resolution of Pcbnew, but not always accepted by board maker
    /// 5 is the minimal value for professional boards.
    int         m_gerberPrecision;

    /// precision of coordinates in SVG files: accepted 3 - 6
    /// 6 is the internal resolution of Pcbnew
    unsigned    m_svgPrecision;

    /// units for SVG plot
    /// false for metric, true for inch/mils
    bool        m_svgUseInch;

    /// Plot gerbers using auxiliary (drill) origin instead of absolute coordinates
    bool        m_useAuxOrigin;

    /// On gerbers 'scrape' away the solder mask from silkscreen (trim silks)
    bool        m_subtractMaskFromSilk;

    /// Autoscale the plot to fit an A4 (landscape?) sheet
    bool        m_A4Output;

    /// Scale ratio index (UI only)
    int         m_scaleSelection;

    /// Output directory for plot files (usually relative to the board file)
    wxString    m_outputDirectory;

    /// Enable plotting of part references
    bool        m_plotReference;

    /// Enable plotting of part values
    bool        m_plotValue;

    /// Force plotting of fields marked invisible
    bool        m_plotInvisibleText;

    /// Plots pads outlines on fab layers
    bool        m_sketchPadsOnFabLayers;
    int         m_sketchPadLineWidth;

    /* These next two scale factors are intended to compensate plotters
     * (mainly printers) X and Y scale error. Therefore they are expected very
     * near 1.0; only X and Y dimensions are adjusted: circles are plotted as
     * circles, even if X and Y fine scale differ; because of this it is mostly
     * useful for printers: postscript plots would be best adjusted using
     * the prologue (that would change the whole output matrix
     */

    double      m_fineScaleAdjustX;     ///< fine scale adjust X axis
    double      m_fineScaleAdjustY;     ///< fine scale adjust Y axis

    /**
     * This width factor is intended to compensate PS printers/ plotters that do
     * not strictly obey line width settings. Only used to plot pads and tracks.
     */
    int         m_widthAdjust;

    int         m_HPGLPenNum;           ///< HPGL only: pen number selection(1 to 9)
    int         m_HPGLPenSpeed;         ///< HPGL only: pen speed, always in cm/s (1 to 99 cm/s)
    double      m_HPGLPenDiam;          ///< HPGL only: pen diameter in MILS, useful to fill areas
                                        ///< However, it is in mm in hpgl files.

    /// Pointer to active color settings to be used for plotting
    COLOR_SETTINGS* m_colors;

    /// Dummy colors object that can be created if there is no Pgm context
    std::shared_ptr<COLOR_SETTINGS> m_default_colors;
};


#endif // PCB_PLOT_PARAMS_H_ */
