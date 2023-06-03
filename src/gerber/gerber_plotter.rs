use std::{io::Write, cell::RefCell, rc::Rc};
use chrono::Local;
use ndarray::{Array1, arr1, Array2};

use crate::{error::Error, sexp::{LayerId, Pcb}};

use super::{pcb_render_settings::PCB_RENDER_SETTINGS, PlotFormat, gerber_metadata::{GBR_METADATA, GBR_APERTURE_ATTRIB}, plot_params::OUTLINE_MODE, gerber_netlist_metadata::GBR_NETLIST_METADATA};

const APER_MACRO_ROUNDRECT_NAME: &'static str = "RoundRect";

const APER_MACRO_ROUNDRECT_HEADER: &'static str = 
"%AMRoundRect*\n\
0 Rectangle with rounded corners*\n\
0 $1 Rounding radius*\n\
0 $2 $3 $4 $5 $6 $7 $8 $9 X,Y pos of 4 corners*\n\
0 Add a 4 corners polygon primitive as box body*\n\
4,1,4,$2,$3,$4,$5,$6,$7,$8,$9,$2,$3,0*\n\
0 Add four circle primitives for the rounded corners*\n\
1,1,$1+$1,$2,$3*\n\
1,1,$1+$1,$4,$5*\n\
1,1,$1+$1,$6,$7*\n\
1,1,$1+$1,$8,$9*\n\
0 Add four rect primitives between the rounded corners*\n\
20,1,$1+$1,$2,$3,$4,$5,0*\n\
20,1,$1+$1,$4,$5,$6,$7,0*\n\
20,1,$1+$1,$6,$7,$8,$9,0*\n\
20,1,$1+$1,$8,$9,$2,$3,0*\
%\n";

// A aperture macro to define a rotated rect pad shape
const APER_MACRO_ROT_RECT_NAME: &'static str = "RotRect";

const APER_MACRO_ROT_RECT_HEADER: &'static str =
"%AMRotRect*\n\
0 Rectangle, with rotation*\n\
0 The origin of the aperture is its center*\n\
0 $1 length*\n\
0 $2 width*\n\
0 $3 Rotation angle, in degrees counterclockwise*\n\
0 Add horizontal line*\n\
21,1,$1,$2,0,0,$3*%\n";


// A aperture macro to define a oval pad shape
// In many gerber readers, the rotation of the full shape is broken
// so we are using a primitive that does not need a rotation to be
// plotted
const APER_MACRO_SHAPE_OVAL_NAME: &'static str = "HorizOval";

const APER_MACRO_SHAPE_OVAL_HEADER: &'static str = 
"%AMHorizOval*\n\
0 Thick line with rounded ends*\n\
0 $1 width*\n\
0 $2 $3 position (X,Y) of the first rounded end (center of the circle)*\n\
0 $4 $5 position (X,Y) of the second rounded end (center of the circle)*\n\
0 Add line between two ends*\n\
20,1,$1,$2,$3,$4,$5,0*\n\
0 Add two circle primitives to create the rounded ends*\n\
1,1,$1,$2,$3*\n\
1,1,$1,$4,$5*%\n";

// A aperture macro to define a trapezoid (polygon) by 4 corners
// and a rotation angle
const APER_MACRO_OUTLINE4P_NAME: &'static str = "Outline4P";

const APER_MACRO_OUTLINE4P_HEADER: &'static str =
"%AMOutline4P*\n\
0 Free polygon, 4 corners , with rotation*\n\
0 The origin of the aperture is its center*\n\
0 number of corners: always 4*\n\
0 $1 to $8 corner X, Y*\n\
0 $9 Rotation angle, in degrees counterclockwise*\n\
0 create outline with 4 corners*\n\
4,1,4,$1,$2,$3,$4,$5,$6,$7,$8,$1,$2,$9*%\n";

// A aperture macro to define a polygon by 5 corners
// and a rotation angle (useful for chamfered rect pads)
const APER_MACRO_OUTLINE5P_NAME: &'static str = "Outline5P";

const APER_MACRO_OUTLINE5P_HEADER: &'static str =
"%AMOutline5P*\n\
0 Free polygon, 5 corners , with rotation*\n\
0 The origin of the aperture is its center*\n\
0 number of corners: always 5*\n\
0 $1 to $10 corner X, Y*\n\
0 $11 Rotation angle, in degrees counterclockwise*\n\
0 create outline with 5 corners*\n\
4,1,5,$1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$1,$2,$11*%\n";

