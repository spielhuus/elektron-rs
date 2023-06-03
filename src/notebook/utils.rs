use rand::{
    distributions::{Alphanumeric, DistString},
    prelude::Distribution,
    seq::SliceRandom,
    thread_rng, Rng,
};
use std::{collections::HashMap, io::Write, path::Path};

use super::parser::ArgType;
use crate::error::Error;

static ERR_NO_INPUT: &str = "Input is not set:";
static ERR_FILE: &str = "Unable to open file:";
static ERR_CAST: &str = "Cast Error:";
static ERR_VAR: &str = "Variable not found:";
static ERR_PYTHON: &str = "Python Syntax Error:";
static ERR_LATEX: &str = "LATEX Syntax Error:";
static ERR_NO_FIGURE: &str = "Source Figure is not set:";
static ERR_STDERR: &str = "";
static ERR_PARSE: &str = "Can not parse Input:";

pub fn echo(out: &mut dyn Write, lang: &str, code: &str, args: &HashMap<String, ArgType>) {
    if let Some(ArgType::String(echo)) = args.get("echo") {
        if echo == "FALSE" {
            return;
        }
    }
    writeln!(out, "```{}", lang).unwrap();
    out.write_all(code.as_bytes()).unwrap();
    writeln!(out, "\n```").unwrap();
}

pub fn newlines(input: String) -> String {
    input
        .lines()
        .into_iter()
        .collect::<Vec<&str>>()
        .join("<br/>")
}

/* pub fn content(out: &mut dyn Write, content: &[u8], args: &HashMap<String, ArgType>) {
    if let Some(ArgType::String(result)) = args.get("results") {
        if result == "hide" {
            return;
        }
    }
    if !content.is_empty() {
        writeln!(out, "{{{{< result >}}}}").unwrap();
        out.write_all(content).unwrap();
        writeln!(out, "{{{{< /result >}}}}\n").unwrap();
    }
} */

/* pub fn error(out: &mut dyn Write, errtype: &str, content: &[u8], args: &HashMap<String, ArgType>) {
    if let Some(ArgType::String(result)) = args.get("error") {
        if result == "hide" {
            return;
        }
    }
    writeln!(out, "{{{{< error message=\"{}\" >}}}}", errtype).unwrap();
    out.write_all(content).unwrap();
    writeln!(out, "{{{{< /error >}}}}\n").unwrap();
} */

/* pub fn figure(out: &mut dyn Write, buffer: &[u8], args: &HashMap<String, ArgType>) {
    writeln!(out, "{{{{< figure {}>}}}}", args_to_string(args)).unwrap();
    out.write_all(buffer).unwrap();
    writeln!(out, "{{{{< /figure >}}}}").unwrap();
} */

pub struct Symbols;
impl Distribution<char> for Symbols {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        const RANGE: u32 = 26;
        const GEN_ASCII_STR_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
        loop {
            let var = rng.next_u32() >> (32 - 6);
            if var < RANGE {
                return GEN_ASCII_STR_CHARSET[var as usize] as char;
            }
        }
    }
}

/* pub fn d3(out: &mut dyn Write, buffer: &[u8], data: &str, args: &HashMap<String, ArgType>) {
    let key = if let Some(ArgType::String(key)) = args.get("key") {
        key.to_string()
    } else {
        thread_rng()
            .sample_iter(&Symbols)
            .take(30)
            .map(char::from)
            .collect()
    };

    writeln!(out, "{{{{< d3 key=\"{}\" x=\"{}\" y=\"{}\" {}>}}}}", key, args.get("x").unwrap(), args.get("y").unwrap(), args_to_string(args)).unwrap();
    writeln!(out, "let {} = {};", key, data).unwrap();
    out.write_all(buffer).unwrap();
    writeln!(out, "{{{{< /d3 >}}}}").unwrap();
} */

