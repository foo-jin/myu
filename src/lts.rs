use crate::MyuError;
use combine::{
    between, eof, from_str,
    parser::{
        char::{char, newline, space, spaces, string},
        range::take_while1,
    },
    skip_many, skip_many1,
    stream::position,
    EasyParser, Parser,
};
use std::{
    collections::{BTreeSet, HashMap},
    str::FromStr,
};

pub type State = u32;
pub type Label = String;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Lts {
    init: State,
    states: BTreeSet<State>,
    trans: HashMap<(State, Label), Vec<State>>,
}

impl Lts {
    pub fn states(&self) -> &BTreeSet<State> {
        &self.states
    }

    pub fn step_transitions<'a>(
        &'a self,
        step: &'a str,
    ) -> impl Iterator<Item = (State, Vec<State>)> + 'a {
        self.states().iter().cloned().map(move |s| {
            self.trans
                .get(&(s, step.to_owned()))
                .cloned()
                .map(|ts| (s, ts))
                .unwrap_or((s, vec![]))
        })
    }

    pub fn init(&self) -> State {
        self.init
    }

    fn add_edge(&mut self, start: State, label: &str, end: State) {
        self.states.insert(start);
        self.states.insert(end);
        self.trans
            .entry((start, label.to_owned()))
            .or_insert_with(Vec::new)
            .push(end);
    }
}

impl FromStr for Lts {
    type Err = MyuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let int = || from_str(take_while1(|c: char| c.is_digit(10)));
        let non_newline_spaces = || skip_many(char(' ').or(char('\t')));
        let aut_header = || {
            (
                string("des").skip(skip_many1(space())).skip(char('(')),
                int().skip(char(',')),
                int().skip(char(',')),
                int().skip(char(')')),
            )
        };
        let aut_edge = || {
            between(
                char('('),
                char(')'),
                (
                    int(),
                    between(
                        string(r#",""#),
                        string(r#"","#),
                        take_while1(|c: char| c != '"'),
                    ),
                    int(),
                ),
            )
        };

        let mut lts = Lts::default();
        let ((_, initial, n_transitions, _n_states), mut s) = aut_header()
            .easy_parse(position::Stream::new(s))
            .map_err(|e| MyuError::LtsParseError(e.to_string()))?;
        lts.init = initial;
        lts.trans.reserve(n_transitions as usize);

        while let Ok((_, mut rest)) =
            non_newline_spaces().and(newline()).skip(spaces()).easy_parse(s)
        {
            if eof().parse(&mut rest).is_ok() {
                break;
            }

            let ((start, label, end), rest) = aut_edge()
                .easy_parse(rest)
                .map_err(|e| MyuError::LtsParseError(e.to_string()))?;
            lts.add_edge(start, label, end);
            s = rest;
        }

        Ok(lts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let input = r#"des (0,12,10)
(0,"lock(p2, f2)",1)
(0,"lock(p1, f1)",2)
(1,"lock(p1, f1)",3)
(1,"lock(p2, f1)",4)
(2,"lock(p2, f2)",3)
(2,"lock(p1, f2)",5)
(4,"eat(p2)",6)
(5,"eat(p1)",7)
(6,"free(p2, f2)",8)
(7,"free(p1, f1)",9)
(8,"free(p2, f1)",0)
(9,"free(p1, f2)",0)"#;

        let mut expected = Lts::default();
        expected.add_edge(0, "lock(p2, f2)", 1);
        expected.add_edge(0, "lock(p1, f1)", 2);
        expected.add_edge(1, "lock(p1, f1)", 3);
        expected.add_edge(1, "lock(p2, f1)", 4);
        expected.add_edge(2, "lock(p2, f2)", 3);
        expected.add_edge(2, "lock(p1, f2)", 5);
        expected.add_edge(4, "eat(p2)", 6);
        expected.add_edge(5, "eat(p1)", 7);
        expected.add_edge(6, "free(p2, f2)", 8);
        expected.add_edge(7, "free(p1, f1)", 9);
        expected.add_edge(8, "free(p2, f1)", 0);
        expected.add_edge(9, "free(p1, f2)", 0);

        assert_eq!(input.parse::<Lts>(), Ok(expected));

        let input = "des (0,12,10)        \n\
(0,\"i\",1)
(0,\"i\",2)
(1,\"i\",3)
(1,\"i\",4)
(2,\"i\",5)
(2,\"i\",4)
(3,\"others\",6)
(5,\"plato\",7)
(6,\"i\",8)
(7,\"i\",9)
(8,\"i\",0)
(9,\"i\",0)
";

        let mut expected = Lts::default();
        expected.add_edge(0, "i", 1);
        expected.add_edge(0, "i", 2);
        expected.add_edge(1, "i", 3);
        expected.add_edge(1, "i", 4);
        expected.add_edge(2, "i", 5);
        expected.add_edge(2, "i", 4);
        expected.add_edge(3, "others", 6);
        expected.add_edge(5, "plato", 7);
        expected.add_edge(6, "i", 8);
        expected.add_edge(7, "i", 9);
        expected.add_edge(8, "i", 0);
        expected.add_edge(9, "i", 0);

        let result = input.parse::<Lts>();
        assert_eq!(result, Ok(expected));
    }
}
