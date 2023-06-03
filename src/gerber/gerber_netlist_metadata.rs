

pub fn ConvertNotAllowedCharsInGerber(aString: String, aAllowUtf8Chars: bool, aQuoteString: bool) -> String {
    /* format string means convert any code > 0x7E and unauthorized codes to a hexadecimal
     * 16 bits sequence Unicode
     * However if aAllowUtf8Chars is true only unauthorized codes will be escaped, because some
     * Gerber files accept UTF8 chars.
     * unauthorized codes are ',' '*' '%' '\' '"' and are used as separators in Gerber files
     */
    let mut txt = String::new();

    if aQuoteString { 
        txt += "\"";
    }

    for code in aString.chars() {
        // wxChar code = aString[ii];
        let mut convert = false;

        match code {
            '\\'|'%'|'*'|',' => { 
                convert = true;
            }

            '"' => {
                if aQuoteString {
                    convert = true;
                }
            }

            _ => {}
        }

        if !aAllowUtf8Chars && code as u8 > 0x7F {
            convert = true;
        }

        if convert {
            // Convert code to 4 hexadecimal digit
            // (Gerber allows only 4 hexadecimal digit) in escape seq:
            // "\uXXXX", XXXX is the Unicode 16 bits hexa value
            /* char hexa[32];
            sprintf( hexa,"\\u%4.4X", code & 0xFFFF); */
            txt += format!("{:#04x}", code as u8).as_str();
        } else {
            txt += code.to_string().as_str();
        }
    }

    if aQuoteString {
        txt += "\"";
    }

    return txt;
}


// This enum enables the different net attributes attached to the object
// the values can be ORed for items which can have more than one attribute
// (A flashed pad has all allowed attributes)
#[derive(Debug, Clone)]
pub enum GBR_NETINFO_TYPE {
    GBR_NETINFO_UNSPECIFIED,    //< idle command (no command)
    GBR_NETINFO_PAD = 1,        //< print info associated to a flashed pad (TO.P attribute)
    GBR_NETINFO_NET = 2,        //< print info associated to a net (TO.N attribute)
    GBR_NETINFO_CMP = 4,        //< print info associated to a component (TO.C attribute)
}

#[derive(Debug, Clone)]
pub struct GBR_DATA_FIELD {
    m_field: String,        //< the Unicode text to print in Gbr file
                            //< (after escape and quoting)
    m_useUTF8: bool,        //< true to use UTF8, false to escape non ASCII7 chars
    m_escapeString: bool,   //< true to quote the field in gbr file
}

impl GBR_DATA_FIELD {
    pub fn new() -> Self {
        Self {
            m_field: String::new(),
            m_useUTF8: false,
            m_escapeString: false,
        }
    }

    pub fn clear(&mut self) {
        self.m_field.clear();
        self.m_useUTF8 = false;
        self.m_escapeString = false;
    }

    pub fn GetValue(&self) -> String { self.m_field.to_string() }

    pub fn SetField(&mut self, aField: String, aUseUTF8: bool, aEscapeString: bool ) {
        self.m_field = aField;
        self.m_useUTF8 = aUseUTF8;
        self.m_escapeString = aEscapeString;
    }

    pub fn IsEmpty(&self) -> bool { self.m_field.is_empty() }

    pub fn GetGerberString(&self) -> String {
        let mut converted = String::new();

        if !self.m_field.is_empty() {
            converted = ConvertNotAllowedCharsInGerber( self.m_field.to_string(), self.m_useUTF8, self.m_escapeString );
        }

        // Convert the char string to std::string. Be careful when converting a wxString to
        // a std::string: using static_cast<const char*> is mandatory
        // std::string txt = static_cast<const char*>( converted.utf8_str() );

        return converted;
    }
}