// A aperture macro to define a polygon by 6 corners
// and a rotation angle (useful for chamfered rect pads)
const APER_MACRO_OUTLINE6P_NAME: &'static str = "Outline6P";

const APER_MACRO_OUTLINE6P_HEADER: &'static str =
"%AMOutline6P*\n\
0 Free polygon, 6 corners , with rotation*\n\
0 The origin of the aperture is its center*\n\
0 number of corners: always 6*\n\
0 $1 to $12 corner X, Y*\n\
0 $13 Rotation angle, in degrees counterclockwise*\n\
0 create outline with 6 corners*\n\
4,1,6,$1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$1,$2,$13*%\n";

// A aperture macro to define a polygon by 7 corners
// and a rotation angle (useful for chamfered rect pads)
const APER_MACRO_OUTLINE7P_NAME: &'static str = "Outline7P";

const APER_MACRO_OUTLINE7P_HEADER: &'static str = 
"%AMOutline7P*\n\
0 Free polygon, 7 corners , with rotation*\n\
0 The origin of the aperture is its center*\n\
0 number of corners: always 7*\n\
0 $1 to $14 corner X, Y*\n\
0 $15 Rotation angle, in degrees counterclockwise*\n\
0 create outline with 7 corners*\n\
4,1,7,$1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$1,$2,$15*%\n";

// A aperture macro to define a polygon by 8 corners
// and a rotation angle (useful for chamfered rect pads)
const APER_MACRO_OUTLINE8P_NAME: &'static str = "Outline8P";

const APER_MACRO_OUTLINE8P_HEADER: &'static str =
"%AMOutline8P*\n\
0 Free polygon, 8 corners , with rotation*\n\
0 The origin of the aperture is its center*\n\
0 number of corners: always 8*\n\
0 $1 to $16 corner X, Y*\n\
0 $17 Rotation angle, in degrees counterclockwise*\n\
0 create outline with 8 corners*\n\
4,1,8,$1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$1,$2,$17*%\n";

pub enum LineWidth {
    DoNotSetLineWidth,
    UseDefaultLineWidth,
    LineWidth(f64),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum APERTURE_TYPE {
    AT_CIRCLE,              // round aperture, to flash pads
    AT_RECT,                // rect aperture, to flash pads
    AT_PLOTTING,            // round aperture, to plot lines
    AT_OVAL,                // oval aperture, to flash pads
    AT_REGULAR_POLY,        // Regular polygon (n vertices, n = 3 .. 12, with rotation)
    AT_REGULAR_POLY3,       // Regular polygon 3 vertices, with rotation
    AT_REGULAR_POLY4,       // Regular polygon 4 vertices, with rotation
    AT_REGULAR_POLY5,       // Regular polygon 5 vertices, with rotation
    AT_REGULAR_POLY6,       // Regular polygon 6 vertices, with rotation
    AT_REGULAR_POLY7,       // Regular polygon 7 vertices, with rotation
    AT_REGULAR_POLY8,       // Regular polygon 8 vertices, with rotation
    AT_REGULAR_POLY9,       // Regular polygon 9 vertices, with rotation
    AT_REGULAR_POLY10,      // Regular polygon 10 vertices, with rotation
    AT_REGULAR_POLY11,      // Regular polygon 11 vertices, with rotation
    AT_REGULAR_POLY12,      // Regular polygon 12 vertices, with rotation
    AM_ROUND_RECT,          // Aperture macro for round rect pads
    AM_ROT_RECT,            // Aperture macro for rotated rect pads
    APER_MACRO_OUTLINE4P,   // Aperture macro for trapezoid pads (outline with 4 corners)
    APER_MACRO_OUTLINE5P,   // Aperture macro for pad polygons with 5 corners (chamfered pads)
    APER_MACRO_OUTLINE6P,   // Aperture macro for pad polygons with 6 corners (chamfered pads)
    APER_MACRO_OUTLINE7P,   // Aperture macro for pad polygons with 7 corners (chamfered pads)
    APER_MACRO_OUTLINE8P,   // Aperture macro for pad polygons with 8 corners (chamfered pads)
    AM_ROTATED_OVAL,        // Aperture macro for rotated oval pads
                            // (not rotated uses a primitive)
    AM_FREE_POLYGON         // Aperture macro to create on the fly a free polygon, with
                            // only one parameter: rotation
}

#[derive(Debug, Clone)]
pub struct APERTURE {
    // Type ( Line, rect , circulaire , ovale poly 3 to 12 vertices, aperture macro )
    m_Type: APERTURE_TYPE,

    // horiz and Vert size
    m_Size: Array1<f64>,

