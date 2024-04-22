use std::{collections::HashMap, path::Path};

use pyo3::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    error::NotebookError,
    notebook::ArgType,
    utils::{check_directory, Symbols},
};

use super::{args_to_string, get_value, param, param_or, CellWrite, CellWriter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioCell(pub usize, pub HashMap<String, ArgType>, pub Vec<String>);
impl CellWrite<AudioCell> for CellWriter {
    fn write(
        out: &mut dyn std::io::Write,
        py: &pyo3::Python,
        globals: &Bound<pyo3::types::PyDict>,
        locals: &Bound<pyo3::types::PyDict>,
        cell: &AudioCell,
        input: &str,
        dest: &str,
    ) -> Result<(), NotebookError> {
        let _body = &cell.2;
        let args = &cell.1;

        let ext = param_or!(args, "ext", "wav");
        let data_key = param_or!(args, "data", "py$ret");
        let samplerate = param!(
            args,
            "samplerate",
            NotebookError::new(
                input.to_string(),
                String::from("AudioCell"),
                String::from("PropertyError"),
                String::from("property samplerate not found"),
                cell.0,
                cell.0,
                None
            )
        );

        //get the data from the pyhton context
        let Ok(py_data) = get_value(samplerate, py, globals, locals) else {
            return Err(NotebookError::new(
                input.to_string(),
                String::from("AudioCell"),
                String::from("VariableError"),
                format!("Variable with name '{}' can not be found.", data_key),
                cell.0,
                cell.0,
                None,
            ));
        };

        let samplerate = if let Ok(data) = py_data.extract::<u32>() {
            Some(data)
        } else {
            None
        }
        .unwrap();

        let v = get_value(data_key, py, globals, locals).map_err(|err| {
            NotebookError::new(
                input.to_string(),
                String::from("AudioCell"),
                String::from("VariableError"),
                err.0,
                cell.0,
                cell.0,
                None,
            )
        })?;
        if let Ok(data) = v.extract::<Vec<f32>>() {
            writeln!(
                out,
                "{{{{< audio {}>}}}}",
                args_to_string(
                    &write_audio(dest, data, ext, samplerate, args).map_err(|err| {
                        NotebookError::new(
                            input.to_string(),
                            String::from("AudioCell"),
                            String::from("WriteError"),
                            format!("can not write audio file: {} ({})", dest, err),
                            cell.0,
                            cell.0,
                            None,
                        )
                    })?
                )
            )
            .unwrap();
            Ok(())
        } else {
            Err(NotebookError::new(
                input.to_string(),
                String::from("AudioCell"),
                String::from("ValueError"),
                String::from("plot value must be of type HashMap<String<Vec<f32>>"),
                cell.0,
                cell.0,
                None,
            ))
        }
    }
}

pub fn write_audio(
    path: &str,
    audio: Vec<f32>,
    ext: &str,
    fs: u32,
    args: &HashMap<String, ArgType>,
) -> Result<HashMap<String, ArgType>, std::io::Error> {
    let out_dir = Path::new(path).join("_files");
    let rand_string: String = thread_rng()
        .sample_iter(&Symbols)
        .take(30)
        .map(char::from)
        .collect();
    let output_file = out_dir
        .join(format!("{}.{}", rand_string, ext))
        .to_str()
        .unwrap()
        .to_string();

    check_directory(&output_file)?;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: fs,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(output_file, spec).unwrap();
    for float in audio {
        writer.write_sample(float).unwrap();
    }
    let mut myargs = args.clone();
    if let Some(ArgType::Options(opts)) = myargs.get_mut("options") {
        opts.insert(
            String::from("path"),
            ArgType::String(format!("_files/{}.{}", rand_string, ext)),
        );
    } else {
        let mut map = HashMap::new();
        map.insert(
            String::from("path"),
            ArgType::String(format!("_files/{}.{}", rand_string, ext)),
        );
        myargs.insert(String::from("options"), ArgType::Options(map));
    }
    Ok(myargs)
}
