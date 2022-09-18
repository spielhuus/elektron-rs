use std::io::Write;

use crate::error::Error;

use super::model::{
    Bus, BusEntry, Effects, Footprint, FpArc, FpCircle, FpLine, FpText, GlobalLabel, GrLine,
    GrText, HierarchicalLabel, Junction, Label, Layers, LibrarySymbol, Model, Net, NoConnect, Pad,
    Polyline, Property, Segment, Sheet, SheetInstance, Stroke, Symbol, SymbolInstance, Text,
    TitleBlock, Via, Wire, Zone,
};

pub trait SexpWriter {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error>;
}

impl SexpWriter for TitleBlock {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(title_block\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(title \"")?;
        out.write_all(self.title.as_bytes())?;
        out.write_all(b"\")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(date \"")?;
        out.write_all(self.date.as_bytes())?;
        out.write_all(b"\")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(rev \"")?;
        out.write_all(self.rev.as_bytes())?;
        out.write_all(b"\")\n")?;
        if !self.company.is_empty() {
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b"(company \"")?;
            out.write_all(self.company.as_bytes())?;
            out.write_all(b"\")\n")?;
        }
        for (index, comment) in &self.comment {
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b"(comment ")?;
            out.write_all(index.to_string().as_bytes())?;
            out.write_all(b" \"")?;
            out.write_all(comment.as_bytes())?;
            out.write_all(b"\")\n")?;
        }
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n\n")?;
        Ok(())
    }
}
impl SexpWriter for Stroke {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(stroke (width ")?;
        out.write_all(self.width.to_string().as_bytes())?;
        out.write_all(b") (type ")?;
        out.write_all(self.linetype.as_bytes())?;
        out.write_all(b") (color ")?;
        out.write_all(self.color.0.to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.color.1.to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.color.2.to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.color.2.to_string().as_bytes())?;
        out.write_all(b"))\n")?;
        Ok(())
    }
}
impl SexpWriter for Effects {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(effects (font (size ")?;
        out.write_all(self.font_size.0.to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.font_size.1.to_string().as_bytes())?;
        out.write_all(b")")?;
        if self.thickness != -1.0 {
            out.write_all(b" (thickness ")?;
            out.write_all(self.thickness.to_string().as_bytes())?;
            out.write_all(b")")?;
        }
        if self.bold {
            out.write_all(b" bold")?;
        }
        if self.italic {
            out.write_all(b" italic")?;
        }
        out.write_all(b")")?;
        if !self.justify.is_empty() {
            out.write_all(b" (justify")?;
            for j in &self.justify {
                out.write_all(b" ")?;
                out.write_all(j.as_bytes())?;
            }
            out.write_all(b")")?;
        }
        if self.hide {
            out.write_all(b" hide")?;
        }
        out.write_all(b")")?;
        Ok(())
    }
}
impl SexpWriter for Property {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(property \"")?;
        out.write_all(self.key.as_bytes())?;
        out.write_all(b"\" \"")?;
        out.write_all(self.value.as_bytes())?;
        out.write_all(b"\" (id ")?;
        out.write_all(self.id.to_string().as_bytes())?;
        out.write_all(b") (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.angle.to_string().as_bytes())?;
        out.write_all(b")")?;
        if let Some(effects) = &self.effects {
            out.write_all(b"\n")?;
            effects.write(out, indent + 1)?;
            out.write_all(b"\n")?;
            out.write_all("  ".repeat(indent).as_bytes())?;
            out.write_all(b")\n")?;
        } else {
            out.write_all(b")\n")?;
        }
        Ok(())
    }
}
impl SexpWriter for Junction {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(junction (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (diameter ")?;
        out.write_all(self.diameter.to_string().as_bytes())?;
        out.write_all(b") (color ")?;
        out.write_all(self.color.0.to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.color.1.to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.color.2.to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.color.2.to_string().as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n  )\n")?;
        Ok(())
    }
}
impl SexpWriter for Label {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(label \"")?;
        out.write_all(self.text.as_bytes())?;
        out.write_all(b"\" (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.angle.to_string().as_bytes())?;
        out.write_all(b")\n")?;
        self.effects.write(out, indent + 1)?;
        out.write_all(b"\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for GlobalLabel {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(global_label \"")?;
        out.write_all(self.text.as_bytes())?;
        out.write_all(b"\" (shape ")?;
        out.write_all(self.shape.as_bytes())?;
        out.write_all(b") (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.angle.to_string().as_bytes())?;
        out.write_all(b")")?;
        if self.fields_autoplaced {
            out.write_all(b" (fields_autoplaced)")?;
        }
        out.write_all(b"\n")?;
        self.effects.write(out, indent + 1)?;
        out.write_all(b"\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n")?;
        self.property.write(out, indent + 1)?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for HierarchicalLabel {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(hierarchical_label \"")?;
        out.write_all(self.text.as_bytes())?;
        out.write_all(b"\" (shape ")?;
        out.write_all(self.shape.as_bytes())?;
        out.write_all(b") (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.angle.to_string().as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b"\n")?;
        self.effects.write(out, indent + 1)?;
        out.write_all(b"\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for NoConnect {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(no_connect (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b"))\n")?;
        Ok(())
    }
}
impl SexpWriter for Wire {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(wire (pts (xy ")?;
        out.write_all(self.pts.get((0, 0)).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.pts.get((0, 1)).unwrap().to_string().as_bytes())?;
        out.write_all(b") (xy ")?;
        out.write_all(self.pts.get((1, 0)).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.pts.get((1, 1)).unwrap().to_string().as_bytes())?;
        out.write_all(b"))\n")?;
        self.stroke.write(out, indent + 1)?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for Bus {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(bus (pts (xy ")?;
        out.write_all(self.pts.get((0, 0)).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.pts.get((0, 1)).unwrap().to_string().as_bytes())?;
        out.write_all(b") (xy ")?;
        out.write_all(self.pts.get((1, 0)).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.pts.get((1, 1)).unwrap().to_string().as_bytes())?;
        out.write_all(b"))\n")?;
        self.stroke.write(out, indent + 1)?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for BusEntry {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(bus_entry")?;
        out.write_all(b" (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") ")?;
        out.write_all(b"(size ")?;
        out.write_all(self.size.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.size.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b")\n")?;
        self.stroke.write(out, indent + 1)?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for Text {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(text \"")?;
        out.write_all(self.text.as_bytes())?;
        out.write_all(b"\" (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.angle.to_string().as_bytes())?;
        out.write_all(b")\n")?;
        self.effects.write(out, indent + 1)?;
        out.write_all(b"\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for Polyline {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(polyline ")?;
        out.write_all(b"(pts")?;
        for p in self.pts.rows().into_iter() {
            out.write_all(b" (xy ")?;
            out.write_all(p.get(0).unwrap().to_string().as_bytes())?;
            out.write_all(b" ")?;
            out.write_all(p.get(1).unwrap().to_string().as_bytes())?;
            out.write_all(b")")?;
        }
        out.write_all(b")\n")?;
        self.stroke.write(out, indent + 1)?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        if let Some(uuid) = &self.uuid {
            out.write_all(b"(uuid ")?;
            out.write_all(uuid.as_bytes())?;
            out.write_all(b")\n")?;
        }
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for LibrarySymbol {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(symbol \"")?;
        out.write_all(self.lib_id.as_bytes())?;
        out.write_all(b"\" ")?;
        if self.power {
            out.write_all(b"(power) ")?;
        }
        if !self.pin_numbers_show {
            out.write_all(b"(pin_numbers hide) ")?;
        }
        if self.pin_names_offset != -1.0 || !self.pin_names_show {
            out.write_all(b"(pin_names")?;
            if self.pin_names_offset != -1.0 {
                out.write_all(b" (offset ")?;
                out.write_all(self.pin_names_offset.to_string().as_bytes())?;
                out.write_all(b")")?;
            }
            if !self.pin_names_show {
                out.write_all(b" hide")?;
            }
            out.write_all(b") ")?;
        }
        out.write_all(b"(in_bom ")?;
        out.write_all(if self.in_bom { b"yes" } else { b"no" })?;
        out.write_all(b") (on_board ")?;
        out.write_all(if self.on_board { b"yes" } else { b"no" })?;
        out.write_all(b")\n")?;
        for p in &self.property {
            p.write(out, indent + 1)?;
        }
        for s in &self.symbols {
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b"(symbol \"")?;
            out.write_all(s.lib_id.as_bytes())?;
            out.write_all(b"\"\n")?;
            for graph in &s.graph {
                match graph {
                    super::model::Graph::Polyline(polyline) => {
                        out.write_all("  ".repeat(indent + 2).as_bytes())?;
                        out.write_all(b"(polyline\n")?;
                        out.write_all("  ".repeat(indent + 3).as_bytes())?;
                        out.write_all(b"(pts\n")?;
                        for p in polyline.pts.rows().into_iter() {
                            out.write_all("  ".repeat(indent + 4).as_bytes())?;
                            out.write_all(b"(xy ")?;
                            out.write_all(p.get(0).unwrap().to_string().as_bytes())?;
                            out.write_all(b" ")?;
                            out.write_all(p.get(1).unwrap().to_string().as_bytes())?;
                            out.write_all(b")\n")?;
                        }
                        out.write_all("  ".repeat(indent + 3).as_bytes())?;
                        out.write_all(b")\n")?;
                        polyline.stroke.write(out, indent + 3)?;
                        out.write_all("  ".repeat(indent + 3).as_bytes())?;
                        out.write_all(b"(fill (type ")?;
                        out.write_all(polyline.fill_type.as_bytes())?;
                        out.write_all(b"))\n")?;

                        out.write_all("  ".repeat(indent + 2).as_bytes())?;
                        out.write_all(b")\n")?;
                    }
                    super::model::Graph::Rectangle(rectangle) => {
                        out.write_all("  ".repeat(indent + 2).as_bytes())?;
                        out.write_all(b"(rectangle (start ")?;
                        out.write_all(rectangle.start.get(0).unwrap().to_string().as_bytes())?;
                        out.write_all(b" ")?;
                        out.write_all(rectangle.start.get(1).unwrap().to_string().as_bytes())?;
                        out.write_all(b") (end ")?;
                        out.write_all(rectangle.end.get(0).unwrap().to_string().as_bytes())?;
                        out.write_all(b" ")?;
                        out.write_all(rectangle.end.get(1).unwrap().to_string().as_bytes())?;
                        out.write_all(b")\n")?;
                        rectangle.stroke.write(out, indent + 3)?;
                        out.write_all("  ".repeat(indent + 3).as_bytes())?;
                        out.write_all(b"(fill (type ")?;
                        out.write_all(rectangle.fill_type.as_bytes())?;
                        out.write_all(b"))\n")?;
                        out.write_all("  ".repeat(indent + 2).as_bytes())?;
                        out.write_all(b")\n")?;
                    }
                    super::model::Graph::Circle(circle) => {
                        out.write_all("  ".repeat(indent + 2).as_bytes())?;
                        out.write_all(b"(circle (center ")?;
                        out.write_all(circle.center.get(0).unwrap().to_string().as_bytes())?;
                        out.write_all(b" ")?;
                        out.write_all(circle.center.get(1).unwrap().to_string().as_bytes())?;
                        out.write_all(b") (radius ")?;
                        out.write_all(circle.radius.to_string().as_bytes())?;
                        out.write_all(b")\n")?;
                        circle.stroke.write(out, indent + 3)?;
                        out.write_all("  ".repeat(indent + 3).as_bytes())?;
                        out.write_all(b"(fill (type ")?;
                        out.write_all(circle.fill_type.as_bytes())?;
                        out.write_all(b"))\n")?;
                        out.write_all("  ".repeat(indent + 2).as_bytes())?;
                        out.write_all(b")\n")?;
                    }
                    super::model::Graph::Arc(arc) => {
                        out.write_all("  ".repeat(indent + 2).as_bytes())?;
                        out.write_all(b"(arc (start ")?;
                        out.write_all(arc.start.get(0).unwrap().to_string().as_bytes())?;
                        out.write_all(b" ")?;
                        out.write_all(arc.start.get(1).unwrap().to_string().as_bytes())?;
                        out.write_all(b") (mid ")?;
                        out.write_all(arc.mid.get(0).unwrap().to_string().as_bytes())?;
                        out.write_all(b" ")?;
                        out.write_all(arc.mid.get(1).unwrap().to_string().as_bytes())?;
                        out.write_all(b") (end ")?;
                        out.write_all(arc.end.get(0).unwrap().to_string().as_bytes())?;
                        out.write_all(b" ")?;
                        out.write_all(arc.end.get(1).unwrap().to_string().as_bytes())?;
                        out.write_all(b")\n")?;
                        arc.stroke.write(out, indent + 3)?;
                        out.write_all("  ".repeat(indent + 3).as_bytes())?;
                        out.write_all(b"(fill (type ")?;
                        out.write_all(arc.fill_type.as_bytes())?;
                        out.write_all(b"))\n")?;
                        out.write_all("  ".repeat(indent + 2).as_bytes())?;
                        out.write_all(b")\n")?;
                    }
                    super::model::Graph::Text(text) => {
                        out.write_all("  ".repeat(indent + 2).as_bytes())?;
                        out.write_all(b"(text \"")?;
                        out.write_all(text.text.as_bytes())?;
                        out.write_all(b"\" (at )")?;

                        out.write_all(text.at.get(0).unwrap().to_string().as_bytes())?;
                        out.write_all(b" ")?;
                        out.write_all(text.at.get(1).unwrap().to_string().as_bytes())?;
                        out.write_all(b" ")?;
                        out.write_all(text.angle.to_string().as_bytes())?;
                        out.write_all(b")\n")?;
                        text.effects.write(out, indent + 3)?;
                        out.write_all(b"\n")?;
                        out.write_all("  ".repeat(indent + 2).as_bytes())?;
                        out.write_all(b")\n")?;
                    }
                }
            }
            for pin in &s.pin {
                out.write_all("  ".repeat(indent + 2).as_bytes())?;
                out.write_all(b"(pin ")?;
                out.write_all(pin.pin_type.as_bytes())?;
                out.write_all(b" ")?;
                out.write_all(pin.pin_graphic_style.as_bytes())?;
                out.write_all(b" (at ")?;
                out.write_all(pin.at.get(0).unwrap().to_string().as_bytes())?;
                out.write_all(b" ")?;
                out.write_all(pin.at.get(1).unwrap().to_string().as_bytes())?;
                out.write_all(b" ")?;
                out.write_all(pin.angle.to_string().as_bytes())?;
                out.write_all(b") (length ")?;
                out.write_all(pin.length.to_string().as_bytes())?;
                out.write_all(b")")?;
                if pin.hide {
                    out.write_all(b" hide")?;
                }
                out.write_all(b"\n")?;
                out.write_all("  ".repeat(indent + 3).as_bytes())?;
                out.write_all(b"(name \"")?;
                out.write_all(pin.name.0.as_bytes())?;
                out.write_all(b"\" ")?;
                pin.name.1.write(out, 0)?;
                out.write_all(b")\n")?;
                out.write_all("  ".repeat(indent + 3).as_bytes())?;
                out.write_all(b"(number \"")?;
                out.write_all(pin.number.0.as_bytes())?;
                out.write_all(b"\" ")?;
                pin.name.1.write(out, 0)?;
                out.write_all(b")\n")?;
                out.write_all("  ".repeat(indent + 2).as_bytes())?;
                out.write_all(b")\n")?;
            }
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b")\n")?;
        }
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for Sheet {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(sheet ")?;
        out.write_all(b"(at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (size ")?;
        out.write_all(self.size.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.size.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b")")?;
        if self.fields_autoplaced {
            out.write_all(b" (fields_autoplaced)")?;
        }
        out.write_all(b"\n")?;
        self.stroke.write(out, indent + 1)?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(fill ")?;
        out.write_all(b"(color ")?;
        out.write_all(self.fill.0.to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.fill.1.to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.fill.2.to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.fill.2.to_string().as_bytes())?;
        out.write_all(b"))\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n")?;
        for property in &self.property {
            property.write(out, indent + 1)?;
        }
        for pin in &self.pin {
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b"(pin \"")?;
            out.write_all(pin.pin_type.as_bytes())?;
            out.write_all(b"\" ")?;
            out.write_all(pin.pin_graphic_style.as_bytes())?;
            out.write_all(b" (at ")?;
            out.write_all(pin.at.get(0).unwrap().to_string().as_bytes())?;
            out.write_all(b" ")?;
            out.write_all(pin.at.get(1).unwrap().to_string().as_bytes())?;
            out.write_all(b" ")?;
            out.write_all(pin.angle.to_string().as_bytes())?;
            out.write_all(b")\n")?;
            out.write_all("  ".repeat(indent + 2).as_bytes())?;
            pin.name.1.write(out, 0)?;
            out.write_all(b"\n")?;
            out.write_all("  ".repeat(indent + 2).as_bytes())?;
            out.write_all(b"(uuid ")?;
            out.write_all(pin.uuid.as_bytes())?;
            out.write_all(b")\n")?;
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b")\n")?;
        }
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for Symbol {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(symbol (lib_id \"")?;
        out.write_all(self.lib_id.as_bytes())?;
        out.write_all(b"\") (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.angle.to_string().as_bytes())?;
        if !self.mirror.is_empty() {
            out.write_all(b") (mirror ")?;
            out.write_all(self.mirror.join(" ").as_bytes())?;
        }
        out.write_all(b") (unit ")?;
        out.write_all(self.unit.to_string().as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(in_bom ")?;
        out.write_all(if self.in_bom { b"yes" } else { b"no" })?;
        out.write_all(b") (on_board ")?;
        out.write_all(if self.on_board { b"yes" } else { b"no" })?;
        out.write_all(b")")?;
        if self.fields_autoplaced {
            out.write_all(b" (fields_autoplaced)")?;
        }
        out.write_all(b"\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(uuid ")?;
        out.write_all(self.uuid.as_bytes())?;
        out.write_all(b")\n")?;
        for p in &self.property {
            p.write(out, indent + 1)?;
        }
        for p in &self.pin {
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b"(pin \"")?;
            out.write_all(p.0.as_bytes())?;
            out.write_all(b"\" (uuid ")?;
            out.write_all(p.1.as_bytes())?;
            out.write_all(b"))\n")?;
        }
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for SheetInstance {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(path \"")?;
        out.write_all(self.path.as_bytes())?;
        out.write_all(b"\" (page \"")?;
        out.write_all(self.page.to_string().as_bytes())?;
        out.write_all(b"\"))\n")?;
        Ok(())
    }
}
impl SexpWriter for SymbolInstance {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(path \"")?;
        out.write_all(self.path.as_bytes())?;
        out.write_all(b"\"\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(reference \"")?;
        out.write_all(self.reference.as_bytes())?;
        out.write_all(b"\") (unit ")?;
        out.write_all(self.unit.to_string().as_bytes())?;
        out.write_all(b") (value \"")?;
        out.write_all(self.value.as_bytes())?;
        out.write_all(b"\") (footprint \"")?;
        out.write_all(self.footprint.as_bytes())?;
        out.write_all(b"\")\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for Layers {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(")?;
        out.write_all(self.ordinal.to_string().as_bytes())?;
        out.write_all(b" \"")?;
        out.write_all(self.canonical_name.as_bytes())?;
        out.write_all(b"\" ")?;
        out.write_all(self.layertype.as_bytes())?;
        if let Some(user_name) = &self.user_name {
            out.write_all(b" \"")?;
            out.write_all(user_name.as_bytes())?;
            out.write_all(b"\"")?;
        }
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for Footprint {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(footprint \"")?;
        out.write_all(self.key.as_bytes())?;
        out.write_all(b"\"")?;
        if self.locked {
            out.write_all(b" locked")?;
        }
        out.write_all(b" (layer \"")?;
        out.write_all(self.layer.as_bytes())?;
        out.write_all(b"\")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(tedit ")?;
        out.write_all(self.tedit.to_string().as_bytes())?;
        out.write_all(b") (tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        if self.angle != 0.0 {
            out.write_all(b" ")?;
            out.write_all(self.angle.to_string().as_bytes())?;
        }
        out.write_all(b")\n")?;
        if !self.descr.is_empty() {
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b"(descr \"")?;
            out.write_all(self.descr.as_bytes())?;
            out.write_all(b"\")\n")?;
        }
        if !self.tags.is_empty() {
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b"(tags \"")?;
            out.write_all(self.tags.join(" ").as_bytes())?;
            out.write_all(b"\")\n")?;
        }
        if !self.path.is_empty() {
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b"(path \"")?;
            out.write_all(self.path.as_bytes())?;
            out.write_all(b"\")\n")?;
        }
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(attr ")?;
        out.write_all(self.attr.join(" ").as_bytes())?;
        out.write_all(b")\n")?;
        for graph in &self.graphics {
            match graph {
                super::model::Graphics::FpText(text) => text.write(out, indent + 1)?,
                super::model::Graphics::FpLine(line) => line.write(out, indent + 1)?,
                super::model::Graphics::FpCircle(circle) => circle.write(out, indent + 1)?,
                super::model::Graphics::FpArc(arc) => arc.write(out, indent + 1)?,
            }
        }
        for pad in &self.pads {
            pad.write(out, indent + 1)?;
        }
        for pad in &self.models {
            pad.write(out, indent + 1)?;
        }
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for FpText {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(fp_text ")?;
        out.write_all(self.key.as_bytes())?;
        out.write_all(b" \"")?;
        out.write_all(self.value.as_bytes())?;
        out.write_all(b"\" (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        if self.angle != -1.0 {
            out.write_all(b" ")?;
            out.write_all(self.angle.to_string().as_bytes())?;
        }
        out.write_all(b") (layer \"")?;
        out.write_all(self.layer.as_bytes())?;
        out.write_all(b"\")")?;
        if self.hidden {
            out.write_all(b" hide")?;
        }
        out.write_all(b"\n")?;
        self.effects.write(out, indent + 1)?;
        out.write_all(b"\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for FpLine {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(fp_line (start ")?;
        out.write_all(self.start.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.start.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (end ")?;
        out.write_all(self.end.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.end.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (layer \"")?;
        out.write_all(self.layer.as_bytes())?;
        out.write_all(b"\") (width ")?;
        out.write_all(self.width.to_string().as_bytes())?;
        out.write_all(b") (tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b"))\n")?;
        Ok(())
    }
}
impl SexpWriter for FpArc {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(fp_circle (start ")?;
        out.write_all(self.start.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.start.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (mid ")?;
        out.write_all(self.mid.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.mid.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (end ")?;
        out.write_all(self.end.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.end.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (layer \"")?;
        out.write_all(self.layer.as_bytes())?;
        out.write_all(b"\") (width ")?;
        out.write_all(self.width.to_string().as_bytes())?;
        out.write_all(b")")?;
        if !self.fill.is_empty() {
            out.write_all(b" (fill ")?;
            out.write_all(self.fill.as_bytes())?;
            out.write_all(b")")?;
        }
        out.write_all(b" (tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b"))\n")?;
        Ok(())
    }
}
impl SexpWriter for FpCircle {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(fp_circle (center ")?;
        out.write_all(self.center.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.center.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (end ")?;
        out.write_all(self.end.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.end.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (layer \"")?;
        out.write_all(self.layer.as_bytes())?;
        out.write_all(b"\") (width ")?;
        out.write_all(self.width.to_string().as_bytes())?;
        out.write_all(b")")?;
        if !self.fill.is_empty() {
            out.write_all(b" (fill ")?;
            out.write_all(self.fill.as_bytes())?;
            out.write_all(b")")?;
        }
        out.write_all(b" (tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b"))\n")?;
        Ok(())
    }
}
impl SexpWriter for Via {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(via ")?;
        out.write_all(b"(at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") ")?;
        out.write_all(b"(size ")?;
        out.write_all(self.size.to_string().as_bytes())?;
        out.write_all(b") (drill ")?;
        out.write_all(self.drill.to_string().as_bytes())?;
        out.write_all(b") (layers ")?;
        out.write_all(self.layers.join(" ").as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b" (net ")?;
        out.write_all(self.net.as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b" (tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b"))\n")?;
        Ok(())
    }
}
impl SexpWriter for Pad {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(pad \"")?;
        out.write_all(self.number.as_bytes())?;
        out.write_all(b"\" ")?;
        out.write_all(self.padtype.as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.padshape.as_bytes())?;
        out.write_all(b" ")?;
        if self.locked {
            out.write_all(b"locked ")?;
        }
        out.write_all(b"(at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        if self.angle != 0.0 {
            out.write_all(b" ")?;
            out.write_all(self.angle.to_string().as_bytes())?;
        }
        out.write_all(b") ")?;
        out.write_all(b"(size ")?;
        out.write_all(self.size.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.size.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b") (drill ")?;
        if self.oval {
            out.write_all(b"oval ")?;
        }
        out.write_all(self.drill.to_string().as_bytes())?;
        out.write_all(b") (layers ")?;
        out.write_all(self.layers.join(" ").as_bytes())?;
        out.write_all(b")")?;
        if self.rratio != 0.0 {
            out.write_all(b" (roundrect_rratio ")?;
            out.write_all(self.rratio.to_string().as_bytes())?;
            out.write_all(b")")?;
        }
        if let Some(net) = &self.net {
            out.write_all(b"\n")?;
            out.write_all("  ".repeat(indent + 1).as_bytes())?;
            out.write_all(b"(net ")?;
            out.write_all(net.number.to_string().as_bytes())?;
            out.write_all(b" \"")?;
            out.write_all(net.name.as_bytes())?;
            out.write_all(b"\")")?;
        }
        out.write_all(b" (tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b"))\n")?;
        Ok(())
    }
}
impl SexpWriter for Model {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(model \"")?;
        out.write_all(self.path.as_bytes())?;
        out.write_all(b"\"\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(offset (xyz ")?;
        out.write_all(self.offset.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.offset.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.offset.get(2).unwrap().to_string().as_bytes())?;
        out.write_all(b"))\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(scale (xyz ")?;
        out.write_all(self.scale.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.scale.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.scale.get(2).unwrap().to_string().as_bytes())?;
        out.write_all(b"))\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(rotate (xyz ")?;
        out.write_all(self.rotate.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.rotate.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.rotate.get(2).unwrap().to_string().as_bytes())?;
        out.write_all(b"))\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for Segment {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(segment ")?;
        out.write_all(b"(start ")?;
        out.write_all(self.start.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.start.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b" (end ")?;
        out.write_all(self.end.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.end.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b" (width ")?;
        out.write_all(self.width.to_string().as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b" (layer \"")?;
        out.write_all(self.layer.as_bytes())?;
        out.write_all(b"\")")?;
        out.write_all(b" (net ")?;
        out.write_all(self.net.to_string().as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b" (tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b"))\n")?;
        Ok(())
    }
}
impl SexpWriter for GrLine {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(gr_line ")?;
        out.write_all(b"(start ")?;
        out.write_all(self.start.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.start.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b" (end ")?;
        out.write_all(self.end.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.end.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b" (layer \"")?;
        out.write_all(self.layer.as_bytes())?;
        out.write_all(b"\")")?;
        out.write_all(b" (width ")?;
        out.write_all(self.width.to_string().as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b" (tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b"))\n")?;
        Ok(())
    }
}
impl SexpWriter for GrText {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(gr_text \"")?;
        out.write_all(self.text.as_bytes())?;
        out.write_all(b"\" (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.angle.to_string().as_bytes())?;
        out.write_all(b")")?;
        out.write_all(b" (layer \"")?;
        out.write_all(self.layer.as_bytes())?;
        out.write_all(b"\")")?;
        out.write_all(b" (tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b")\n")?;
        self.effects.write(out, indent + 1)?;
        out.write_all(b"\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
impl SexpWriter for Zone {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(zone (net ")?;
        out.write_all(self.net.to_string().as_bytes())?;
        out.write_all(b") (net_name \"")?;
        out.write_all(self.net_name.as_bytes())?;
        out.write_all(b"\") (layer \"")?;
        out.write_all(self.layer.as_bytes())?;
        out.write_all(b"\") (tstamp ")?;
        out.write_all(self.tstamp.as_bytes())?;
        out.write_all(b") (hatch edge ")?;
        out.write_all(self.hatch_edge.to_string().as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(connect_pads (clearance ")?;
        out.write_all(self.pad_clearance.to_string().as_bytes())?;
        out.write_all(b"))\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(min_thickness ")?;
        out.write_all(self.min_thickness.to_string().as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(fill ")?;
        if self.filled {
            out.write_all(b"yes")?;
        } else {
            out.write_all(b"no")?;
        }
        out.write_all(b" (thermal_gap ")?;
        out.write_all(self.fill_thermal_gap.to_string().as_bytes())?;
        out.write_all(b") (thermal_bridge_width ")?;
        out.write_all(self.fill_thermal_bridge.to_string().as_bytes())?;
        out.write_all(b"))\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(polygon\n")?;
        out.write_all("  ".repeat(indent + 2).as_bytes())?;
        out.write_all(b"(pts\n")?;
        for xy in self.polygon.rows() {
            out.write_all("  ".repeat(indent + 3).as_bytes())?;
            out.write_all(b"(xy ")?;
            out.write_all(xy[0].to_string().as_bytes())?;
            out.write_all(b" ")?;
            out.write_all(xy[1].to_string().as_bytes())?;
            out.write_all(b")\n")?;
        }
        out.write_all("  ".repeat(indent + 2).as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b")\n")?;

        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(filled_polygon\n")?;
        out.write_all("  ".repeat(indent + 2).as_bytes())?;
        out.write_all(b"(layer \"")?;
        out.write_all(self.layer.as_bytes())?;
        out.write_all(b"\")\n")?;
        out.write_all("  ".repeat(indent + 2).as_bytes())?;
        out.write_all(b"(pts\n")?;
        for xy in self.filled_polygon.1.rows() {
            out.write_all("  ".repeat(indent + 3).as_bytes())?;
            out.write_all(b"(xy ")?;
            out.write_all(xy[0].to_string().as_bytes())?;
            out.write_all(b" ")?;
            out.write_all(xy[1].to_string().as_bytes())?;
            out.write_all(b")\n")?;
        }
        out.write_all("  ".repeat(indent + 2).as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
        Ok(())
    }
}
