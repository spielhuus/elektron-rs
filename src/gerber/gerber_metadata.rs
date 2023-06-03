use super::gerber_netlist_metadata::GBR_NETLIST_METADATA;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum GBR_APERTURE_ATTRIB {
    GBR_APERTURE_ATTRIB_NONE,           ///< uninitialized attribute.
    GBR_APERTURE_ATTRIB_ETCHEDCMP,      ///< aperture used for etched components.

    ///< aperture used for connected items like tracks (not vias).
    GBR_APERTURE_ATTRIB_CONDUCTOR,
    GBR_APERTURE_ATTRIB_EDGECUT,        ///< aperture used for board cutout,

    ///< aperture used for not connected items (texts, outlines on copper).
    GBR_APERTURE_ATTRIB_NONCONDUCTOR,
    GBR_APERTURE_ATTRIB_VIAPAD,         ///< aperture used for vias.

    ///< aperture used for through hole component on outer layer.
    GBR_APERTURE_ATTRIB_COMPONENTPAD,

    ///< aperture used for SMD pad. Excluded BGA pads which have their own type.
    GBR_APERTURE_ATTRIB_SMDPAD_SMDEF,

    ///< aperture used for SMD pad with a solder mask defined by the solder mask.
    GBR_APERTURE_ATTRIB_SMDPAD_CUDEF,

    ///< aperture used for BGA pads with a solder mask defined by the copper shape.
    GBR_APERTURE_ATTRIB_BGAPAD_SMDEF,

    ///< aperture used for BGA pad with a solder mask defined by the solder mask.
    GBR_APERTURE_ATTRIB_BGAPAD_CUDEF,

    ///< aperture used for edge connector pad (outer layers).
    GBR_APERTURE_ATTRIB_CONNECTORPAD,
    GBR_APERTURE_ATTRIB_WASHERPAD,      ///< aperture used for mechanical pads (NPTH).
    GBR_APERTURE_ATTRIB_TESTPOINT,      ///< aperture used for test point pad (outer layers).

    ///< aperture used for fiducial pad (outer layers), at board level.
    GBR_APERTURE_ATTRIB_FIDUCIAL_GLBL,

    ///< aperture used for fiducial pad (outer layers), at footprint level.
    GBR_APERTURE_ATTRIB_FIDUCIAL_LOCAL,

    ///< aperture used for heat sink pad (typically for SMDs).
    GBR_APERTURE_ATTRIB_HEATSINKPAD,

    ///< aperture used for castellated pads in copper layer files.
    GBR_APERTURE_ATTRIB_CASTELLATEDPAD,

    ///< aperture used for castellated pads in drill files.
    GBR_APERTURE_ATTRIB_CASTELLATEDDRILL,

    GBR_APERTURE_ATTRIB_VIADRILL,       ///< aperture used for via holes in drill files.
    GBR_APERTURE_ATTRIB_CMP_DRILL,      ///< aperture used for pad holes in drill files.

    ///< aperture used for pads oblong holes in drill files.
    GBR_APERTURE_ATTRIB_CMP_OBLONG_DRILL,

    ///< aperture used for flashed cmp position in placement files.
    GBR_APERTURE_ATTRIB_CMP_POSITION,

    ///< aperture used for flashed pin 1 (or A1 or AA1) position in placement files.
    GBR_APERTURE_ATTRIB_PAD1_POSITION,

    ///< aperture used for flashed pads position in placement files.
    GBR_APERTURE_ATTRIB_PADOTHER_POSITION,

    ///< aperture used to draw component physical body outline without pins in placement files.
    GBR_APERTURE_ATTRIB_CMP_BODY,

    ///< aperture used to draw component physical body outline with pins in placement files.
    GBR_APERTURE_ATTRIB_CMP_LEAD2LEAD,

    ///< aperture used to draw component footprint bounding box in placement files.
    GBR_APERTURE_ATTRIB_CMP_FOOTPRINT,

    ///< aperture used to draw component outline courtyard in placement files.
    GBR_APERTURE_ATTRIB_CMP_COURTYARD,
}

pub struct GBR_METADATA {
    m_ApertAttribute: GBR_APERTURE_ATTRIB,
    pub m_NetlistMetadata: GBR_NETLIST_METADATA,
}

impl GBR_METADATA {
    pub fn new() -> Self {
        Self {
            m_ApertAttribute: GBR_APERTURE_ATTRIB::GBR_APERTURE_ATTRIB_NONE,
            m_NetlistMetadata: GBR_NETLIST_METADATA::new(),
        }
    }
    pub fn GetApertureAttrib(&self) -> GBR_APERTURE_ATTRIB {
         self.m_ApertAttribute.clone()
    }
}
