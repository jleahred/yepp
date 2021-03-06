#![warn(missing_docs)]

pub(crate) mod proc_peg_files;

use crate::parser::{
    atom,
    atom::Atom,
    expression::{self, Expression, MetaExpr, MultiExpr, ReplTemplate, RuleInfo},
};
use idata::IString;

/// Generate a string with rust code from a ```expression::SetOfRules```
pub(crate) fn rust_from_rules(rules: &expression::SetOfRules) -> String {
    let add_rule = |crules: String, rule: &str| -> String {
        let begin = if crules.is_empty() { "  " } else { ", " };
        crules + "\n       " + begin + rule
    };

    rules.0.iter().fold("".to_string(), |acc, (name, ri)| {
        add_rule(acc, &rule2code(name, ri))
    })
}

fn rule2code(name: &str, ri: &RuleInfo) -> String {
    format!(
        r##"r#"{}"# => RuleInfo{{ expr:{}, descr:{} }}"##,
        name,
        expr2code(&ri.expr),
        match &ri.descr {
            Some(d) => format!("Some({})", d),
            None => "None".to_owned(),
        }
    )
}

fn expr2code(expr: &Expression) -> String {
    match expr {
        Expression::Simple(atom) => atom2code(atom),
        Expression::And(mexpr) => format!("and!({})", mexpr2code(mexpr)),
        Expression::Or(mexpr) => format!("or!({})", mexpr2code(mexpr)),
        Expression::Not(e) => format!("not!({})", expr2code(e)),
        Expression::Peek(e) => format!("peek!({})", expr2code(e)),
        Expression::Repeat(rep) => repeat2code(rep),
        Expression::RuleName(rname) => format!(r##"ref_rule!(r#"{}"#)"##, rname),
        Expression::MetaExpr(me) => metaexpr2code(me),
    }
}

fn metaexpr2code(me: &MetaExpr) -> String {
    use crate::parser::expression::MetaExpr::{Named, Transf2};
    use crate::parser::expression::{NamedExpr, Transf2Expr};
    match me {
        Named(NamedExpr { name, expr }) => format!("named!(\"{}\", {})", name, expr2code(expr)),
        Transf2(Transf2Expr {
            mexpr,
            transf2_rules,
        }) => transf2code(mexpr, transf2_rules),
    }
}

fn transf2code(expr: &MultiExpr, t2: &ReplTemplate) -> String {
    format!(
        "transf2!( and!( {} ) , t2rules!({}) )",
        mexpr2code(expr),
        transf2templ2code(t2)
    )
}

fn transf2templ2code(t: &ReplTemplate) -> String {
    use crate::parser::expression::ReplItem;
    t.0.iter().fold("".to_string(), |acc, i| {
        let code = match i {
            ReplItem::Text(t) => format!(r#"t2_text!("{}"), "#, t),
            ReplItem::ByPos(p) => format!(r#"t2_bypos!("{}"), "#, p),
            ReplItem::ByName(p) => format!(r#"t2_byname!("{}"), "#, p),
            ReplItem::ByNameOpt(p) => format!(r#"t2_byname_opt!("{}"), "#, p),
            ReplItem::Function(p) => format!(r#"t2_funct!("{}"), "#, p),
        };
        acc.iappend(&code)
    })
}

fn mexpr2code(mexpr: &expression::MultiExpr) -> String {
    mexpr
        .0
        .iter()
        .fold(String::new(), |acc, expr| match acc.len() {
            0 => expr2code(expr),
            _ => format!("{}, {}", acc, expr2code(expr)),
        })
}

fn atom2code(atom: &Atom) -> String {
    let replace_esc = |s: String| {
        s.replace("\n", r#"\n"#)
            .replace("\r", r#"\r"#)
            .replace("\t", r#"\t"#)
            .replace(r#"""#, r#"\""#)
    };

    match atom {
        Atom::Literal(s) => format!(r#"lit!("{}")"#, replace_esc(s.to_string())),
        Atom::Expected(s) => format!(r#"expected!("{}")"#, replace_esc(s.to_string())),
        Atom::Match(mrules) => match_rules2code(mrules),
        Atom::Dot => "dot!()".to_string(),
        Atom::Eof => "eof!()".to_string(),
    }
}

fn match_rules2code(mrules: &atom::MatchRules) -> String {
    fn bounds2code(acc: String, bounds: &[(char, char)]) -> String {
        match bounds.split_first() {
            Some(((f, t), rest)) => {
                format!(", from '{}', to '{}' {}", f, t, bounds2code(acc, rest))
            }
            None => acc,
        }
    }

    format!(
        r##"ematch!(chlist r#"{}"#  {})"##,
        &mrules.0,
        bounds2code(String::new(), &mrules.1)
    )
}

fn repeat2code(rep: &expression::RepInfo) -> String {
    "rep!(".to_owned()
        + &expr2code(&rep.expression)
        + ", "
        + &rep.min.0.to_string()
        + &match rep.max {
            Some(ref m) => format!(", {}", m.0),
            None => "".to_owned(),
        }
        + ")"
}
