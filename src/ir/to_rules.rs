use crate::ir::{Command, Error, Ir};
use crate::parser::expression::{
    Expression, MetaExpr, MultiExpr, NamedExpr, RepInfo, ReplItem, ReplTemplate, SetOfRules,
    Transf2Expr,
};
use idata::cont::IVec;

impl Ir {
    /// get rules from an Ir code
    pub(crate) fn get_rules(self) -> Result<SetOfRules, Error> {
        let (_ir, rules) = get_rule_rec(self, SetOfRules::empty())?;
        Ok(rules)
    }
}

fn get_rule_rec(ir: Ir, rules: SetOfRules) -> Result<(Ir, SetOfRules), Error> {
    if ir.peek() == Some(Command("EOP".to_string())) {
        Ok((ir, rules))
    } else {
        let (ir, rule) = get_rule(ir)?;
        let rules = rules.merge(rule);
        get_rule_rec(ir, rules)
    }
}

fn get_expr(ir: Ir) -> Result<(Ir, Expression), Error> {
    //  ATOM
    //  LIT
    //  literal
    let (ir, cmd) = ir.get()?;
    match cmd.0.as_ref() {
        "ATOM" => get_atom(ir),
        "AND" => {
            let (ir, mexpr) = get_mexpr(ir)?;
            Ok((ir, Expression::And(mexpr)))
        }
        "OR" => {
            let (ir, mexpr) = get_mexpr(ir)?;
            Ok((ir, Expression::Or(mexpr)))
        }
        "REPEAT" => get_repeat(ir),
        "MATCH" => get_match(ir),
        "NAMED" => get_named(ir),
        "NEGATE" => get_negate(ir),
        "EXPECTED" => get_expected(ir),
        "TRANSF2" => get_transf2(ir),
        "PEEK" => check_peek(ir),
        other => Err(Error(format!("unknown cmd reading expression <{}>", other))),
    }
}

fn get_transf2(ir: Ir) -> Result<(Ir, Expression), Error> {
    let (ir, repl_templ) = get_transf2_items_rec(ir, ReplTemplate::empty())?;
    let (ir, expr) = get_expr(ir)?;
    Ok((
        ir,
        Expression::MetaExpr(MetaExpr::Transf2(Transf2Expr {
            mexpr: MultiExpr(vec![expr]),
            transf2_rules: repl_templ,
        })),
    ))
}

fn get_transf2_item(ir: Ir) -> Result<(Ir, ReplItem), Error> {
    //  TEXT
    //  txt
    //
    if let Some(c) = ir.peek() {
        match c.0.as_ref() {
            "TEXT" => {
                let (ir, txt) = get_transf2_text(ir)?;
                Ok((ir, ReplItem::Text(txt)))
            }
            "NAMED" => {
                let (ir, txt) = get_transf2_named(ir)?;
                Ok((ir, ReplItem::ByName(txt)))
            }
            "NAMED_OPT" => {
                let (ir, otxt) = get_transf2_named(ir)?;
                Ok((ir, ReplItem::ByNameOpt(otxt)))
            }
            "POS" => {
                let (ir, pos) = get_transf2_pos(ir)?;
                Ok((ir, ReplItem::ByPos(pos)))
            }
            "FUNCT" => {
                let (ir, txt) = get_transf2_named(ir)?;
                Ok((ir, ReplItem::Function(txt)))
            }
            other => Err(Error(format!(
                "unxecpected command reading get_transf2 <{}>",
                other
            ))),
        }
    } else {
        Err(Error("Missing transf2 item".to_string()))
    }
}

fn get_transf2_items_rec(ir: Ir, repl_templ: ReplTemplate) -> Result<(Ir, ReplTemplate), Error> {
    if ir.peek() == Some(Command("EOTRANSF2".to_string())) {
        let (ir, _) = ir.get()?;
        Ok((ir, repl_templ))
    } else {
        let (ir, item) = get_transf2_item(ir)?;
        get_transf2_items_rec(ir, repl_templ.ipush(item))
    }
}

fn get_transf2_named(ir: Ir) -> Result<(Ir, String), Error> {
    let (ir, _) = ir.get()?;
    let (ir, named) = ir.get()?;
    Ok((ir, named.0))
}

fn get_transf2_pos(ir: Ir) -> Result<(Ir, usize), Error> {
    let (ir, _) = ir.get()?;
    let (ir, str_pos) = ir.get()?;
    let pos = str_pos
        .0
        .parse()
        .map_err(|e| Error(format!("Failed reading pos transformation... {:?} ", e)))?;
    Ok((ir, pos))
}

fn get_transf2_text(ir: Ir) -> Result<(Ir, String), Error> {
    let (ir, _) = ir.get()?;
    let (ir, txt) = ir.get()?;
    Ok((ir, txt.0))
}

fn get_expected(ir: Ir) -> Result<(Ir, Expression), Error> {
    //  <err message>

    let (ir, msg) = ir.get()?;
    let expr = expected!(msg.0);
    Ok((ir, expr))
}

fn get_negate(ir: Ir) -> Result<(Ir, Expression), Error> {
    //  <expr>

    let (ir, expr) = get_expr(ir)?;
    let expr = not!(expr);
    Ok((ir, expr))
}

fn check_peek(ir: Ir) -> Result<(Ir, Expression), Error> {
    //  <expr>

    let (ir, expr) = get_expr(ir)?;
    let expr = peek!(expr);
    Ok((ir, expr))
}

