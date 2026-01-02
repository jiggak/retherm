use std::{fs, io::Result, path::Path};

use regex::Regex;

fn main() -> Result<()> {
    // https://github.com/esphome/esphome/
    // path: esphome/components/api/
    // tag: 2025.12.2

    prost_build::Config::new()
        .default_package_filename("esphome")
        .compile_protos(
            &["esphome_2025.12.2/api.proto"],
            &["esphome_2025.12.2/"]
        )?;

    println!("cargo::rerun-if-changed=esphome_2025.12.2/api.proto");

    Ok(())
}

fn extract_messages<P: AsRef<Path>>(proto_file: P) -> Result<Vec<(String, i32)>> {
    let input = fs::read_to_string(proto_file)?;

    let re = Regex::new(
        r"message\s+(\w+)\s*\{[^}]*?option\s*\(id\)\s*=\s*(\d+);"
    ).unwrap();

    let mut messages = vec![];

    for cap in re.captures_iter(&input) {
        let name = &cap[1];
        let id = &cap[2];
        messages.push((name.to_string(), id.parse().unwrap()));
    }

    Ok(messages)
}
