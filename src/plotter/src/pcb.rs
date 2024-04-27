//! Plot the PCB
use crate::{error::Error, schema::Themer, Theme};
use log::{debug, error, warn};
use pyo3::{prelude::*, types::PyDict};
use rand::Rng;
use std::fs;
use svg::{
    node::element::{Circle, Description, Group, Path, Symbol, Text, Title, Use},
    parser::Event,
    Document,
};

pub const LAYERS: &[&str; 9] = &[
    "F_Cu",
    "B_Cu",
    "F_Paste",
    "B_Paste",
    "F_SilkS",
    "B_SilkS",
    "F_Mask",
    "B_Mask",
    "Edge_Cuts",
];

macro_rules! end {
    ( $group:expr, $stack:expr ) => {
        if let Some(item) = $stack.end() {
            match item {
                SvgTypes::Group(g) => $group = $group.add(g),
                SvgTypes::Path(p) => $group = $group.add(p),
                SvgTypes::Circle(c) => $group = $group.add(c),
                SvgTypes::Text(t) => $group = $group.add(t),
                SvgTypes::Desc(d) => $group = $group.add(d),
                SvgTypes::Title(d) => $group = $group.add(d),
            }
        }
    };
}

macro_rules! arm {
    ( $items:expr, $token:expr, $last:expr, $target_type:expr ) => {
        match $token {
            SvgTypes::Group(child_g) => {
                $last = $last.add(child_g);
                $items.push($target_type($last));
            }
            SvgTypes::Path(child_path) => {
                $last = $last.add(child_path);
                $items.push($target_type($last));
            }
            SvgTypes::Circle(child_circle) => {
                $last = $last.add(child_circle);
                $items.push($target_type($last));
            }
            SvgTypes::Text(child_circle) => {
                $last = $last.add(child_circle);
                $items.push($target_type($last));
            }
            SvgTypes::Desc(child_circle) => {
                $last = $last.add(child_circle);
                $items.push($target_type($last));
            }
            SvgTypes::Title(child) => {
                $last = $last.add(child);
                $items.push($target_type($last));
            }
        }
    };
}