fn get_named(ir: Ir) -> Result<(Ir, Expression), Error> {
    //  name
    //  <expr>

    let (ir, n) = ir.get()?;
    let (ir, expr) = get_expr(ir)?;
    let expr = Expression::MetaExpr(MetaExpr::Named(NamedExpr {
        name: n.0,
        expr: Box::new(expr),
    }));
    Ok((ir, expr))
}

fn get_match(ir: Ir) -> Result<(Ir, Expression), Error> {
    //  CHARS
    //  ASDFASDF
    //  BETWEEN
    //  a
    //  b
    //  0
    //  9

    let (ir, chars) = get_match_chars(ir)?;
    let (ir, between) = get_match_between(ir)?;
    let amatch =
        crate::parser::atom::Atom::Match(crate::parser::atom::MatchRules::init(&chars, between));
    let expr = Expression::Simple(amatch);
    Ok((ir, expr))
}

fn get_match_chars(ir: Ir) -> Result<(Ir, String), Error> {
    if Some(Command("CHARS".to_string())) == ir.peek() {
        let (ir, _) = ir.get()?;
        let (ir, c) = ir.get()?;
        Ok((ir, c.0))
    } else {
        Ok((ir, "".to_string()))
    }
}

fn get_match_between(ir: Ir) -> Result<(Ir, Vec<(char, char)>), Error> {
    if Some(Command("BETW".to_string())) == ir.peek() {
        let (ir, _) = ir.get()?;
        let (ir, v) = get_between_rec(ir, vec![])?;
        Ok((ir, v))
    } else {
        Ok((ir, vec![]))
    }
}

fn get_between_rec(ir: Ir, v: Vec<(char, char)>) -> Result<(Ir, Vec<(char, char)>), Error> {
    if ir.peek() == Some(Command("EOBETW".to_string())) {
        let (ir, _) = ir.get()?;
        Ok((ir, v))
    } else {
        let (ir, ch1) = ir.get()?;
        let (ir, ch2) = ir.get()?;
        let fc = |s: String| {
            s.chars()
                .next()
                .ok_or_else(|| Error(format!("expected char received <{}>", s)))
        };

        let ch1 = fc(ch1.0)?;
        let ch2 = fc(ch2.0)?;

        let v = v.ipush((ch1, ch2));
        get_between_rec(ir, v)
    }
}

fn get_repeat(ir: Ir) -> Result<(Ir, Expression), Error> {
    //  1
    //  inf
    //  expr
    let (ir, min) = ir.get()?;
    let min = match min.0.parse::<usize>() {
        Ok(v) => Ok(v),
        Err(e) => Err(Error(format!("getting min size {}", e))),
    }?;
    let (ir, max) = ir.get()?;
    let max = if max.0 == "inf" {
        Ok(None)
    } else {
        match max.0.parse::<usize>() {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(Error(format!("getting max size {}", e))),
        }
    }?;
    let (ir, expr) = get_expr(ir)?;
    Ok((
        ir,
        Expression::Repeat(RepInfo::new(Box::new(expr), min, max)),
    ))
}

fn get_mexpr(ir: Ir) -> Result<(Ir, MultiExpr), Error> {
    get_mexpr_rec(ir, MultiExpr::new(vec![]))
}

fn get_mexpr_rec(ir: Ir, me: MultiExpr) -> Result<(Ir, MultiExpr), Error> {
    if ir.peek() == Some(Command("CLOSE_MEXPR".to_string())) {
        let (ir, _) = ir.get()?;
        Ok((ir, me))
    } else {
        let (ir, e) = get_expr(ir)?;
        let me = me.ipush(e);
        get_mexpr_rec(ir, me)
    }
}

fn get_atom(ir: Ir) -> Result<(Ir, Expression), Error> {
    //  LIT
    //  literal
    let (ir, cmd) = ir.get()?;
    match cmd.0.as_ref() {
        "LIT" => get_lit(ir),
        "RULREF" => get_rulref(ir),
        "DOT" => Ok((ir, dot!())),
        "EOF" => Ok((ir, eof!())),
        other => Err(Error(format!("unknown cmd reading atom <{}>", other))),
    }
}

fn get_rulref(ir: Ir) -> Result<(Ir, Expression), Error> {
    //  name
    let (ir, cmd) = ir.get()?;
    Ok((ir, ref_rule!(cmd.0)))
}

fn get_lit(ir: Ir) -> Result<(Ir, Expression), Error> {
    //  literal
    let (ir, cmd) = ir.get()?;
    Ok((ir, lit!(cmd.0)))
}

fn get_rule(ir: Ir) -> Result<(Ir, SetOfRules), Error> {
    //  RULE
    //  name
    //  DESCR
    //  description
    //  ATOM
    //  LIT
    //  literal
    let ir = ir.consume("RULE")?;

    let (ir, name) = ir.get()?;

    let ir = ir.consume("DESCR")?;

    let (ir, descr) = ir.get()?;
    let descr = Command(descr.0.trim().to_owned());
    let descr = if !descr.0.is_empty() {
        Some(descr.0)
    } else {
        None
    };

    let (ir, expr) = get_expr(ir)?;

    Ok((ir, rules! { &name.0 => RuleInfo{expr, descr} }))
}
