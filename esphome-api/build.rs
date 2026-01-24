/*
 * ReTherm - Home Assistant native interface for Gen2 Nest thermostat
 * Copyright (C) 2026 Josh Kropf <josh@slashdev.ca>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{env, fs, path::Path};

use anyhow::Result;
use convert_case::{Case, Casing};
use proc_macro2::{TokenStream};
use quote::{format_ident, quote};
use regex::Regex;

fn main() -> Result<()> {
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
            impl MessageId for #message_type {
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
            pub fn decode<B: prost::bytes::Buf>(message_id: u64, buffer: &mut B) -> Result<Self, prost::DecodeError> {
                match message_id {
                    #(
                        #message_names::ID => Ok(ProtoMessage::#message_names(#message_names::decode(buffer)?)),
                    )*
                    _ => Err(prost::DecodeError::new(format!("Unhandled message id {message_id}")))
                }
            }
        }
    };

    let encode = quote! {
        impl ProtoMessage {
            pub fn encode<B: prost::bytes::BufMut>(&self, buffer: &mut B) -> Result<(), prost::EncodeError> {
                match self {
                    #(
                        ProtoMessage::#message_names(message) => message.encode(buffer),
                    )*
                }
            }
        }
    };

    let encoded_len = quote! {
        impl ProtoMessage {
            pub fn encoded_len(&self) -> usize {
                match self {
                    #(
                        ProtoMessage::#message_names(message) => message.encoded_len(),
                    )*
                }
            }
        }
    };

    let message_id = quote! {
        impl ProtoMessage {
            pub fn message_id(&self) -> u64 {
                match self {
                    #(
                        ProtoMessage::#message_names(_) => #message_names::ID,
                    )*
                }
            }
        }
    };

    quote! {
        #enum_def
        #decode
        #encode
        #encoded_len
        #message_id
    }
}
