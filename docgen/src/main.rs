use std::{fs, io::Write};

use rustdoc_types::{Crate, Item, ItemEnum, ItemKind, StructKind};

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
    let krate: Crate = serde_json::from_str(&data).unwrap();
    let mut writer = std::io::stdout();

    for name in struct_names {
        struct_markdown(&mut writer, &krate, &name);
    }
}

fn find_struct<'a>(krate: &'a Crate, name: &str) -> Option<&'a Item> {
    eprintln!("Find struct {name}");

    for (key, item) in krate.paths.iter() {
        if item.kind == ItemKind::Struct && item.crate_id == 0 {
            if item.path.last().unwrap() == name {
                eprintln!("Path match {:?}", item.path);

                return krate.index.get(&key);
            }
        }
    }

    None
}

fn struct_markdown<W: Write>(out: &mut W, krate: &Crate, name: &str) {
    let struct_item = find_struct(krate, name)
        .expect(&format!("Struct {name} not found"));

    let struct_name = struct_item.name.as_ref().unwrap();
    let struct_docs = if let Some(docs) = &struct_item.docs {
        docs.lines().collect()
    } else {
        vec![]
    };

    let mut struct_docs = struct_docs.into_iter();
    let title = struct_docs.next().unwrap_or(struct_name.as_str());

    writeln!(out, "# {title}").unwrap();

    for line in struct_docs {
        writeln!(out, "{line}").unwrap();
    }

    writeln!(out, "").unwrap();

    if let ItemEnum::Struct(s) = &struct_item.inner {
        if let StructKind::Plain { fields, .. } = &s.kind {
            for field_id in fields {
                let field = &krate.index[&field_id];
                let field_name = field.name.as_ref().unwrap();
                if let Some(field_docs) = &field.docs {
                    writeln!(out, "## {}\n\n{}\n", field_name, field_docs).unwrap();
                } else {
                    eprintln!("Skipping empty docs {name}:{field_name}");
                }
            }
        }
    }
}