/* pub fn bom(out: &mut dyn Write, dir: &str, args: &HashMap<String, ArgType>) {
    let group = if let Some(ArgType::String(group)) = args.get("group") {
        group == "TRUE" || group == "true"
    } else { false };
    if let Some(ArgType::List(input)) = args.get("input") {
        writeln!(out, "bom:").unwrap();
        for input in input {
            let input_file = Path::new(&dir).join(input).join(format!("{}.kicad_sch", input)).to_str().unwrap().to_string();
            if let Ok(schema) = Schema::load(input_file.as_str()) {
                let res = reports::bom(&schema, group).unwrap();

                writeln!(out, "  {}:", input).unwrap();
                for item in res {
                    writeln!(out, "    -").unwrap();
                    writeln!(out, "       amount: {}", item.amount).unwrap();
                    writeln!(out, "       value: {}", item.value).unwrap();
                    writeln!(out, "       references: {}", item.references.join(" ")).unwrap();
                    writeln!(out, "       description: {}", item.description).unwrap();
                    writeln!(out, "       footprint: {}", item.footprint).unwrap();
                }
            } else { error(out, ERR_FILE, input_file.as_bytes(), args); }
        }
    } else { error(out, ERR_NO_INPUT, &[], args); }
} */

/* pub fn schema(out: &mut dyn Write, dir: &str, args: &HashMap<String, ArgType>) {
    let out_dir = Path::new(dir).join("_files");
    check_directory(out_dir.to_str().unwrap());
    let border = if let Some(ArgType::String(group)) = args.get("border") {
        group == "TRUE" || group == "true"
    } else { false };
    if let Some(ArgType::List(input)) = args.get("input") {
        writeln!(out, "schema:").unwrap();
        for input in input {
            let input_file = Path::new(&dir).join(input).join(format!("{}.kicad_sch", input)).to_str().unwrap().to_string();
            let output_file = out_dir.join(format!("{}_schema.svg", input)).to_str().unwrap().to_string();

            if let Ok(schema) = Schema::load(input_file.as_str()) {
                plot::plot_schema(&schema, output_file.as_str(), 1.0, border, "kicad_2000", None).unwrap();
                writeln!(out, "  {}: {}", input, output_file).unwrap();
            } else { error(out, ERR_FILE, input_file.as_bytes(), args); }
        }
    } else { error(out, ERR_NO_INPUT, &[], args); }
} */

/* pub fn pcb(out: &mut dyn Write, dir: &str, args: &HashMap<String, ArgType>) {
    let out_dir = Path::new(dir).join("_files");
    check_directory(out_dir.to_str().unwrap());
    let border = if let Some(ArgType::String(group)) = args.get("border") {
        group == "TRUE" || group == "true"
    } else { false };
    if let Some(ArgType::List(input)) = args.get("input") {
        writeln!(out, "pcb:").unwrap();
        for input in input {
            let input_file = Path::new(&dir).join(input).join(format!("{}.kicad_pcb", input)).to_str().unwrap().to_string();
            let output_file = out_dir.join(format!("{}_pcb.svg", input)).to_str().unwrap().to_string();

            if let Ok(schema) = Pcb::load(input_file.as_str()) {
                plot::plot_pcb(&schema, output_file.as_str(), 1.0, border, "kicad_2000").unwrap();
                writeln!(out, "  {}: {}", input, output_file).unwrap();
            } else { error(out, ERR_FILE, input_file.as_bytes(), args); }
        }
    } else { error(out, ERR_NO_INPUT, &[], args); }
} */

pub fn clean_svg(input: &str) -> String {
    let mut vec: Vec<String> = Vec::new();
    let rand_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    for line in input.lines() {
        if !line.starts_with("<?xml version=") {
            let res = line.replace("id=\"", format!("id=\"{}", rand_string).as_str());
            let res = res.replace(
                "xlink:href=\"#",
                format!("xlink:href=\"#{}", rand_string).as_str(),
            );
            vec.push(res);
        }
    }
    vec.join("\n")
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};

    use super::Symbols;

    #[test]
    fn test_standard() {
        let rand_string1: String = thread_rng()
            .sample_iter(&Symbols)
            .take(30)
            .map(char::from)
            .collect();

        let rand_string2: String = thread_rng()
            .sample_iter(&Symbols)
            .take(30)
            .map(char::from)
            .collect();

        assert!(rand_string1.chars().any(|c| matches!(c, 'a'..='z')));
        assert!(rand_string2.chars().any(|c| matches!(c, 'a'..='z')));
        assert_ne!(rand_string1, rand_string2);
    }
}
