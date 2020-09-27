use std::rc::Rc;

use pest::{Parser, error::Error};
use pest_derive::Parser;
use crate::base_expr::{BExprBody, BExpr, BExprBodyExpr};
use crate::literals::Literal;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SchemeParser;

pub fn parse(s: &str) -> Result<BExprBody, Error<Rule>> {
    let mut pairs = SchemeParser::parse(Rule::program, s)?;

    let body = pairs.next().unwrap();
    Ok(build_body_from_expr(body))
}

fn build_bexpr_from_expr(pair: pest::iterators::Pair<Rule>) -> BExpr {
    match pair.as_rule() {
        Rule::expr => build_bexpr_from_expr(pair.into_inner().next().unwrap()),
        Rule::literal => BExpr::Lit(build_literal_from_expr(pair.into_inner().next().unwrap())),
        Rule::builtin => BExpr::BuiltinIdent(pair.as_str().to_owned()),
        Rule::if_form => build_if_from_expr(pair),
        Rule::set_form => build_set_from_expr(pair),
        Rule::let_form => build_let_from_expr(pair),
        Rule::lambda_form => build_lambda_from_expr(pair),
        Rule::app => build_app_from_expr(pair),
        Rule::variable => BExpr::Var(pair.as_str().to_owned()),
        e => unreachable!("{:?}", e),
    }
}

fn build_if_from_expr(pair: pest::iterators::Pair<Rule>) -> BExpr {
    let mut pair = pair.into_inner();
    let cond = pair.next().unwrap();
    let cond = build_bexpr_from_expr(cond);
    let ift = pair.next().unwrap();
    let ift = build_bexpr_from_expr(ift);
    let iff = pair.next().map(build_bexpr_from_expr)
                         .unwrap_or(BExpr::Lit(Literal::Void));
    BExpr::If(Rc::new(cond), Rc::new(ift), Rc::new(iff))
}

fn build_set_from_expr(pair: pest::iterators::Pair<Rule>) -> BExpr {
    let mut pair = pair.into_inner();
    let name = pair.next().unwrap().as_str().to_owned();
    let expr = pair.next().unwrap();
    let expr = build_bexpr_from_expr(expr);

    BExpr::Set(name, Rc::new(expr))
}

fn build_bodyexpr_from_expr(pair: pest::iterators::Pair<Rule>) -> BExprBodyExpr {
    match pair.as_rule() {
        Rule::define_form => {
            build_bexprbodyexpr_from_define(pair)
        }
        Rule::expr => {
            BExprBodyExpr::Expr(build_bexpr_from_expr(pair))
        }
        r => unreachable!("{:?}", r),
    }
}

fn build_body_from_expr(pair: pest::iterators::Pair<Rule>) -> BExprBody {
    let pair = pair.into_inner();
    let mut things = pair.map(build_bodyexpr_from_expr).collect::<Vec<_>>();
    let last = things.pop().unwrap();
    if let BExprBodyExpr::Expr(e) = last {
        BExprBody(things, Rc::new(e))
    } else {
        unreachable!()  // grammar should prevent this
    }
}

fn build_let_from_expr(pair: pest::iterators::Pair<Rule>) -> BExpr {
    let mut pair = pair.into_inner();
    let bindings = pair.next().unwrap().into_inner();
    let bindings = bindings.map(|pair| {
        let mut pair = pair.into_inner();
        let name = pair.next().unwrap().as_str().to_owned();
        let expr = pair.next().unwrap();
        let expr = build_bexpr_from_expr(expr);

        (name, expr)
    }).collect::<Vec<_>>();

    let body = pair.next().unwrap();
    let body = build_body_from_expr(body);

    BExpr::Let(bindings, body)
}

fn build_lambda_from_expr(pair: pest::iterators::Pair<Rule>) -> BExpr {
    let mut pair = pair.into_inner();
    let bindings = pair.next().unwrap().into_inner()
                                       .map(|pair| pair.as_str().to_owned())
                                       .collect::<Vec<_>>();
    let body = pair.next().unwrap();
    let body = build_body_from_expr(body);

    BExpr::Lam(bindings, body)
}

fn build_app_from_expr(pair: pest::iterators::Pair<Rule>) -> BExpr {
    let mut pair = pair.into_inner();
    let function = pair.next().unwrap();
    let function = build_bexpr_from_expr(function);

    let params = pair.map(build_bexpr_from_expr)
                     .collect::<Vec<_>>();

    BExpr::App(Rc::new(function), params)
}

fn build_literal_from_expr(pair: pest::iterators::Pair<Rule>) -> Literal {
    match pair.as_rule() {
        Rule::number => Literal::Int(pair.as_str().parse().unwrap()),
        Rule::quoted_string => Literal::String(pair.into_inner().next().unwrap().as_str().to_owned()),
        Rule::null => Literal::Void,
        _ => unreachable!(),
    }
}

fn build_bexprbodyexpr_from_define(pair: pest::iterators::Pair<Rule>) -> BExprBodyExpr {
    let mut pair = pair.into_inner();
    let name = pair.next().unwrap().as_str().to_owned();
    let expr = pair.next().unwrap();
    let expr = build_bexpr_from_expr(expr);

    BExprBodyExpr::Def(name, expr)
}
