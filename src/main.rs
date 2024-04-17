use rasn_compiler::prelude::*;
use regex::Regex;

use ros_backend::ros::Ros;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <ASN.1 files>", args[0]);
        std::process::exit(1);
    }

    // Compile ROS messages
    let compiler_res = Compiler::new()
        .with_backend(Ros)
        .add_asn_sources_by_path(
            args[1..]
            .iter(),
        )
        .compile_to_string();
    let generated = &compiler_res.unwrap().generated;

    // Split generated code into individual messages
    let re_name = Regex::new(r"##\s([\w-]+)\s(\w+)\b").unwrap();
    let re_def = Regex::new(r"#<typedef>\n((.|\n)*?)\n#</typedef>").unwrap();
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
