use crate::ast;
/// Support for minimum expressions elements
/// Here we have the parser and types for non dependencies kind
use crate::parser::{ErrPriority, Error, Result, Status};
use std::result;

use super::ErrorAlternatives;

#[cfg(test)]
mod test;

//-----------------------------------------------------------------------
//-----------------------------------------------------------------------
//
//  T Y P E S
//
//-----------------------------------------------------------------------
//-----------------------------------------------------------------------

/// This is a minimum expression element
#[derive(Debug, PartialEq)]
pub(crate) enum Atom {
    /// Literal string
    Literal(String),
    /// Character matches a list of chars or a list of ranges
    Match(MatchRules),
    /// Indicates an error.
    /// It will propagate an error while processing
    Expected(String),
    /// Any char
    Dot,
    /// End Of File
    Eof,
}

/// contains a char slice and a (char,char) slice
/// if char matches one in char slice -> OK
/// if char matches between tuple in elems slice -> OK
#[derive(Debug, PartialEq)]
pub(crate) struct MatchRules(pub(crate) String, pub(crate) Vec<(char, char)>);

//-----------------------------------------------------------------------
//-----------------------------------------------------------------------
//
//  A P I
//
//-----------------------------------------------------------------------
//-----------------------------------------------------------------------

pub(crate) fn parse<'a>(status: Status<'a>, atom: &'a Atom) -> Result<'a> {
    match atom {
        Atom::Literal(literal) => parse_literal(status, &literal),
        Atom::Expected(error) => parse_expected(&status, &error),
        Atom::Match(ref match_rules) => parse_match(status, &match_rules),
        Atom::Dot => parse_dot(status),
        Atom::Eof => parse_eof(status),
    }
}

impl MatchRules {
    /// Create a MatchRules instance based on string and bounds
    pub(crate) fn init(s: &str, bounds: Vec<(char, char)>) -> Self {
        MatchRules(s.to_string(), bounds)
    }
    #[allow(dead_code)] //  used in tests
    pub(crate) fn new() -> Self {
        MatchRules("".to_string(), vec![])
    }
    #[allow(dead_code)] //  used in tests
    pub(crate) fn with_chars(mut self, chrs: &str) -> Self {
        self.0 = chrs.to_string();
        self
    }
    #[allow(dead_code)] //  used in tests
    pub(crate) fn with_bound_chars(mut self, bounds: Vec<(char, char)>) -> Self {
        self.1 = bounds;
        self
    }
}

//-----------------------------------------------------------------------
//
//  SUPPORT
//
//-----------------------------------------------------------------------

macro_rules! ok {
    ($st:expr, $val:expr) => {
        Ok(($st, ast::Node::Val($val.to_owned())))
    };
}

fn parse_literal<'a>(mut status: Status<'a>, literal: &'a str) -> Result<'a> {
    for ch in literal.chars() {
        status = parse_char(status, ch)
            .map_err(|st| Error::from_status_normal_simple(&st, &literal.to_string()))?;
    }
    ok!(status, literal)
}

fn parse_expected<'a>(status: &Status<'a>, error: &'a str) -> Result<'a> {
    Err(Error::from_status(
        &status,
        &ErrorAlternatives::from_string(&error),
        ErrPriority::Critical,
    ))
}

fn parse_dot(status: Status) -> Result {
    let (status, ch) = status
        .get_char()
        .map_err(|st| Error::from_status_normal_simple(&st, "anything"))?;

    ok!(status, ch.to_string())
}

fn parse_match<'a>(status: Status<'a>, match_rules: &MatchRules) -> Result<'a> {
    let match_char = |ch: char| -> bool {
        if match_rules.0.find(ch).is_some() {
            true
        } else {
            for &(b, t) in &match_rules.1 {
                if b <= ch && ch <= t {
                    return true;
                }
            }
            false
        }
    };

    status
        .get_char()
        .and_then(|(st, ch)| {
            if match_char(ch) {
                ok!(st, ch.to_string())
            } else {
                Err(st)
            }
        })
        .map_err(|st| -> Error {
            Error::from_status_normal_simple(
                &st,
                &format!("match {} {:?}", match_rules.0, match_rules.1),
            )
        })
}

fn parse_eof(status: Status) -> Result {
    match status.get_char() {
        Ok((st, _ch)) => Err(Error::from_status_normal_simple(&st, "expected EOF")),
        Err(st) => ok!(st, "EOF"),
    }
}

fn parse_char(status: Status, ch: char) -> result::Result<Status, Status> {
    let (st, got_ch) = status.get_char()?;
    if ch == got_ch {
        Ok(st)
    } else {
        Err(st)
    }
}

impl<'a> Status<'a> {
    fn get_char(mut self) -> result::Result<(Self, char), Self> {
        match self.it_parsing.next() {
            None => Err(self),
            Some(ch) => {
                self.pos.n += 1;
                match ch {
                    '\n' => {
                        self.pos.col = 0;
                        self.pos.row += 1;
                        self.pos.start_line = self.pos.n;
                    }
                    '\r' => {
                        self.pos.col = 0;
                    }
                    _ => {
                        self.pos.col += 1;
                    }
                }
                Ok((self, ch))
            }
        }
    }
}