fn check_directory(filename: &str) -> Result<(), Error> {
    let path = std::path::Path::new(filename);
    if path.to_str().unwrap() != "" && !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

fn clean_style(input: &str, style: &super::Style, themer: &Themer) -> Result<String, Error> {
    let mut res = String::new();
    for token in input.split(';') {
        let colon = token.rfind(':');
        if let Some(colon) = colon {
            let key = &token[0..colon].trim();
            let value = &token[colon + 1..token.len()].trim();
            if key == &"stroke" && value != &"#FFFFFF" && value != &"#000000" {
                res += format!(
                    "stroke:{}; ",
                    themer.hex_color(themer.stroke(&vec![style.clone()]))
                )
                .as_str();
            } else if key == &"fill" && value != &"#FFFFFF" && value != &"#000000" {
                res += format!(
                    "fill:{}; ",
                    themer.hex_color(themer.stroke(&vec![style.clone()]))
                )
                .as_str();
            } else {
                res += format!("{}:{}; ", key, value).as_str();
            }
        }
    }
    Ok(res)
}

enum SvgTypes {
    Group(Group),
    Path(Path),
    Circle(Circle),
    Text(Text),
    Desc(Description),
    Title(Title),
}

struct SvgStack {
    items: Vec<SvgTypes>,
}

impl SvgStack {
    fn new() -> Self {
        Self { items: Vec::new() }
    }
    ///End a Node, pop it from the stack and add it to the prior one.
    fn end(&mut self) -> Option<SvgTypes> {
        if self.items.is_empty() {
            error!("empty stack");
            None
        } else if self.items.len() == 1 {
            self.items.pop()
        } else {
            let g = self.items.pop().unwrap();
            let last = self.items.pop().unwrap();
            match last {
                SvgTypes::Group(mut last_g) => {
                    arm!(self.items, g, last_g, SvgTypes::Group);
                }
                SvgTypes::Path(mut last_path) => {
                    arm!(self.items, g, last_path, SvgTypes::Path);
                }
                SvgTypes::Circle(mut last_c) => {
                    arm!(self.items, g, last_c, SvgTypes::Circle);
                }
                SvgTypes::Text(mut last_c) => {
                    arm!(self.items, g, last_c, SvgTypes::Text);
                }
                SvgTypes::Desc(mut last_c) => {
                    arm!(self.items, g, last_c, SvgTypes::Desc);
                }
                SvgTypes::Title(mut last) => {
                    arm!(self.items, g, last, SvgTypes::Title);
                }
            }
            None
        }
    }
}

trait SvgType<E> {
    ///Start a new Node, add it to the stack.
    fn start(&mut self, item: E);
}

impl SvgType<Group> for SvgStack {
    fn start(&mut self, item: Group) {
        self.items.push(SvgTypes::Group(item));
    }
}

impl SvgType<Path> for SvgStack {
    fn start(&mut self, item: Path) {
        self.items.push(SvgTypes::Path(item));
    }
}

impl SvgType<Circle> for SvgStack {
    fn start(&mut self, item: Circle) {
        self.items.push(SvgTypes::Circle(item));
    }
}

impl SvgType<Text> for SvgStack {
    fn start(&mut self, item: Text) {
        self.items.push(SvgTypes::Text(item));
    }
}

impl SvgType<Description> for SvgStack {
    fn start(&mut self, item: Description) {
        self.items.push(SvgTypes::Desc(item));
    }
}

impl SvgType<Title> for SvgStack {
    fn start(&mut self, item: Title) {
        self.items.push(SvgTypes::Title(item));
    }
}

impl SvgType<String> for SvgStack {
    fn start(&mut self, item: String) {
        if !self.items.is_empty() {
            let text = self.items.pop().unwrap();
            match text {
                SvgTypes::Group(mut text) => {
                    text = text.add(svg::node::Text::new(item));
                    self.items.push(SvgTypes::Group(text));
                }
                SvgTypes::Path(mut text) => {
                    text = text.add(svg::node::Text::new(item));
                    self.items.push(SvgTypes::Path(text));
                }
                SvgTypes::Circle(mut text) => {
                    text = text.add(svg::node::Text::new(item));
                    self.items.push(SvgTypes::Circle(text));
                }
                SvgTypes::Text(mut text) => {
                    text = text.add(svg::node::Text::new(item));
                    self.items.push(SvgTypes::Text(text));
                }
                SvgTypes::Desc(mut text) => {
                    text = text.add(svg::node::Text::new(item));
                    self.items.push(SvgTypes::Desc(text));
                }
                SvgTypes::Title(mut text) => {
                    text = text.add(svg::node::Text::new(item));
                    self.items.push(SvgTypes::Title(text));
                }
            }
        } else {
            warn!("start text: items is empty ({})", item);
        }
    }
}

pub fn plot_pcb(
    input: String,
    output: String,
    layers: Option<&Vec<String>>,
    theme: Option<Theme>,
) -> Result<(f64, f64), Error> {
    let themer = Themer::new(theme.unwrap_or_default());

    let layers = if let Some(layers) = layers {
        layers.clone()
    } else {
        LAYERS
            .to_vec()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>()
    };

    //prepare temp directory
    let mut rng = rand::thread_rng();
    let num: u32 = rng.gen();
    let tmp_folder = num.to_string();
    let last_slash = input.rfind('/').unwrap();
    let basedir = &input[0..last_slash];
    let last_dot = input.rfind('.').unwrap();
    let name = &input[last_slash + 1..last_dot];

    let mut document = Document::new();

    let mut width = 0.0;
    let mut height = 0.0;

    for layer in &layers {
        let style = crate::Style::from(layer.to_string());
        Python::with_gil(|py| {
            // let list = PyList::new(py, &[input.clone(), tmp_folder.clone(), layers]);
            let locals = PyDict::new_bound(py);
            locals.set_item("input", input.clone()).unwrap();
            locals.set_item("tmp_folder", tmp_folder.clone()).unwrap();
            locals.set_item("layer", layer).unwrap();
            pyo3::py_run!(
                py,
                *locals,
                r#"
    import pcbnew 
    layer_names = { 
        "B_Cu": pcbnew.B_Cu,
        "F_Cu": pcbnew.F_Cu,
        "In1_Cu": pcbnew.In1_Cu,
        "In2_Cu": pcbnew.In2_Cu,
        "In3_Cu": pcbnew.In3_Cu,
        "In4_Cu": pcbnew.In4_Cu,
        "In5_Cu": pcbnew.In5_Cu,
        "In6_Cu": pcbnew.In6_Cu,
        "In7_Cu": pcbnew.In7_Cu,
        "In8_Cu": pcbnew.In8_Cu,
        "In9_Cu": pcbnew.In9_Cu,
        "In10_Cu": pcbnew.In10_Cu,
        "In11_Cu": pcbnew.In11_Cu,
        "In12_Cu": pcbnew.In12_Cu,
        "In13_Cu": pcbnew.In13_Cu,
        "In14_Cu": pcbnew.In14_Cu,
        "In15_Cu": pcbnew.In15_Cu,
        "In16_Cu": pcbnew.In16_Cu,
        "In17_Cu": pcbnew.In17_Cu,
        "In18_Cu": pcbnew.In18_Cu,
        "In19_Cu": pcbnew.In19_Cu,
        "In20_Cu": pcbnew.In20_Cu,
        "In21_Cu": pcbnew.In21_Cu,
        "In22_Cu": pcbnew.In22_Cu,
        "In23_Cu": pcbnew.In23_Cu,
        "In24_Cu": pcbnew.In24_Cu,
        "In25_Cu": pcbnew.In25_Cu,
        "In26_Cu": pcbnew.In26_Cu,
        "In27_Cu": pcbnew.In27_Cu,
        "In28_Cu": pcbnew.In28_Cu,
        "In29_Cu": pcbnew.In29_Cu,
        "In30_Cu": pcbnew.In30_Cu,
        "B_Cu": pcbnew.B_Cu,
        "B_Adhes": pcbnew.B_Adhes,
        "F_Adhes": pcbnew.F_Adhes,
        "B_Paste": pcbnew.B_Paste,
        "F_Paste": pcbnew.F_Paste,
        "B_SilkS": pcbnew.B_SilkS,
        "F_SilkS": pcbnew.F_SilkS,
        "B_Mask": pcbnew.B_Mask,
        "F_Mask": pcbnew.F_Mask,
        "Dwgs_User": pcbnew.Dwgs_User,
        "Cmts_User": pcbnew.Cmts_User,
        "Eco1_User": pcbnew.Eco1_User,
        "Eco2_User": pcbnew.Eco2_User,
        "Edge_Cuts": pcbnew.Edge_Cuts,
        "Margin": pcbnew.Margin,
        "B_CrtYd": pcbnew.B_CrtYd,
        "F_CrtYd": pcbnew.F_CrtYd,
        "B_Fab": pcbnew.B_Fab,
        "F_Fab": pcbnew.F_Fab,
        "User_1": pcbnew.User_1,
        "User_2": pcbnew.User_2,
        "User_3": pcbnew.User_3,
        "User_4": pcbnew.User_4,
        "User_5": pcbnew.User_5,
        "User_6": pcbnew.User_6,
        "User_7": pcbnew.User_7,
        "User_8": pcbnew.User_8,
        "User_9": pcbnew.User_9,
    }

    print(f"Plot: {layer}")
    board = pcbnew.LoadBoard(input)    
    plot_controller = pcbnew.PLOT_CONTROLLER(board)
    plot_options = plot_controller.GetPlotOptions()    

    pcb_bounding_box = board.ComputeBoundingBox(True) 
    print("origin", pcb_bounding_box.GetOrigin()) 
    print("height", pcb_bounding_box.GetHeight()) 
    print("width", pcb_bounding_box.GetWidth())
    plot_options.SetUseAuxOrigin(True) 
    board.GetDesignSettings().SetAuxOrigin(pcb_bounding_box.GetOrigin()) 

    settings_manager = pcbnew.GetSettingsManager() 
    color_settings = settings_manager.GetColorSettings() 
    plot_options.SetColorSettings(color_settings) 

    plot_options.SetOutputDirectory(tmp_folder)
    plot_options.SetPlotFrameRef(False)
    #plot_options.SetDrillMarksType(pcbnew.PCB_PLOT_PARAMS.FULL_DRILL_SHAPE)
    plot_options.SetSkipPlotNPTH_Pads(False)
    plot_options.SetMirror(False)
    plot_options.SetFormat(pcbnew.PLOT_FORMAT_SVG)
    plot_options.SetSvgPrecision(4)
    plot_options.SetPlotViaOnMaskLayer(True)    
    plot_controller.OpenPlotfile(layer, pcbnew.PLOT_FORMAT_SVG, "Top mask layer")
    plot_controller.SetColorMode(True)
    plot_controller.SetLayer(layer_names[layer])
    plot_controller.PlotLayer()
    plot_controller.ClosePlot()

    width = pcbnew.pcbIUScale.IUTomm(pcb_bounding_box.GetWidth())
    height = pcbnew.pcbIUScale.IUTomm(pcb_bounding_box.GetHeight())"#
            );

            width = locals
                .get_item("width")
                .unwrap()
                .unwrap()
                .extract::<f64>()
                .unwrap();
            height = locals
                .get_item("height")
                .unwrap()
                .unwrap()
                .extract::<f64>()
                .unwrap();
        });

        document = document
            .set("width", format!("{}mm", width))
            .set("height", format!("{}mm", height))
            .set("viewBox", (0, 0, width, height));

        let path = format!(
            "{}/{}/{}-{}.svg",
            basedir,
            tmp_folder.clone(),
            name,
            layer.clone()
        );
        debug!("convert pcb svg: {:?}", path);
        let mut content = String::new();
        let mut group = Symbol::new()
            .set("class", layer.to_string())
            .set("id", format!("{}-{}", name, layer));

        let mut stack = SvgStack::new();
        for event in svg::open(path, &mut content).unwrap() {
            match event {
                Event::Tag(path, t, attributes) => {
                    if path == "g" {
                        match t {
                            svg::node::element::tag::Type::Start => {
                                let mut my_group = Group::new().set("class", layer.to_string());
                                for a in attributes {
                                    if a.0 != "style" {
                                        my_group = my_group.set(a.0, a.1);
                                    } else {
                                        my_group = my_group
                                            .set(a.0, clean_style(&a.1, &style, &themer).unwrap());
                                    }
                                }
                                stack.start(my_group);
                            }
                            svg::node::element::tag::Type::End => {
                                end!(group, stack);
                            }
                            svg::node::element::tag::Type::Empty => {
                                warn!("empty group: {:?}", path);
                            }
                        }
                    } else if path == "path" {
                        match t {
                            svg::node::element::tag::Type::Start => {
                                let mut my_group = Path::new().set("class", layer.to_string());
                                for a in attributes {
                                    if a.0 != "style" {
                                        my_group = my_group.set(a.0, a.1);
                                    } else {
                                        my_group = my_group
                                            .set(a.0, clean_style(&a.1, &style, &themer).unwrap());
                                    }
                                }
                                stack.start(my_group);
                            }
                            svg::node::element::tag::Type::End => {
                                end!(group, stack);
                            }
                            svg::node::element::tag::Type::Empty => {
                                let mut my_path = Path::new().set("class", layer.to_string());
                                for a in attributes {
                                    if a.0 != "style" {
                                        my_path = my_path.set(a.0, a.1);
                                    } else {
                                        my_path = my_path
                                            .set(a.0, clean_style(&a.1, &style, &themer).unwrap());
                                    }
                                }
                                stack.start(my_path);
                                end!(group, stack);
                            }
                        }
                    } else if path == "circle" {
                        match t {
                            svg::node::element::tag::Type::Start => {
                                let mut my_circle = Circle::new().set("class", layer.to_string());
                                for a in attributes {
                                    if a.0 != "style" {
                                        my_circle = my_circle.set(a.0, a.1);
                                    } else {
                                        my_circle = my_circle
                                            .set(a.0, clean_style(&a.1, &style, &themer).unwrap());
                                    }
                                }
                                stack.start(my_circle);
                            }
                            svg::node::element::tag::Type::End => {
                                stack.end();
                                end!(group, stack);
                            }
                            svg::node::element::tag::Type::Empty => {
                                let mut my_circle = Circle::new().set("class", layer.to_string());
                                for a in attributes {
                                    if a.0 != "style" {
                                        my_circle = my_circle.set(a.0, a.1);
                                    } else {
                                        my_circle = my_circle
                                            .set(a.0, clean_style(&a.1, &style, &themer).unwrap());
                                    }
                                }
                                stack.start(my_circle);
                                end!(group, stack);
                            }
                        }
                    } else if path == "text" {
                        match t {
                            svg::node::element::tag::Type::Start => {
                                let mut my_text = Text::new().set("class", layer.to_string());
                                for a in attributes {
                                    if a.0 != "style" {
                                        my_text = my_text.set(a.0, a.1);
                                    } else {
                                        my_text = my_text
                                            .set(a.0, clean_style(&a.1, &style, &themer).unwrap());
                                    }
                                }
                                stack.start(my_text);
                            }
                            svg::node::element::tag::Type::End => {
                                end!(group, stack);
                            }
                            svg::node::element::tag::Type::Empty => {
                                let mut my_text = Text::new().set("class", layer.to_string());
                                for a in attributes {
                                    if a.0 != "style" {
                                        my_text = my_text.set(a.0, a.1);
                                    } else {
                                        my_text = my_text
                                            .set(a.0, clean_style(&a.1, &style, &themer).unwrap());
                                    }
                                }
                                stack.start(my_text);
                                end!(group, stack);
                            }
                        }
                    } else if path == "desc" {
                        match t {
                            svg::node::element::tag::Type::Start => {
                                let mut my_desc =
                                    Description::new().set("class", layer.to_string());
                                for a in attributes {
                                    if a.0 != "style" {
                                        my_desc = my_desc.set(a.0, a.1);
                                    } else {
                                        my_desc = my_desc
                                            .set(a.0, clean_style(&a.1, &style, &themer).unwrap());
                                    }
                                }
                                stack.start(my_desc);
                            }
                            svg::node::element::tag::Type::End => {
                                end!(group, stack);
                            }
                            svg::node::element::tag::Type::Empty => {
                                let mut my_desc =
                                    Description::new().set("class", layer.to_string());
                                for a in attributes {
                                    if a.0 != "style" {
                                        my_desc = my_desc.set(a.0, a.1);
                                    } else {
                                        my_desc = my_desc
                                            .set(a.0, clean_style(&a.1, &style, &themer).unwrap());
                                    }
                                }
                                stack.start(my_desc);
                                end!(group, stack);
                            }
                        }
                    } else if path == "title" {
                        match t {
                            svg::node::element::tag::Type::Start => {
                                let mut my_title = Title::new().set("class", layer.to_string());
                                for a in attributes {
                                    if a.0 != "style" {
                                        my_title = my_title.set(a.0, a.1);
                                    } else {
                                        my_title = my_title
                                            .set(a.0, clean_style(&a.1, &style, &themer).unwrap());
                                    }
                                }
                                stack.start(my_title);
                            }
                            svg::node::element::tag::Type::End => {
                                end!(group, stack);
                            }
                            _ => {}
                        }
                    } else if path != "svg" {
                        warn!("O {:?}", path);
                    }
                }
                Event::Text(text) => {
                    stack.start(text.to_string());
                }
                Event::Error(error) => {
                    error!("error: {}", error);
                }
                Event::Comment(_) | Event::Declaration(_) | Event::Instruction(_) => {}
            }
        }
        document = document.add(group);
    }

    for layer in layers {
        document = document.add(
            Use::new()
                .set("href", format!("#{}-{}", name, layer))
                .set("x", 0)
                .set("y", 0),
        );
    }

    //save the target svg
    match svg::save(output.clone(), &document) {
        Ok(_) => {}
        Err(err) => {
            return Err(Error::IoError(format!(
                "Can not open output File: {} ({})",
                output, err
            )));
        }
    }

    //move the target folder
    let last_slash = output.rfind('/').unwrap(); //TODO fails when output is only file name without path
    if let Err(err) = check_directory(&format!("{}/pcb", &output[0..last_slash])) {
        return Err(Error::IoError(format!(
            "{} can not create output directory: '{}'",
            "Error:", //.red(),
            err       //TODO .bold()
        )));
    };

    let paths = fs::read_dir(format!("{}/{}", basedir, tmp_folder)).unwrap();
    for path in paths {
        let filename = path.as_ref().unwrap().file_name(); //[last_slash+1 .. path.len()];
        let target = format!(
            "{}/pcb/{}",
            &output[0..last_slash],
            filename.as_os_str().to_str().unwrap()
        );
        fs::copy(path.unwrap().path().to_str().unwrap(), target)?;
    }
    fs::remove_dir_all(format!("{}/{}", basedir, tmp_folder))?;
    Ok((width, height))
}

#[cfg(test)]
mod test {
    use super::clean_style;

    #[test]
    fn iterate() {
        let style = crate::Style::FCu;
        let themer = crate::Themer::new(crate::Theme::Kicad2020);
        let res = clean_style("fill:#000000; fill-opacity:0.0; stroke:#000000; stroke-width:0.0000; stroke-opacity:1; stroke-linecap:round; stroke-linejoin:round;", &style, &themer);
        assert_eq!("fill:#000000; fill-opacity:0.0; stroke:#000000; stroke-width:0.0000; stroke-opacity:1; stroke-linecap:round; stroke-linejoin:round; ", res.unwrap());
    }
    #[test]
    fn iterate_with_color() {
        let style = crate::Style::FCu;
        let themer = crate::Themer::new(crate::Theme::Kicad2020);
        let res = clean_style("fill:#4D7FC4; fill-opacity:0.0; stroke:#4D7FC4; stroke-width:0.2500; stroke-opacity:1; stroke-linecap:round; stroke-linejoin:round;", &style, &themer);
        assert_eq!("fill:#c83434; fill-opacity:0.0; stroke:#c83434; stroke-width:0.2500; stroke-opacity:1; stroke-linecap:round; stroke-linejoin:round; ", res.unwrap());
    }
}
