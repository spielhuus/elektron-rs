use std::io::Write;

use super::PlotFormat;




pub struct Plotter {
    m_objectAttributesDictionary: String,
    m_useX2format: bool,
    m_outputFile: Box<dyn Write>,
}

impl Plotter {
    pub fn new(out: Box<dyn Write>) -> Self {
        Self {
            m_objectAttributesDictionary: String::new(),
            m_useX2format: true,
            m_outputFile: out,
        }
    }
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
    pub fn clearNetAttribute(&mut self) {
        // disable a Gerber net attribute (exists only in X2 with net attributes mode).
        if self.m_objectAttributesDictionary.is_empty() { // No net attribute or not X2 mode
            return
        }

        // Remove all net attributes from object attributes dictionary
        if self.m_useX2format {
            writeln!(self.m_outputFile, "%TD*%");
        } else {
            writeln!(self.m_outputFile, "G04 #@! TD*\n");
        }

        self.m_objectAttributesDictionary.clear();
    }
    pub fn SetLayerPolarity(&mut self, aPositive: bool) {
        if aPositive {
            writeln!(self.m_outputFile, "%%LPD*%%\n" );
        } else {
            writeln!(self.m_outputFile, "%%LPC*%%\n" );
        }
    }

}
