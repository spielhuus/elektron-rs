use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use super::super::cells::{CellWrite, CellWriter};
use crate::{error::NotebookError, notebook::ArgType};

use super::{args_to_string, get_value, param, param_or};

pub fn check_directory(filename: &str) -> Result<(), std::io::Error> {
    let path = std::path::Path::new(filename);
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct D3Cell(pub usize, pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<D3Cell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &pyo3::types::PyDict,
        locals: &pyo3::types::PyDict,
        cell: &D3Cell,
        input: &str,
        dest: &str,
    ) -> Result<(), NotebookError> {
        let code = &cell.2;
        let args = &cell.1;

        //collect the required arguements.
        let x_key = param!(
            args,
            "x",
            NotebookError::new(input.to_string(),"D3Cell".to_string(), "ParamError".to_string(), String::from("property 'x' must be set."), cell.0, cell.0, None)
        );
        let width = param!(
            args,
            "width",
            NotebookError::new(input.to_string(), "D3Cell".to_string(), "ParamError".to_string(), String::from("property 'width' must be set."), cell.0, cell.0, None)
        );
        let height = param!(
            args,
            "height",
            NotebookError::new(input.to_string(),"D3Cell".to_string(), "ParamError".to_string(), String::from("property 'height' must be set."), cell.0, cell.0, None)
        );
        let data_key = param!(
            args,
            "data",
            NotebookError::new(input.to_string(), "D3Cell".to_string(), "ParamError".to_string(), String::from("property 'data' must be set."), cell.0, cell.0, None)
        );

        //collect optional data
        let y_type = param_or!(args, "yType", "scaleLinear");
        let x_type = param_or!(args, "xType", "scaleLinear");

        let mut y_keys: Option<Vec<String>> = if let Some(ArgType::String(y_key)) = args.get("y") {
            let mut res = Vec::new();
            for key in y_key.split(',') {
                res.push(key.to_string());
            }
            Some(res)
        } else {
            None
        };

        let y_range: Option<Vec<String>> =
            if let Some(ArgType::String(y_range)) = args.get("yRange") {
                let mut res: Vec<String> = Vec::new();
                for r in y_range.split(',') {
                    res.push(r.to_string());
                }
                Some(res)
            } else {
                None
            };

        //get the data from the pyhton context
        let Ok(py_data) = get_value(data_key.as_str(), py, globals, locals) else {
            return Err(NotebookError::new(input.to_string(), "D3Cell".to_string(), "VariableError".to_string(), format!(
                "Variable with name '{}' can not be found.",
                data_key), cell.0, cell.0, None
            ));
        };

        let data = if let Ok(data) = py_data.extract::<HashMap<String, Vec<f64>>>() {
            Some(data)
        } else {
            None
        };

        let range_data =
            if let Ok(data) = py_data.extract::<HashMap<String, HashMap<String, Vec<f64>>>>() {
                Some(data)
            } else {
                None
            };

        //get the key for the element
        let key = if data_key.starts_with("py$") {
            data_key.strip_prefix("py$").unwrap()
        } else if data_key.contains('.') {
            data_key.split('.').next().unwrap()
        } else {
            data_key
        };

        let element = param_or!(args, "element", key);

        //create output directory
        let out_dir = Path::new(dest).join("_files");
        let output_file = out_dir.to_str().unwrap().to_string();
        check_directory(&output_file).map_err(|e| NotebookError::new(input.to_string(), "D3Cell".to_string(), "IOError".to_string(), e.to_string(), cell.0, cell.0, None))?;

        let mut x_domain: Option<(f64, f64)> = None;
        let mut y_domain: Option<(f64, f64)> = None;
        let mut y_size = 0;

        //process data
        if let Some(data) = data {
            let output_file = out_dir
                .join(format!("{}_0.json", element))
                .to_str()
                .unwrap()
                .to_string();

            let mut keys = Vec::new();
            let mut res = HashMap::new();
            for (k, v) in &data {
                if k == x_key {
                    res.insert(k, v);
                    x_domain = Some(range(x_domain, v));
                } else if let Some(y_keys) = &y_keys {
                    if y_keys.contains(k) || x_key == k {
                        res.insert(k, v);
                        y_domain = Some(range(y_domain, v));
                    }
                } else if y_keys.is_none() {
                    res.insert(k, v);
                    y_domain = Some(range(y_domain, v));
                    keys.push(k.to_string());
                }
            }
            //set all keys if not set
            if y_keys.is_none() {
                y_keys = Some(keys);
            }
            let mut outfile = File::create(output_file).map_err(|e| NotebookError::new(input.to_string(), "D3Cell".to_string(), "IOError".to_string(), e.to_string(), cell.0, cell.0, None))?;
            outfile.write_all(Json::to_string(&res).as_bytes()).map_err(|e| NotebookError::new(input.to_string(), "D3Cell".to_string(), "IOError".to_string(), e.to_string(), cell.0, cell.0, None))?;
        } else if let Some(data) = range_data {
            let series = data_key.rsplit('.').next().unwrap();
            for (plot, plots) in data {
                y_size += 1;
                if plot.starts_with(series) {
                    let mut res = HashMap::new();
                    let number = if plot == series {
                        1
                    } else {
                        plot.strip_prefix(series).unwrap().parse::<usize>().unwrap()
                    };
                    for (key, vec) in &plots {
                        if key == x_key {
                            res.insert(key, vec);
                            x_domain = Some(range(x_domain, vec));
                            continue;
                        }
                        if let Some(y_keys) = &y_keys {
                            if y_keys.contains(key) {
                                res.insert(key, vec);
                                y_domain = Some(range(y_domain, vec));
                                continue;
                            }
                        }
                        if let Some(y_range) = &y_range {
                            if y_range.contains(key) {
                                res.insert(key, vec);
                                y_domain = Some(range(y_domain, vec));
                                continue;
                            }
                        }
                    }
                    let output_file = out_dir
                        .join(format!("{}_{}.json", element, number - 1))
                        .to_str()
                        .unwrap()
                        .to_string();
                    let mut outfile = File::create(output_file).map_err(|e| NotebookError::new(input.to_string(), "D3Cell".to_string(), "IOError".to_string(), e.to_string(), cell.0, cell.0, None))?;
                    outfile.write_all(Json::to_string(&res).as_bytes()).map_err(|e| NotebookError::new(input.to_string(), "D3Cell".to_string(), "IOError".to_string(), e.to_string(), cell.0, cell.0, None))?;
                }
            }
        } else {
            return Err(NotebookError::new(
                input.to_string(),
                "D3Cell".to_string(),
                "ParamError".to_string(),
                format!("plot value must be of type HashMap<Sting<Vec<f64>>, but is {:?}", py_data),
                cell.0, 
                cell.0, 
                None
            ));
        }

        let colors = COLORS
            .iter()
            .copied()
            .take(
                if let Some(y_keys) = &y_keys {
                    y_keys.len()
                } else {
                    0
                } + if let Some(y_range) = &y_range {
                    y_range.len()
                } else {
                    0
                },
            )
            .collect::<Vec<&str>>();

        let (min_x, max_x) = if let Some(ArgType::String(xdomain)) = args.get("xDomain") {
            let tokens: Vec<&str> = xdomain.split(',').map(|t| t.trim()).collect();
            if tokens.len() == 2 {
                if let (Ok(min), Ok(max)) = (tokens[0].parse::<f64>(), tokens[1].parse::<f64>()) {
                    Ok((min, max))
                } else {
                    Err(NotebookError::new(input.to_string(), String::from("D3Cell"), String::from("VariableError"), String::from(
                        "Variable xDomain must be \"min, max\"",
                    ), cell.0, cell.0, None))
                }
            } else {
                Err(NotebookError::new(input.to_string(), String::from("D3Cell"), String::from("VariableError"), String::from(
                    "Variable xDomain must be \"min, max\"",
                ), cell.0, cell.0, None))
            }
        } else {
            x_domain.ok_or_else(|| NotebookError::new(
                    input.to_string(), 
                    String::from("D3Cell"), 
                    String::from("VariableError"), 
                    String::from("xDomain not calculated."),
                    cell.0,
                    cell.0,
                    None))
        }?;

        let (min_y, max_y) = if let Some(ArgType::String(ydomain)) = args.get("yDomain") {
            let tokens: Vec<&str> = ydomain.split(',').map(|t| t.trim()).collect();
            if tokens.len() == 2 {
                if let (Ok(min), Ok(max)) = (tokens[0].parse::<f64>(), tokens[1].parse::<f64>()) {
                    Ok((min, max))
                } else {
                    Err(NotebookError::new(input.to_string(), String::from("D3Cell"), String::from("VariableError"), String::from(
                        "Variable yDomain must be \"min, max\"",
                    ), cell.0, cell.0, None))
                }
            } else {
                Err(NotebookError::new(input.to_string(), String::from("D3Cell"), String::from("VariableError"), String::from(
                    "Variable yDomain must be \"min, max\"",
                ), cell.0, cell.0, None))
            }
        } else {
            y_domain.ok_or_else(|| NotebookError::new(input.to_string(), String::from("D3Cell"), String::from("VariableError"), String::from("yDomain not calculated."), cell.0, cell.0, None))
        }?;

        writeln!(out, "{{{{< d3 key=\"{}\" x=\"{}\" y=\"{}\" yRange=\"{}\" ySize=\"{}\" xDomain=\"{}, {}\" yDomain=\"{}, {}\" \
                      width=\"{}\" height=\"{}\" yType=\"{}\" xType=\"{}\" colors=\"{}\" xLabel=\"{}\" yLabel=\"{}\" range=\"{}\" {}>}}}}", 
                element, x_key, 
                if let Some(y_keys) = y_keys { y_keys.join(",") } else { String::new() }, 
                if let Some(y_range) = y_range { y_range.join(",") } else { String::new() }, 
                y_size - 2,
                min_x, max_x, min_y, max_y, width, height, y_type, x_type, colors.join(","),
                param_or!(args, "xLabel", ""),
                param_or!(args, "yLabel", ""),
                param_or!(args, "range", ""),
                args_to_string(args))
            .unwrap();
        out.write_all(code.join("\n").as_bytes()).unwrap();
        writeln!(out, "{{{{< /d3 >}}}}").unwrap();

        Ok(())
    }
}

