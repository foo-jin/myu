use crate::mu_calculus::Formula;
use combine::{
    between, choice,
    error::ParseError,
    parser,
    parser::{
        char::{char, newline, space, spaces, string, upper},
        regex::find,
        repeat::skip_until,
    },
    skip_many1,
    stream::RangeStream,
    Parser,
};
use regex::Regex;

parser! {
    pub fn formula['a, I]()(I) -> Formula
    where [I: RangeStream<Token=char, Range=&'a str> + 'a,
       I::Error: ParseError<I::Token, I::Range, I::Position>,]
    {
    formula_()
    }
}

fn formula_<'a, I>() -> impl Parser<I, Output = Formula> + 'a
where
    I: RangeStream<Token = char, Range = &'a str> + 'a,
    I::Error: ParseError<I::Token, I::Range, I::Position>,
{
    let true_lit = string("true").map(|_| Formula::True);
    let false_lit = string("false").map(|_| Formula::False);
    let var = upper().map(|c| Formula::Var { name: c });
    let boolean_op = between(
        char('('),
        char(')'),
        (formula(), string("&&").or(string("||")), formula()),
    )
    .map(|(f1, op, f2)| match op {
        "&&" => Formula::And { f1: Box::new(f1), f2: Box::new(f2) },
        "||" => Formula::Or { f1: Box::new(f1), f2: Box::new(f2) },
        _ => unreachable!(),
    });
    let action = Regex::new(r"^[a-z][a-z0-9_]*").unwrap();
    let modal = |open, close| {
        between(char(open), char(close), find(action.clone())).and(formula())
    };
    let diamond_modal = modal('<', '>').map(|(step, f): (&'a str, Formula)| {
        Formula::Diamond { step: step.to_owned(), f: Box::new(f) }
    });
    let box_modal = modal('[', ']').map(|(step, f): (&'a str, Formula)| {
        Formula::Box { step: step.to_owned(), f: Box::new(f) }
    });
    let fixpoint = |sigma| {
        (
            string(sigma).skip(skip_many1(space())),
            upper().skip(spaces()),
            char('.'),
            formula(),
        )
    };
    let mu = fixpoint("mu")
        .map(|(_, var, _, g)| Formula::Mu { var, f: Box::new(g) });
    let nu = fixpoint("nu")
        .map(|(_, var, _, g)| Formula::Nu { var, f: Box::new(g) });
    let comment =
        char('%').and(skip_until(newline())).and(formula()).map(|(_, f)| f);

    between(
        spaces(),
        spaces(),
        choice((
            true_lit,
            false_lit,
            var,
            boolean_op,
            diamond_modal,
            box_modal,
            mu,
            nu,
            comment,
        )),
    )
}
