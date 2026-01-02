use std::{env, fs, path::Path};

use anyhow::Result;
use convert_case::{Case, Casing};
use proc_macro2::{TokenStream};
use quote::{format_ident, quote};
use regex::Regex;

fn main() -> Result<()> {
    // https://github.com/esphome/esphome/
    // path: esphome/components/api/
    // tag: 2025.12.2

    prost_build::Config::new()
        .default_package_filename("esphome_proto")
        .compile_protos(
            &["esphome_2025.12.2/api.proto"],
            &["esphome_2025.12.2/"]
        )?;

    let messages = extract_messages("esphome_2025.12.2/api.proto")?;

    let out_dir = env::var("OUT_DIR")?;

    let dest_path = Path::new(&out_dir).join("message_ids.rs");
    let tokens = generate_message_id_impl(&messages);
    write_formatted_code(tokens, dest_path)?;

    let dest_path = Path::new(&out_dir).join("proto_message.rs");
    let tokens = generate_proto_message_enum(&messages);
    write_formatted_code(tokens, dest_path)?;

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=esphome_2025.12.2/api.proto");

    Ok(())
}

// Ideally this would use some sort of parser for protobuf syntax
fn extract_messages<P: AsRef<Path>>(proto_file: P) -> Result<Vec<(String, i32)>> {
    let input = fs::read_to_string(proto_file)?;

    let re = Regex::new(
        r"message\s+(\w+)\s*\{[^}]*?option\s*\(id\)\s*=\s*(\d+);"
    )?;

    let mut messages = vec![];

    for cap in re.captures_iter(&input) {
        let name = &cap[1];
        let id = &cap[2];
        messages.push((name.to_string(), id.parse()?));
    }

    Ok(messages)
}

fn write_formatted_code<P: AsRef<Path>>(tokens: TokenStream, file_path: P) -> Result<()> {
    // When generated code is invalid, the parse failure is way more terse
    // compared to the rust compiler.
    // Maybe dumping generated code to stderr is a good idea?
    // It would likely be very difficult to read.
    let syntax_tree = syn::parse2(tokens)?;
    let formatted = prettyplease::unparse(&syntax_tree);
    fs::write(file_path, formatted)?;
    Ok(())
}

fn generate_message_id_impl(messages: &Vec<(String, i32)>) -> TokenStream {
    let messages_id_impl = messages.iter().map(|message| {
        let message_type = message.0.to_case(Case::UpperCamel);
        let message_type = format_ident!("{}", message_type);
        let message_id = message.1 as u64;
        quote! {
            impl crate::MessageId for crate::proto::#message_type {
                const ID: u64 = #message_id;
            }
        }
    });

    quote! {
        #(
            #messages_id_impl
        )*
    }
}

fn generate_proto_message_enum(messages: &Vec<(String, i32)>) -> TokenStream {
    let message_names: Vec<_> = messages.iter()
        .map(|m| format_ident!("{}", m.0.to_case(Case::UpperCamel)))
        .collect();

    let enum_def = quote! {
        #[derive(Debug)]
        pub enum ProtoMessage {
            #(
                #message_names(#message_names),
            )*
        }
    };

    let decode = quote! {
        impl ProtoMessage {
            pub fn decode<B: Buf>(message_id: u64, buffer: &mut B) -> Result<Self> {
                match message_id {
                    #(
                        #message_names::ID => Ok(ProtoMessage::#message_names(#message_names::decode(buffer)?)),
                    )*
                    _ => Err(anyhow!("Unhandled message id {}", message_id))
                }
            }
        }
    };

    quote! {
        #enum_def
        #decode
    }
}