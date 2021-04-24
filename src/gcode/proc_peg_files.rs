// use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub(crate) fn run(dir: &Path) {
    _run(dir, false)
}

pub(crate) fn run_force(dir: &Path) {
    _run(dir, true)
}

pub(crate) fn _run(dir: &Path, force: bool) {
    print!("running: {:?}\n", dir);
    for entry in fs::read_dir(dir).expect("cannot read directory") {
        let entry = entry.expect("cannot read file");
        let path = entry.path();
        if path.is_dir() {
            run(&path);
        } else if path.is_file() && path.extension() == Some(OsStr::new("peg")) {
            let orig_file = &path;
            let dest_file = &path.with_extension("rs");

            if force || require_generation(orig_file, dest_file) {
                gen_file(&orig_file, &dest_file);
            }
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

fn require_generation(origin: &Path, destiny: &Path) -> bool {
    let created_origin = fs::metadata(origin)
        .expect(&format!("error getting metadata from file {:?}", origin))
        .modified()
        .expect(&format!("cannot read date from file {:?}", origin));
    let meta_dest = fs::metadata(destiny);

    if let Ok(md) = meta_dest {
        created_origin
            > md.modified()
                .expect(&format!("cannot read date from file {:?}", md))
    } else {
        true
    }
}

fn gen_file(origin: &Path, destiny: &Path) {
    println!("init generate file {:?}", origin);
    let txt_peg = fs::read_to_string(&origin).expect(&format!("failed to read input {:?}", origin));

    let rust_rules = get_rust_rules2parse_peg2(&txt_peg);

    let _ = fs::rename(&destiny, destiny.with_extension("rs.backup"));

    fs::write(
        &destiny,
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

    println!("end generated {:?}", origin);
}
