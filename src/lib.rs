#![warn(missing_docs)]
// #![feature(external_doc)]
// #![doc(include = "../README.md")]

//! For an introduction and context view, read...
//!
//! [README.md](https://github.com/jleahred/yepp)
//!
//! A very basic example...
//! ```rust
//! extern crate yepp;
//!
//! fn main() -> Result<(), yepp::Error> {
//!     let result = yepp::Peg::new(
//!         "
//!         main    =   char+
//!         char    =   'a'     -> A
//!                 /   'b'     -> B
//!                 /   .
//!     ",
//!     )
//!     .gen_rules()?
//!     .parse("aaacbbabdef")?
//!     .replace(None)?
//!     //  ...
//!     ;
//!
//!     println!("{:#?}", result);
//!     Ok(())
//! }
//!
//!```
//!
//!
//!
//! Please, read [README.md](https://github.com/jleahred/yepp) for
//! more context information
//!

extern crate idata;
extern crate im;

use std::result;

use parser::ErrorAlternatives;

#[macro_use]
pub(crate) mod macros;
pub(crate) mod ast;
pub(crate) mod gcode;
pub(crate) mod ir;
pub(crate) mod parser;
pub(crate) mod rules_for_peg;

// -------------------------------------------------------------------------------------
//  T Y P E S

//  T Y P E S
// -------------------------------------------------------------------------------------

// -------------------------------------------------------------------------------------
//  A P I

/// Peg type for fluent API
pub struct Peg<'a>(&'a str);

/// Errors for fluent API
#[derive(Debug)]
pub enum Error {
    /// error on parsing
    ParserErr(crate::parser::Error),
    /// error on replace
    ReplaceErr(String),
    /// error processing IR
    IRErr(crate::ir::Error),
}

impl<'a> Peg<'a> {
    /// create an instance of Peg
    pub fn new(txt: &'a str) -> Self {
        Self(txt)
    }

    /// generate rules from peg grammar (fluent API)
    pub fn gen_rules(&self) -> result::Result<crate::parser::expression::SetOfRules, Error> {
        use crate::ir::IR;

        let irtxt = crate::rules_for_peg::rules().parse(self.0)?.replace(None)?;
        let ir = IR::new(&irtxt.str());

        Ok(ir.get_rules().unwrap())
    }
}

impl crate::parser::expression::SetOfRules {
    /// parse from a set of rules (fluent API)
    pub fn parse(&self, text: &str) -> Result<ast::Node, Error> {
        crate::parse(text, self).map_err(|e| Error::ParserErr(e))
    }

    /// parse with debug info
    pub fn parse_debug(&self, text: &str) -> Result<ast::Node, Error> {
        crate::parse_debug(text, self).map_err(|e| Error::ParserErr(e))
    }
}

///  given a file or dir, process the .peg files
///  generating rust code
pub fn process_peg_files(dir: &std::path::Path) {
    gcode::proc_peg_files::run(dir)
}

/// Type to user defined funtions callbacks
pub struct FnCallBack(pub fn(&str) -> Option<String>);

impl ast::Node {
    /// run the tree replacing acording the rules
    pub fn replace(
        &self,
        fcallback: Option<&FnCallBack>,
    ) -> Result<crate::ast::replace::Replaced, Error> {
        ast::replace::replace(&self, fcallback).map_err(|e| Error::ReplaceErr(e))
    }
}

//  A P I
// -------------------------------------------------------------------------------------

// -------------------------------------------------------------------------------------

fn parse(s: &str, rules: &parser::expression::SetOfRules) -> Result<ast::Node, parser::Error> {
    parse_with_debug(s, rules, false)
}

fn parse_debug(
    s: &str,
    rules: &parser::expression::SetOfRules,
) -> Result<ast::Node, parser::Error> {
    parse_with_debug(s, rules, true)
}

fn parse_with_debug(
    s: &str,
    rules: &parser::expression::SetOfRules,
    debug: bool,
) -> Result<ast::Node, parser::Error> {
    let (st, ast) = if debug {
        parser::expression::parse(parser::Status::init_debug(s, &rules, debug))?
    } else {
        parser::expression::parse(parser::Status::init(s, &rules))?
    };
    match (st.pos.n == s.len(), st.potential_error.clone()) {
        (true, _) => Ok(ast),
        (false, Some(e)) => Err(e),
        (false, None) => Err(parser::Error::from_status_normal_simple(
            &st,
            "not consumed full input",
        )),
    }
}

// -------------------------------------------------------------------------------------
