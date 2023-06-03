use super::plot_params::COLOR_SETTINGS;

#[derive(Debug, Clone)]
pub struct PCB_RENDER_SETTINGS {
    m_defaultPenWidth: f64,
}

impl PCB_RENDER_SETTINGS {
    pub fn new() -> Self {
        Self{
            m_defaultPenWidth: 0.0,
        }
    }


    pub fn LoadColors(&mut self, aSettings: COLOR_SETTINGS) {

    }
    pub fn GetDefaultPenWidth(&self) -> f64 { self.m_defaultPenWidth }
    pub fn SetDefaultPenWidth(&mut self, aWidth: f64 ) { self.m_defaultPenWidth = aWidth; }
}

/* {
public:
    friend class PCB_PAINTER;

    ///< Flags to control clearance lines visibility
    enum CLEARANCE_MODE
    {
        CL_NONE             = 0x00,

        // Object type
        CL_PADS             = 0x01,
        CL_VIAS             = 0x02,
        CL_TRACKS           = 0x04,

        // Existence
        CL_NEW              = 0x08,
        CL_EDITED           = 0x10,
        CL_EXISTING         = 0x20
    };

    PCB_RENDER_SETTINGS();

    /**
     * Load settings related to display options (high-contrast mode, full or outline modes
     * for vias/pads/tracks and so on).
     *
     * @param aOptions are settings that you want to use for displaying items.
     */
    void LoadDisplayOptions( const PCB_DISPLAY_OPTIONS& aOptions, bool aShowPageLimits );

    virtual void LoadColors( const COLOR_SETTINGS* aSettings ) override;

    /// @copydoc RENDER_SETTINGS::GetColor()
    virtual COLOR4D GetColor( const VIEW_ITEM* aItem, int aLayer ) const override;

    /**
     * Turn on/off sketch mode for given item layer.
     *
     * @param aItemLayer is the item layer that is changed.
     * @param aEnabled decides if it is drawn in sketch mode (true for sketched mode,
     *                 false for filled mode).
     */
    inline void SetSketchMode( int aItemLayer, bool aEnabled )
    {
        m_sketchMode[aItemLayer] = aEnabled;
    }

    /**
     * Return sketch mode setting for a given item layer.
     *
     * @param aItemLayer is the item layer that is changed.
     */
    inline bool GetSketchMode( int aItemLayer ) const
    {
        return m_sketchMode[aItemLayer];
    }

    /**
     * Turn on/off sketch mode for graphic items (DRAWSEGMENTs, texts).
     *
     * @param aEnabled decides if it is drawn in sketch mode (true for sketched mode,
     *                 false for filled mode).
     */
    inline void SetSketchModeGraphicItems( bool aEnabled )
    {
        m_sketchGraphics = aEnabled;
    }

    /**
     * Turn on/off drawing outline and hatched lines for zones.
     */
    void EnableZoneOutlines( bool aEnabled )
    {
        m_zoneOutlines = aEnabled;
    }

    inline bool IsBackgroundDark() const override
    {
        auto luma = m_layerColors[ LAYER_PCB_BACKGROUND ].GetBrightness();

        return luma < 0.5;
    }

    const COLOR4D& GetBackgroundColor() override { return m_layerColors[ LAYER_PCB_BACKGROUND ]; }

    void SetBackgroundColor( const COLOR4D& aColor ) override
    {
        m_layerColors[ LAYER_PCB_BACKGROUND ] = aColor;
    }

    const COLOR4D& GetGridColor() override { return m_layerColors[ LAYER_GRID ]; }

    const COLOR4D& GetCursorColor() override { return m_layerColors[ LAYER_CURSOR ]; }

    /**
     * Switch the contrast mode setting (HIGH_CONTRAST_MODE:NORMAL, DIMMED or HIDDEN )
     * to control how the non active layers are shown
     */
    void SetContrastModeDisplay( HIGH_CONTRAST_MODE aMode ) { m_contrastModeDisplay = aMode; }

    /**
     * @return the contrast mode setting (HIGH_CONTRAST_MODE:NORMAL, DIMMED or HIDDEN ).
     */
    HIGH_CONTRAST_MODE GetContrastModeDisplay() { return m_contrastModeDisplay; }

    inline bool GetCurvedRatsnestLinesEnabled() const { return m_curvedRatsnestlines; }

    inline bool GetGlobalRatsnestLinesEnabled() const { return m_globalRatsnestlines; }

    NET_COLOR_MODE GetNetColorMode() const { return m_netColorMode; }
    void SetNetColorMode( NET_COLOR_MODE aMode ) { m_netColorMode = aMode; }

    RATSNEST_MODE GetRatsnestDisplayMode() const { return m_ratsnestDisplayMode; }
    void SetRatsnestDisplayMode( RATSNEST_MODE aMode ) { m_ratsnestDisplayMode = aMode; }

    std::map<wxString, KIGFX::COLOR4D>& GetNetclassColorMap() { return m_netclassColors; }

    std::map<int, KIGFX::COLOR4D>& GetNetColorMap() { return m_netColors; }

    std::set<int>& GetHiddenNets() { return m_hiddenNets; }
    const std::set<int>& GetHiddenNets() const { return m_hiddenNets; }

    void SetZoneDisplayMode( ZONE_DISPLAY_MODE mode ) { m_zoneDisplayMode = mode; }

protected:
    ///< Maximum font size for netnames (and other dynamically shown strings)
    static const double MAX_FONT_SIZE;

    bool               m_sketchMode[GAL_LAYER_ID_END];
    bool               m_sketchGraphics;
    bool               m_sketchText;

    bool               m_padNumbers;
    bool               m_netNamesOnPads;
    bool               m_netNamesOnTracks;
    bool               m_netNamesOnVias;

    bool               m_zoneOutlines;

    bool               m_curvedRatsnestlines = true;
    bool               m_globalRatsnestlines = true;

    ZONE_DISPLAY_MODE  m_zoneDisplayMode;
    HIGH_CONTRAST_MODE m_contrastModeDisplay;
    RATSNEST_MODE      m_ratsnestDisplayMode;

    int                m_clearanceDisplayFlags;

    ///< How to display nets and netclasses with color overrides
    NET_COLOR_MODE m_netColorMode;

    ///< Overrides for specific netclass colors
    std::map<wxString, KIGFX::COLOR4D> m_netclassColors;

    ///< Overrides for specific net colors, stored as netcodes for the ratsnest to access easily
    std::map<int, KIGFX::COLOR4D> m_netColors;

    ///< Set of net codes that should not have their ratsnest displayed
    std::set<int> m_hiddenNets;

    // These opacity overrides multiply with any opacity in the base layer color
    double m_trackOpacity;     ///< Opacity override for all tracks
    double m_viaOpacity;       ///< Opacity override for all types of via
    double m_padOpacity;       ///< Opacity override for SMD pads and PTHs
    double m_zoneOpacity;      ///< Opacity override for filled zones
}; */
