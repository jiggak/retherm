use serde_json::{Map, Value};
use std::{fs, io::Write};

fn main() {
    let mut args = std::env::args();

    // skip first arg, the program name/path
    args.next();

    if args.len() < 2 {
        println!("usage: [JSON FILE] [STRUCT NAMES]...");
        return;
    }

    let file_path = args.next().unwrap();

    let struct_names: Vec<_> = args.collect();

    let data = fs::read_to_string(file_path).unwrap();
    let json: Value = serde_json::from_str(&data).unwrap();
    let mut writer = std::io::stdout();

    for name in struct_names {
        struct_markdown(&mut writer, &json, &name);
    }
}

fn find_struct<'a>(json: &'a Value, name: &str) -> Option<&'a Map<String, Value>> {
    let index = json["index"].as_object().unwrap();
    let paths = json["paths"].as_object().unwrap();

    for (key, item) in paths {
        if item["kind"] == "struct" {
            let path = item["path"].as_array().unwrap();
            if path.last().unwrap() == name {
                return index[key].as_object();
            }
        }
    }

    None
}

fn struct_markdown<W: Write>(out: &mut W, json: &Value, name: &str) {
    let index = json["index"].as_object().unwrap();
    let struct_obj = find_struct(json, name)
        .expect(&format!("Struct {name} not found"));

    let name = struct_obj["name"].as_str().unwrap();
    let docs = struct_obj["docs"].as_str().unwrap_or("");

    let mut lines = docs.lines();
    let title = lines.next().unwrap_or(name);

    writeln!(out, "# {title}").unwrap();

    for line in lines {
        writeln!(out, "{line}").unwrap();
    }

    writeln!(out, "").unwrap();

    if let Some(fields) = struct_obj["inner"]["struct"]["kind"]["plain"]["fields"].as_array() {
        for field_id in fields {
            let field = &index[&field_id.to_string()];
            let field_name = field["name"].as_str().unwrap();
            let field_docs = field["docs"].as_str().unwrap_or("");

            writeln!(out, "## {}\n\n{}\n", field_name, field_docs).unwrap();
        }
    }
}
