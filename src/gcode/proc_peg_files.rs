// use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub(crate) fn run(dir: &Path) {
    print!("running: {:?}\n", dir);
    for entry in fs::read_dir(dir).expect("cannot read directory") {
        let entry = entry.expect("cannot read file");
        let path = entry.path();
        if path.is_dir() {
            run(&path);
        } else if path.is_file() && path.extension() == Some(OsStr::new("peg")) {
            print!("running file!!!: {:?}\n", path);
            let txt_peg = fs::read_to_string(&path).expect("failed to read input");

            let rust_rules = get_rust_rules2parse_peg2(&txt_peg);

            let gen_file = path.with_extension("rs");

            let _ = fs::rename(&gen_file, path.with_extension("rs.backup"));

            fs::write(
                &gen_file,
                format!(
                    "
#![warn(missing_docs)]
//! Module to deal with rules (aka SetOfRules)
//!

use crate::parser;

pub(crate) fn rules() -> parser::expression::SetOfRules {{
    rules!(
        {}
    )
}}
",
                    rust_rules
                ),
            )
            .expect("failed to write result");
            // fs::write(&gen_file, rust_rules).expect("failed to write result")

            // let grammar = lr1::parse(&src);
            // let mut gen = Generator::new();
            // let res = gen.build(&grammar);
            // let new_path = path.with_extension("rs");
            // fs::write(new_path, res).expect("failed to write result")
        }
    }
}

fn get_rust_rules2parse_peg2(txt_peg: &str) -> String {
    use crate::ir::IR;

    let irtxt = crate::rules_for_peg::rules()
        .parse(txt_peg)
        .unwrap()
        .replace(None)
        .unwrap();
    let ir = IR::new(&irtxt.str());

    let rules = ir.get_rules().unwrap();

    crate::gcode::rust_from_rules(&rules)
}