    // list of corners for polygon shape
    m_Corners: Vec<Array1<f64>>,

    // Radius for polygon and round rect shape
    m_Radius: f64,

    // Rotation in degrees
    m_Rotation: f64,

    // code number ( >= 10 )
    m_DCode: u32,

    // the attribute attached to this aperture
    // Only one attribute is allowed by aperture
    // 0 = no specific aperture attribute
    ApertureAttribute: GBR_APERTURE_ATTRIB,
}

impl APERTURE {

    pub fn from(
        aSize: Array1<f64>,
        aType: APERTURE_TYPE,
        aRadius: f64,
        aRotDegree: f64,
        D_code: u32,
        aApertureAttribute: GBR_APERTURE_ATTRIB) -> Self {
    Self {
        m_Size: aSize,
        m_Type: aType,
        m_Radius: aRadius,
        m_Rotation: aRotDegree,
        m_DCode: D_code,
        ApertureAttribute: aApertureAttribute,
        m_Corners: Vec::new(),
    }
    }

    pub fn SetSize(&mut self, aSize: Array1<f64>) {
        self.m_Size = aSize;
    }

    pub fn GetSize(&self) -> Array1<f64> {
        self.m_Size.clone()
    }

    pub fn SetDiameter(&mut self, aDiameter: f64) {
        self.m_Radius = aDiameter/2.0;
    }

    pub fn GetDiameter(&self) -> f64 {
        // For round primitive, the diameter is the m_Size.x ot m_Size.y
        if self.m_Type == APERTURE_TYPE::AT_CIRCLE || self.m_Type == APERTURE_TYPE::AT_PLOTTING {
            self.m_Size[0];
        }
        // For rounded shapes (macro apertures), return m_Radius * 2
        // but usually they use the radius (m_Radius)
        self.m_Radius*2.0
    }

    pub fn SetRegPolyVerticeCount(&mut self, aCount: u32 ) {
        let aCount = if aCount < 3  {
            3
        } else if aCount > 12 {
            12
        } else { aCount };
        //TODO: self.m_Type = APERTURE_TYPE::AT_REGULAR_POLY3 as u32 - 3 + aCount;
    }

    pub fn GetRegPolyVerticeCount(&self) -> usize {
        self.m_Type.clone() as usize - APERTURE_TYPE::AT_REGULAR_POLY3 as usize + 3
    }

    pub fn SetRotation(&mut self, aRotDegree: f64) {
        // The rotation is stored in  degree
       self.m_Rotation = aRotDegree;
    }

    pub fn GetRotation(&self) -> f64 {
        // The rotation is stored in degree
        self.m_Rotation
    }
}

/** A class to define an aperture macros based on a free polygon, i.e. using a
 * primitive 4 to describe a free polygon with a rotation.
 * the aperture macro has only one parameter: rotation and is defined on the fly
 * for  aGerber file
 */
struct APER_MACRO_FREEPOLY {
    m_Corners: Vec<Array1<f64>>,
    m_Id: usize,
}

impl APER_MACRO_FREEPOLY {

    pub fn new(aPolygon: &Vec<Array1<f64>>, aId: usize ) -> Self {
        Self {
            m_Corners: aPolygon.to_vec(),
            m_Id: aId,
        }
    }

    /**
     * @return true if aPolygon is the same as this, i.e. if the
     * aPolygon is the same as m_Corners
     * @param aOther is the candidate to compare
     */
    pub fn IsSamePoly(&self,  aPolygon: Vec<Array1<f64>>) -> bool {
        false //TODO:
    }

    /**
     * print the aperture macro definition to aOutput
     * @param aOutput is the FILE to write
     * @param aIu2GbrMacroUnit is the scaling factor from coordinates value to
     * the Gerber file macros units (always mm or inches)
     */
    /*TODO: pub fn Format( FILE * aOutput, double aIu2GbrMacroUnit ) {
    } */

    pub fn CornersCount(&self) -> usize { self.m_Corners.len() }
}

pub struct APER_MACRO_FREEPOLY_LIST {
    m_AMList: Vec<APER_MACRO_FREEPOLY>,
}

impl APER_MACRO_FREEPOLY_LIST {

    pub fn new() -> Self {
        Self {
            m_AMList: Vec::new(),
        }
    }

    pub fn ClearList(&mut self) { self.m_AMList.clear(); }

    pub fn AmCount(&self) -> usize { self.m_AMList.len() }

    /**
     * append a new APER_MACRO_FREEPOLY containing the polygon aPolygon to the current list
     */
    pub fn Append(&mut self, aPolygon: &Vec<Array1<f64>>) {
        //TODO: 
    }