/**
 * Information which can be added in a gerber file as attribute of an object.
 *
 * The #GBR_INFO_TYPE types can be OR'ed to add 2 (or more) attributes.  There are only 3
 * net attributes defined attached to an object by the %TO command:
 *  - %TO.P
 *  - %TO.N
 *  - %TO.C
 *
 * The .P attribute can be used only for flashed pads (using the D03 command) and only for
 * external copper layers, if the component is on a external copper layer for other copper
 * layer items (pads on internal layers, tracks ... ), only .N and .C can be used.
 */
#[derive(Debug, Clone)]
pub struct GBR_NETLIST_METADATA{
    // these members are used in the %TO object attributes command.
    m_NetAttribType: GBR_NETINFO_TYPE, //< the type of net info
                                ///< (used to define the gerber string to create)
    m_NotInNet: bool,           ///< true if a pad of a footprint cannot be connected
                                ///< (for instance a mechanical NPTH, ot a not named pad)
                                ///< in this case the pad net name is empty in gerber file
    m_Padname: GBR_DATA_FIELD,  ///< for a flashed pad: the pad name ((TO.P attribute)
    m_PadPinFunction: GBR_DATA_FIELD,  ///< for a pad: the pin function (defined in schematic)
    m_Cmpref: String,    ///< the component reference parent of the data
    m_Netname: String,   ///< for items associated to a net: the netname

    pub m_ExtraData: String,       ///< a string to print after %TO object attributes, if not empty
                                ///< it is printed "as this"
    /**
     * If true, do not clear all attributes when a attribute has changed.  This is useful
     * when some attributes need to be persistent.   If false, attributes will be cleared
     * if only one attribute cleared.  This is a more secure way to set attributes, when
     * all attribute changes are not safely managed.
     */
    m_TryKeepPreviousAttributes: bool,
}

impl GBR_NETLIST_METADATA {

    pub fn new() -> Self {
        Self {
            m_NetAttribType: GBR_NETINFO_TYPE::GBR_NETINFO_UNSPECIFIED,
            m_NotInNet: false,
            m_Padname: GBR_DATA_FIELD::new(),
            m_PadPinFunction: GBR_DATA_FIELD::new(),
            m_Cmpref: String::new(),
            m_Netname: String::new(),
            m_ExtraData: String::new(),
            m_TryKeepPreviousAttributes: false, 
        }
    }

    /**
     * Clear the extra data string printed at end of net attributes.
     */
    pub fn ClearExtraData(&mut self) {
        self.m_ExtraData.clear();
    }

    /**
     * Set the extra data string printed at end of net attributes
     */
    pub fn SetExtraData(&mut self, aExtraData: String) {
        self.m_ExtraData = aExtraData;
    }

    /**
     * Remove the net attribute specified by \a aName.
     *
     * If aName == NULL or empty, remove all attributes.
     *
     * @param aName is the name (.CN, .P .N or .C) of the attribute to remove.
     */
    pub fn ClearAttribute(&mut self, aName: String ) {
        //TODO:
        /* if( self.m_NetAttribType == GBR_NETINFO_UNSPECIFIED )
        {
            m_Padname.clear();
            m_PadPinFunction.clear();
            m_Cmpref.clear();
            m_Netname.clear();
            return;
        }

        if aName.is_empty() || aName == ".CN" {
            self.m_NetAttribType = GBR_NETINFO_UNSPECIFIED;
            self.m_Padname.clear();
            self.m_PadPinFunction.clear();
            self.m_Cmpref.clear();
            self.m_Netname.clear();
            return;
        }

        if aName == ".C" {
            m_NetAttribType &= ~GBR_NETINFO_CMP;
            m_Cmpref.clear();
            return;
        }

        if( *aName == wxT( ".N" ) )
        {
            m_NetAttribType &= ~GBR_NETINFO_NET;
            m_Netname.clear();
            return;
        }

        if( *aName == wxT( ".P" ) )
        {
            m_NetAttribType &= ~GBR_NETINFO_PAD;
            m_Padname.clear();
            m_PadPinFunction.clear();
            return;
        } */
    }
}

