use std::path::PathBuf;

use regex::Regex;
use clap::Parser;

use rasn_compiler::prelude::*;
use ros_backend::msgs::Msgs;

#[derive(Parser, Debug)]
struct Cli {
    /// Main PDU name
    #[clap(short, long)]
    pdu: String,
    /// ASN.1 files to compile
    paths: Vec<std::path::PathBuf>,
}

fn main() {
    let args = Cli::parse();

    // Compile ROS messages
    let compiler_res = Compiler::new()
        .with_backend(Msgs)
        .add_asn_sources_by_path(
            args.paths.iter(),
        )
        .compile_to_string();
    let generated = &compiler_res.unwrap().generated;

    // Split generated code into individual messages
    let re_name = Regex::new(r"##\s([\w-]+)\s(\w+)\b").unwrap();
    let re_def = Regex::new(r"<typedef>\n((.|\n)*?)\n</typedef>").unwrap();
    generated.split_inclusive("</typedef>").for_each(|s| {
        if let Some(def_caps) = re_def.captures(s) {
            let definition = def_caps.get(1).unwrap().as_str();
            let name = if let Some(name_caps) = re_name.captures(definition) {
                name_caps.get(2).unwrap().as_str()
            } else {
                "unknown"
            };
            let path = PathBuf::from(format!("out/{}.msg", name));
            std::fs::write(path, definition).unwrap();
        }
    });
}