    /**
     * @return the index in m_AMList of the APER_MACRO_FREEPOLY having the
     * same polygon as aPolygon, or -1
     * @param aCandidate is the polygon candidate to compare
     */
    pub fn FindAm(&self, aPolygon: Vec<Array1<f64>>) -> usize {
        0 //TODO: 
    }

    /*
     * print the aperture macro list to aOutput
     * @param aOutput is the FILE to write
     * @param aIu2GbrMacroUnit is the scaling factor from coordinates value to
     * the Gerber file macros units (always mm or inches)
     */
    /*TODO: pub fn Format( FILE * aOutput, double aIu2GbrMacroUnit ) {
    } */

}

// A helper function to convert a X2 attribute string to a X1 structured comment:
fn makeStringCompatX1(aText: String,  aUseX1CompatibilityMode: bool) -> String {
    if aUseX1CompatibilityMode {
        format!("G04 #@! {}", aText.replace("%", ""))
    } else {
        aText
    }
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

pub fn AddGerberX2Header(aPlotter: Rc<RefCell<GERBER_PLOTTER>>, aBoard: &Pcb, aUseX1CompatibilityMode: bool ) -> Result<(), Error> {
    
    // Creates the TF,.GenerationSoftware. Format is:
    // %TF,.GenerationSoftware,<vendor>,<application name>[,<application version>]*%
    aPlotter.borrow_mut().AddLineToHeader( makeStringCompatX1(format!(
        "%TF.GenerationSoftware,{},{}*%",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    ), aUseX1CompatibilityMode ) );
    
    let rev = if !aBoard.title_block.date.is_empty() {
        aBoard.title_block.date.replace(",", "_").to_string()
    } else {
        String::from("rev?")
    };

    // creates the TF.CreationDate attribute:
    /* text = GbrMakeCreationDateAttributeString( aUseX1CompatibilityMode ?
                                                    GBR_NC_STRING_FORMAT_X1 :
                                                    GBR_NC_STRING_FORMAT_X2 );
    aPlotter.AddLineToHeader( text ); */

    // Creates the TF,.ProjectId. Format is (from Gerber file format doc):
    // %TF.ProjectId,<project id>,<project GUID>,<revision id>*%
    // <project id> is the name of the project, restricted to basic ASCII symbols only,
    // Rem: <project id> accepts only ASCII 7 code (only basic ASCII codes are allowed in
    // gerber files) and comma not accepted.
    // All illegal chars will be replaced by underscore.
    //
    // <project GUID> is a string which is an unique id of a project.
    // However Kicad does not handle such a project GUID, so it is built from the board name
    /* let filename = aBoard.filename();
    let msg = aBoard.fGetFullName(); */

    // Build a <project GUID>, from the board name
    // wxString guid = GbrMakeProjectGUIDfromString( msg );

    // build the <project id> string: this is the board short filename (without ext)
    // and all non ASCII chars and comma are replaced by '_'
    aPlotter.borrow_mut().AddLineToHeader(makeStringCompatX1(format!(
        "%TF.ProjectId,{},{},{}*%",
        aBoard.title_block.title.replace(",", "_"),
        guid(&aBoard.filename().unwrap()),
        rev
    ), aUseX1CompatibilityMode ) );

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
        registration_id.Printf( wxT( "PX%xPY%x" ), auxOrigin.x, auxOrigin.y );

    text.Printf( wxT( "%%TF.SameCoordinates,%s*%%" ), registration_id.GetData() ); */
    aPlotter.borrow_mut().AddLineToHeader( makeStringCompatX1(format!("%TF.SameCoordinates,PX{}xPY{}*%", 100, 100), aUseX1CompatibilityMode ) );
    Ok(())
}

pub fn AddGerberX2Attribute(aPlotter: Rc<RefCell<GERBER_PLOTTER>>, aBoard: &Pcb, aLayer: LayerId,
                           aUseX1CompatibilityMode: bool )
{
    AddGerberX2Header(aPlotter.clone(), aBoard, aUseX1CompatibilityMode );

    //Add the TF.FileFunction
    aPlotter.borrow_mut().AddLineToHeader(makeStringCompatX1(layer_function(aLayer), aUseX1CompatibilityMode));

    // Add the TF.FilePolarity (for layers which support that)
    /* text = GetGerberFilePolarityAttribute( aLayer );

    if( !text.IsEmpty() )
        aPlotter->AddLineToHeader( makeStringCompatX1( text, aUseX1CompatibilityMode ) ); */
}


pub struct GERBER_PLOTTER {
    m_gerberDisableApertMacros: bool,
    m_useX2format: bool,
    m_useNetAttributes: bool,
    m_headerExtraLines: Vec<String>,
    m_renderSettings: PCB_RENDER_SETTINGS,
    m_objectAttributesDictionary: String,
    m_outputFile: Box<dyn Write>,
    m_tmpFile: Vec<String>,
    m_gerberUnitInch: bool,
    m_gerberUnitFmt: usize,
    m_penState: Option<char>,
    m_currentPenWidth: f64,

    m_currentApertureIdx: usize, // The index of the current aperture in m_apertures
    m_apertures: Vec<APERTURE>,

    m_hasApertureRoundRect: bool,     // true is at least one round rect aperture is in use
    m_hasApertureRotOval: bool,       // true is at least one oval rotated aperture is in use
    m_hasApertureRotRect: bool,       // true is at least one rect. rotated aperture is in use
    m_hasApertureOutline4P: bool,     // true is at least one rotated rect/trapezoid aperture
                                        // is in use
    m_hasApertureChamferedRect: bool, // true is at least one chamfered rect is in use
    m_am_freepoly_list: APER_MACRO_FREEPOLY_LIST,
}

impl GERBER_PLOTTER {
    pub fn new(out: Box<dyn Write>) -> Self {
        Self {
            m_gerberDisableApertMacros: false,
            m_useX2format: true,
            m_useNetAttributes: false,
            m_headerExtraLines: Vec::new(),
            m_renderSettings: PCB_RENDER_SETTINGS::new(),
            m_objectAttributesDictionary: String::new(),
            m_outputFile: out,
            m_tmpFile: Vec::new(),
            m_am_freepoly_list: APER_MACRO_FREEPOLY_LIST::new(),
            m_gerberUnitInch: false,
            m_gerberUnitFmt: 0,
            m_penState: None,
            m_currentPenWidth: 0.0,
            m_currentApertureIdx: usize::MAX,
            m_apertures: Vec::new(),
            m_hasApertureRoundRect: false,
            m_hasApertureRotOval: false,
            m_hasApertureRotRect: false,
            m_hasApertureOutline4P: false,
            m_hasApertureChamferedRect: false,
        }
    }
    pub fn OpenFile(&mut self, filename: String) -> Result<(), Error> {
        Ok(())
    }

    /**
     * Set the line width for the next drawing.
     *
     * @param width is specified in IUs.
     * @param aData is an auxiliary parameter, mainly used in gerber plotter.
     */
    pub fn SetCurrentLineWidth(&mut self, width: LineWidth, aData: &GBR_METADATA) {
        let width = match width {
            LineWidth::DoNotSetLineWidth => return,
            LineWidth::UseDefaultLineWidth => {
                self.m_renderSettings.GetDefaultPenWidth()
            },
            LineWidth::LineWidth(w) => w,
        };

        // GBR_METADATA* gbr_metadata = static_cast<GBR_METADATA*>( aData );
        let aperture_attribute = aData.GetApertureAttrib();

        self.selectAperture( arr1(&[width, width]), 0.0, 0.0, APERTURE_TYPE::AT_PLOTTING, aperture_attribute);

        self.m_currentPenWidth = width;
    }

    pub fn GetCurrentLineWidth(&self) -> f64 { self.m_currentPenWidth }
    pub fn StartPlot(&mut self) -> Result<(), Error>{
        let m_hasApertureRoundRect = false;     // true is at least one round rect aperture is in use
        let m_hasApertureRoundRect = false;     // true is at least one round rect aperture is in use
        let m_hasApertureRotOval = false;       // true is at least one oval rotated aperture is in use
        let m_hasApertureRotRect = false;       // true is at least one rect. rotated aperture is in use
        let m_hasApertureOutline4P = false;     // true is at least one rotated rect/trapezoid aperture
                                             // is in use
        let m_hasApertureChamferedRect = false; // true is at least one chamfered rect is in use
        self.m_am_freepoly_list.ClearList();

        /* finalFile = m_outputFile;     // the actual gerber file will be created later

        // Create a temp file in system temp to avoid potential network share buffer issues for
        // the final read and save.
        m_workFilename = wxFileName::CreateTempFileName( wxEmptyString );
        workFile   = wxFopen( m_workFilename, wxT( "wt" ));
        m_outputFile = workFile;
        wxASSERT( m_outputFile );

        if( m_outputFile == nullptr )
            return false; */

        for line in &self.m_headerExtraLines {
            if !line.is_empty() {
                self.m_tmpFile.push(line.to_string()); //TODO: TO_UTF8( m_headerExtraLines[ii] ) );
            }
        }

        // Set coordinate format to 3.6 or 4.5 absolute, leading zero omitted
        // the number of digits for the integer part of coordinates is needed
        // in gerber format, but is not very important when omitting leading zeros
        // It is fixed here to 3 (inch) or 4 (mm), but is not actually used
        let leadingDigitCount = if self.m_gerberUnitInch { 3 } else { 4 };

        self.m_tmpFile.push(format!("%FSLAX{}{}Y{}{}*%",
                 leadingDigitCount, self.m_gerberUnitFmt,
                 leadingDigitCount, self.m_gerberUnitFmt));
        self.m_tmpFile.push(format!(
                 "G04 Gerber Fmt {}.{}, Leading zero omitted, Abs format (unit {})*",
                 leadingDigitCount, self.m_gerberUnitFmt,
                 if self.m_gerberUnitInch { "inch" } else { "mm" }));

        // let Title = m_creator + wxT( " " ) + GetBuildVersion();

        // In gerber files, ASCII7 chars only are allowed.
        // So use a ISO date format (using a space as separator between date and time),
        // not a localized date format
        let datetime = Local::now();
        self.m_tmpFile.push(format!("G04 Created by {} ({}) date {}*",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            datetime));
                 // TO_UTF8( Title ), TO_UTF8( date.FormatISOCombined( ' ') ) );

        /* Mass parameter: unit = INCHES/MM */
        if self.m_gerberUnitInch {
            self.m_tmpFile.push(String::from("%MOIN*%"));
        } else {
            self.m_tmpFile.push(String::from("%MOMM*%"));
        }

        // Be sure the usual dark polarity is selected:
        self.m_tmpFile.push(String::from("%LPD*%"));

        // Set initial interpolation mode: always G01 (linear):
        self.m_tmpFile.push(String::from("G01*"));

        // Add aperture list start point
        self.m_tmpFile.push(String::from("G04 APERTURE LIST*"));

        // Give a minimal value to the default pen size, used to plot items in sketch mode
        /* if( m_renderSettings ) {
            const int pen_min = 0.1 * m_IUsPerDecimil * 10000 / 25.4;   // for min width = 0.1 mm
            m_renderSettings->SetDefaultPenWidth( std::max( m_renderSettings->GetDefaultPenWidth(),
                                                            pen_min ) );
        } */
        Ok(())
    }
    pub fn AddLineToHeader(&mut self, aExtraString: String) {
        self.m_headerExtraLines.push( aExtraString );
    }
    pub fn ClearHeaderLinesList(&mut self) {
        self.m_headerExtraLines.clear();
    }
    pub fn DisableApertMacros(&mut self, aDisable: bool ) { self.m_gerberDisableApertMacros = aDisable; }
    pub fn UseX2format(&mut self, aEnable: bool ) { self.m_useX2format = aEnable; }
    pub fn UseX2NetAttributes(&mut self, aEnable: bool ) { self.m_useNetAttributes = aEnable; }
    pub fn SetRenderSettings(&mut self, aSettings: PCB_RENDER_SETTINGS) { self.m_renderSettings = aSettings; }
    pub fn RenderSettings(&self) -> PCB_RENDER_SETTINGS { self.m_renderSettings.clone() } //TODO: }

    pub fn GetPlotterType(&self) -> PlotFormat {
        PlotFormat::GERBER
    }
    pub fn StartBlock(&mut self) {
        // Currently, it is the same as EndBlock(): clear all aperture net attributes
        self.EndBlock();
    }

    pub fn EndBlock(&mut self) {
        // Remove all net attributes from object attributes dictionary
        self.clearNetAttribute();
    }

    pub fn EndPlot(&mut self) -> Result<(), Error> {

        // Placement of apertures in RS274X
        for line in &self.m_tmpFile {
            writeln!(self.m_outputFile, "{}", line)?; //TODO: write without format

            // char* substr = strtok( line, "\n\r" );

            if line ==  "G04 APERTURE LIST*" {
                // Add aperture list macro:
                if self.m_hasApertureRoundRect | self.m_hasApertureRotOval ||
                    self.m_hasApertureOutline4P || self.m_hasApertureRotRect ||
                    self.m_hasApertureChamferedRect || self.m_am_freepoly_list.AmCount() > 0 {

                    writeln!(self.m_outputFile, "G04 Aperture macros list*\n")?;

                    if self.m_hasApertureRoundRect {
                        self.m_outputFile.write_all(APER_MACRO_ROUNDRECT_HEADER.as_bytes())?;
                    }
                    if self.m_hasApertureRotOval {
                        self.m_outputFile.write_all(APER_MACRO_SHAPE_OVAL_HEADER.as_bytes())?;
                    }
                    if self.m_hasApertureRotRect {
                        self.m_outputFile.write_all(APER_MACRO_ROT_RECT_HEADER.as_bytes())?;
                    }

                    if self.m_hasApertureOutline4P {
                        self.m_outputFile.write_all(APER_MACRO_OUTLINE4P_HEADER.as_bytes())?;
                    }

                    if self.m_hasApertureChamferedRect {
                        self.m_outputFile.write_all(APER_MACRO_OUTLINE5P_HEADER.as_bytes())?;
                        self.m_outputFile.write_all(APER_MACRO_OUTLINE6P_HEADER.as_bytes())?;
                        self.m_outputFile.write_all(APER_MACRO_OUTLINE7P_HEADER.as_bytes())?;
                        self.m_outputFile.write_all(APER_MACRO_OUTLINE8P_HEADER.as_bytes())?;
                    }

                    /*TODO: if self.m_am_freepoly_list.AmCount() > 0 {
                        // aperture sizes are in inch or mm, regardless the
                        // coordinates format
                        let fscale = 0.0001 * self.m_plotScale / self.m_IUsPerDecimil; // inches

                        if !m_gerberUnitInch {
                            fscale *= 25.4;     // size in mm
                        }

                        m_am_freepoly_list.Format( m_outputFile, fscale );
                    } */

                    writeln!(self.m_outputFile, "G04 Aperture macros list end*");
                }

                //TODO: writeApertureList();
                writeln!(self.m_outputFile, "G04 APERTURE END LIST*");
            }
        }

        /* fclose( workFile );
        fclose( finalFile );
        ::wxRemoveFile( m_workFilename );
        m_outputFile = nullptr; */

        writeln!(self.m_outputFile, "M02*")?;

        Ok(())
    }

    pub fn clearNetAttribute(&mut self) {
        // disable a Gerber net attribute (exists only in X2 with net attributes mode).
        if self.m_objectAttributesDictionary.is_empty() { // No net attribute or not X2 mode
            return
        }

        // Remove all net attributes from object attributes dictionary
        if self.m_useX2format {
            self.m_tmpFile.push(String::from("%TD*%"));
        } else {
            self.m_tmpFile.push(String::from("G04 #@! TD*"));
        }

        self.m_objectAttributesDictionary.clear();
    }

    pub fn formatNetAttribute(&mut self, aData: GBR_NETLIST_METADATA) {
        // print a Gerber net attribute record.
        // it is added to the object attributes dictionary
        // On file, only modified or new attributes are printed.
        /* if( aData == nullptr )
            return; */

        if !self.m_useNetAttributes {
            return;
        }

        let useX1StructuredComment = !self.m_useX2format;

        let mut clearDict = false;
        let mut short_attribute_string = String::new();

        /*TODO: if( !FormatNetAttribute( short_attribute_string, m_objectAttributesDictionary,
                            aData, clearDict, useX1StructuredComment ) ) {
            return;
        } */

        if clearDict {
            self.clearNetAttribute();
        }

        if( !short_attribute_string.is_empty() ) {
            // fputs( short_attribute_string.c_str(), m_outputFile );
            self.m_tmpFile.push(short_attribute_string);
        }

        if( self.m_useX2format && !aData.m_ExtraData.is_empty() ) {
            /* std::string extra_data = TO_UTF8( aData->m_ExtraData );
            fputs( extra_data.c_str(), m_outputFile ); */
            self.m_tmpFile.push(aData.m_ExtraData);
        }
    }

    pub fn SetLayerPolarity(&mut self, aPositive: bool) {
        if aPositive {
            self.m_tmpFile.push(String::from("%LPD*%"));
        } else {
            self.m_tmpFile.push(String::from("%LPC*%"));
        }
    }
    pub fn ThickSegment(&mut self, start: Array1<f64>, end: Array1<f64>, width: f64,
                                    tracemode: OUTLINE_MODE, aData: &GBR_METADATA ) {
    // if tracemode == OUTLINE_MODE::Filled {
        // GBR_METADATA *gbr_metadata = static_cast<GBR_METADATA*>( aData );
        self.SetCurrentLineWidth( LineWidth::LineWidth(width), &aData );

        // if( gbr_metadata )
            self.formatNetAttribute(aData.m_NetlistMetadata.clone());

        self.MoveTo( start );
        self.FinishTo( end );
    /* }
    else
    {
        SetCurrentLineWidth( USE_DEFAULT_LINE_WIDTH );
        segmentAsOval( start, end, width, tracemode );
    } */
    }

    // Convenience functions for PenTo
    pub fn MoveTo(&mut self, pos: Array1<f64>) {
        self.PenTo( pos, 'U' );
    }
    pub fn FinishTo(&mut self, pos: Array1<f64>) {
        self.PenTo( pos.clone(), 'D' );
        self.PenTo( pos, 'Z' );
    }
    pub fn PenTo(&mut self, aPos: Array1<f64>, plume: char ) {
        let pos_dev = self.userToDeviceCoordinates( aPos );

        match plume {
            'Z' => {},
            'U' => {
                self.emitDcode( pos_dev, 2 );
            },
            'D' => {
                self.emitDcode( pos_dev, 1 );
            },
            _ => {}
        }

        self.m_penState = Some(plume);
    }
    pub fn emitDcode(&mut self, pt: Array1<f64>, dcode: u8 ) {

        // fprintf( m_outputFile, "X%dY%dD%02d*\n", KiROUND( pt.x ), KiROUND( pt.y ), dcode );
        // self.m_tmpFile.push("X{}Y{}D{}*\n", KiROUND( pt.x ), KiROUND( pt.y ), dcode );
        self.m_tmpFile.push(format!("X{}Y{}D{}*\n", pt[0], pt[1], dcode ));
    }

    pub fn userToDeviceCoordinates(&self, aCoordinate: Array1<f64>) -> Array1<f64> {
        /* wxPoint pos = aCoordinate - m_plotOffset;

        double x = pos.x * m_plotScale;
        double y = ( m_paperSize.y - pos.y * m_plotScale );

        if( m_plotMirror )
        {
            if( m_mirrorIsHorizontal )
                x = ( m_paperSize.x - pos.x * m_plotScale );
            else
                y = pos.y * m_plotScale;
        }

        if( m_yaxisReversed )
            y = m_paperSize.y - y;

        x *= m_iuPerDeviceUnit;
        y *= m_iuPerDeviceUnit;

        return DPOINT( x, y ); */
        aCoordinate
    }
    pub fn selectAperture(&mut self, aSize: Array1<f64>, aRadius: f64, aRotDegree: f64,
                                         aType: APERTURE_TYPE, aApertureAttribute: GBR_APERTURE_ATTRIB) {

        let mut change = ( self.m_currentApertureIdx == usize::MAX ) ||
                      ( self.m_apertures[self.m_currentApertureIdx].m_Type != aType ) ||
                      ( self.m_apertures[self.m_currentApertureIdx].m_Size != aSize ) ||
                      ( self.m_apertures[self.m_currentApertureIdx].m_Radius != aRadius ) ||
                      ( self.m_apertures[self.m_currentApertureIdx].m_Rotation != aRotDegree );

        if !change  {
            change = self.m_apertures[self.m_currentApertureIdx].ApertureAttribute != aApertureAttribute;
        }

        if change {
            // Pick an existing aperture or create a new one
            self.m_currentApertureIdx = self.GetOrCreateAperture( aSize, aRadius, aRotDegree,
                                                        aType, aApertureAttribute );
            self.m_tmpFile.push(format!("D{}*", self.m_apertures[self.m_currentApertureIdx].m_DCode ));
        }
    }

    pub fn GetOrCreateAperture(&mut self, aSize: Array1<f64>, aRadius: f64, aRotDegree: f64,
                         aType: APERTURE_TYPE, aApertureAttribute: GBR_APERTURE_ATTRIB ) -> usize {

        let mut last_D_code = 9;

        // Search an existing aperture
        // for( int idx = 0; idx < (int)m_apertures.size(); ++idx )
        for (idx, tool) in self.m_apertures.iter().enumerate() {
            // APERTURE* tool = &m_apertures[idx];
            last_D_code = tool.m_DCode;

            if (tool.m_Type == aType) && (tool.m_Size == aSize) &&
                (tool.m_Radius == aRadius) && (tool.m_Rotation == aRotDegree) &&
                (tool.ApertureAttribute == aApertureAttribute) {
                return idx;
            }
        }

        // Allocate a new aperture
        let new_tool = APERTURE::from(aSize, aType, aRadius, aRotDegree, last_D_code + 1, aApertureAttribute);
        self.m_apertures.push( new_tool );

        self.m_apertures.len() - 1
    }
}
