use std::io::Write;

use crate::error::Error;

use super::model::{
    Effects, GlobalLabel, Junction, Label, LibrarySymbol, NoConnect, Property, Sheet,
    SheetInstance, Stroke, Symbol, SymbolInstance, Text, TitleBlock, Wire,
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
        out.write_all(b"))")?;
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
        out.write_all(b")\n")?;
        self.effects.write(out, indent + 1)?;
        out.write_all(b"\n")?;
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b")\n")?;
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
impl SexpWriter for Text {
    fn write(&self, out: &mut dyn Write, indent: usize) -> Result<(), Error> {
        out.write_all("  ".repeat(indent).as_bytes())?;
        out.write_all(b"(text \"")?;
        out.write_all(self.text.as_bytes())?;
        out.write_all(b"\" (at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
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
        out.write_all(b"(pin_names")?;
        // if self.pin_names_offset != 0.0 {
        out.write_all(b" (offset ")?;
        out.write_all(self.pin_names_offset.to_string().as_bytes())?;
        out.write_all(b")")?;
        // }
        if !self.pin_names_show {
            out.write_all(b" hide")?;
        }
        out.write_all(b") ")?;
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
        out.write_all(b"(symbol (sheet ")?;
        out.write_all(b"(at ")?;
        out.write_all(self.at.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.at.get(1).unwrap().to_string().as_bytes())?;
        out.write_all(b"\") (size ")?;
        out.write_all(self.size.get(0).unwrap().to_string().as_bytes())?;
        out.write_all(b" ")?;
        out.write_all(self.size.get(1).unwrap().to_string().as_bytes())?;
        if self.fields_autoplaced {
            out.write_all(b" (fields_autoplaced)")?;
        }
        out.write_all(b"\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(fill ")?;
        out.write_all(b" (color ")?;
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
            out.write_all(b")\n")?;
            out.write_all("  ".repeat(indent + 3).as_bytes())?;
            out.write_all(b"(name \"")?;
            out.write_all(pin.name.0.as_bytes())?;
            out.write_all(b"\" ")?;
            pin.name.1.write(out, 0)?;
            out.write_all("  ".repeat(indent + 3).as_bytes())?;
            out.write_all(b")\n")?;
            out.write_all("  ".repeat(indent + 3).as_bytes())?;
            out.write_all(b"(number \"")?;
            out.write_all(pin.number.0.as_bytes())?;
            out.write_all(b"\" ")?;
            pin.name.1.write(out, 0)?;
            out.write_all("  ".repeat(indent + 3).as_bytes())?;
            out.write_all(b")\n")?;
            out.write_all("  ".repeat(indent + 2).as_bytes())?;
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
        out.write_all(b") (unit ")?;
        out.write_all(self.unit.to_string().as_bytes())?;
        out.write_all(b")\n")?;
        out.write_all("  ".repeat(indent + 1).as_bytes())?;
        out.write_all(b"(in_bom ")?;
        out.write_all(if self.in_bom { b"yes" } else { b"no" })?;
        out.write_all(b") (on_board ")?;
        out.write_all(if self.on_board { b"yes" } else { b"no" })?;
        out.write_all(b")\n")?;
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

#[cfg(test)]
mod tests {
    use crate::sexp::{Schema, SexpParser};

    #[test]
    fn test_write() {
        /* let doc = SexpParser::load("samples/files/summe/summe.kicad_sch").unwrap();
        write(&mut std::io::stdout(), doc.iter().node()).unwrap(); */
    }
}