struct Json;
trait ToJson<T> {
    fn to_string(data: T) -> String;
}

impl ToJson<&HashMap<&String, &Vec<f64>>> for Json {
    fn to_string(data_map: &HashMap<&String, &Vec<f64>>) -> String {
        let mut result = json::JsonValue::new_object();
        for key in data_map.keys() {
            let mut items = json::JsonValue::new_array();
            for data in data_map[key] {
                items.push(*data).unwrap();
            }
            result.insert(key, items).unwrap();
        }
        result.to_string()
    }
}

impl ToJson<&Vec<f64>> for Json {
    fn to_string(datas: &Vec<f64>) -> String {
        let mut items = json::JsonValue::new_array();
        for data in datas {
            items.push(*data).unwrap();
        }
        items.to_string()
    }
}

const COLORS: &[&str; 135] = &[
    "Red",
    "Green",
    "Blue",
    "Orange",
    "Cyan",
    "Magenta",
    "Yellow",
    "Purple",
    "Black",
    "Violet",
    "Bisque",
    "BlanchedAlmond",
    "BlueViolet",
    "Brown",
    "BurlyWood",
    "CadetBlue",
    "Chartreuse",
    "Chocolate",
    "Coral",
    "CornflowerBlue",
    "Cornsilk",
    "Crimson",
    "DarkBlue",
    "DarkCyan",
    "DarkGoldenRod",
    "DarkGrey",
    "DarkGreen",
    "DarkKhaki",
    "DarkMagenta",
    "DarkOliveGreen",
    "Darkorange",
    "DarkOrchid",
    "DarkRed",
    "DarkSalmon",
    "DarkSeaGreen",
    "DarkSlateBlue",
    "DarkSlateGrey",
    "DarkTurquoise",
    "DarkViolet",
    "DeepPink",
    "DeepSkyBlue",
    "DimGray",
    "DodgerBlue",
    "FireBrick",
    "FloralWhite",
    "ForestGreen",
    "Fuchsia",
    "Gainsboro",
    "GhostWhite",
    "Gold",
    "GoldenRod",
    "Grey",
    "GreenYellow",
    "HoneyDew",
    "HotPink",
    "IndianRed",
    "Indigo",
    "Ivory",
    "Khaki",
    "Lavender",
    "LavenderBlush",
    "LawnGreen",
    "LemonChiffon",
    "LightBlue",
    "LightCoral",
    "LightCyan",
    "LightGoldenRodYellow",
    "LightGrey",
    "LightGreen",
    "LightPink",
    "LightSalmon",
    "LightSeaGreen",
    "LightSkyBlue",
    "LightSlateGrey",
    "LightSteelBlue",
    "LightYellow",
    "Lime",
    "LimeGreen",
    "Linen",
    "Maroon",
    "MediumAquaMarine",
    "MediumBlue",
    "MediumOrchid",
    "MediumPurple",
    "MediumSeaGreen",
    "MediumSlateBlue",
    "MediumSpringGreen",
    "MediumTurquoise",
    "MediumVioletRed",
    "MidnightBlue",
    "MintCream",
    "MistyRose",
    "Moccasin",
    "NavajoWhite",
    "Navy",
    "OldLace",
    "Olive",
    "OliveDrab",
    "OrangeRed",
    "Orchid",
    "PaleGoldenRod",
    "PaleGreen",
    "PaleTurquoise",
    "PaleVioletRed",
    "PapayaWhip",
    "PeachPuff",
    "Peru",
    "Pink",
    "Plum",
    "PowderBlue",
    "RosyBrown",
    "RoyalBlue",
    "SaddleBrown",
    "Salmon",
    "SandyBrown",
    "SeaGreen",
    "SeaShell",
    "Sienna",
    "Silver",
    "SkyBlue",
    "SlateBlue",
    "SlateGrey",
    "Snow",
    "SpringGreen",
    "SteelBlue",
    "Tan",
    "Teal",
    "Thistle",
    "Tomato",
    "Turquoise",
    "Wheat",
    "WhiteSmoke",
    "YellowGreen",
    "Aqua",
    "Aquamarine",
];

fn range(old: Option<(f64, f64)>, data: &[f64]) -> (f64, f64) {
    let mut max = data.first().unwrap();
    let mut min = data.first().unwrap();
    for v in data {
        if max < v {
            max = v;
        }
        if min > v {
            min = v;
        }
    }
    if let Some(old) = old {
        (
            if old.0 < *min { old.0 } else { *min },
            if old.1 > *max { old.1 } else { *max },
        )
    } else {
        (*min, *max)
    }
}

#[cfg(test)]
mod tests {
    use super::range;

    #[test]
    fn test_range() {
        assert_eq!((-1.0, 1.0), range(None, &[1.0, 0.9, 0.3, 0.0, -1.0, -0.4]));
        assert_eq!(
            (-1.3, 1.2),
            range(None, &[1.0, 1.2, -1.3, 0.9, 0.3, 0.0, -1.0, -0.4])
        );
        assert_eq!((1.0, 1.3), range(None, &[1.0, 1.1, 1.2, 1.3]));
    }
}
