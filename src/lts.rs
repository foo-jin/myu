use nom::{
    bytes::complete::{is_not, tag},
    character::complete::{digit1, space1},
    combinator::map_res,
    sequence::delimited,
    IResult,
};
use std::{
    collections::{BTreeSet, HashMap},
    str::FromStr,
};

pub type State = u16;
pub type Label = String;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Lts {
    states: BTreeSet<State>,
    labels: BTreeSet<Label>,
    trans: HashMap<(State, Label), Vec<State>>,
}

impl Lts {
    pub fn states(&self) -> &BTreeSet<State> {
        &self.states
    }

    pub fn transitions(&self) -> &HashMap<(State, Label), Vec<State>> {
        &self.trans
    }

    fn add_edge(&mut self, start: State, label: &str, end: State) {
        self.states.insert(start);
        self.states.insert(end);
        self.labels.insert(label.to_owned());
        self.trans
            .entry((start, label.to_owned()))
            .or_insert_with(Vec::new)
            .push(end);
    }
}

impl FromStr for Lts {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lts = Lts::default();
        let (s, (_first, n_transitions, n_states)) = aut_header(s).unwrap();
        lts.trans.reserve(n_transitions as usize);
        for l in s.trim().lines() {
            let (_, (start, label, end)) = aut_edge(l).unwrap();
            lts.add_edge(start, label, end);
        }
        Ok(lts)
    }
}

fn parse_int(s: &str) -> IResult<&str, u16> {
    map_res(digit1, str::parse::<u16>)(s)
}

fn aut_header(s: &str) -> IResult<&str, (State, u16, u16)> {
    let (s, _) = tag("des")(s)?;
    let (s, _) = space1(s)?;
    let (s, _) = tag("(")(s)?;
    let (s, first_state) = parse_int(s)?;
    let (s, _) = tag(",")(s)?;
    let (s, n_transitions) = parse_int(s)?;
    let (s, _) = tag(",")(s)?;
    let (s, n_states) = parse_int(s)?;
    let (s, _) = tag(")")(s)?;
    Ok((s, (first_state, n_transitions, n_states)))
}

fn aut_edge(s: &str) -> IResult<&str, (State, &str, State)> {
    let (s, _) = tag("(")(s)?;
    let (s, start) = parse_int(s)?;
    let (s, label) = delimited(tag(",\""), is_not("\""), tag("\","))(s)?;
    let (s, end) = parse_int(s)?;
    let (s, _) = tag(")")(s)?;
    Ok((s, (start, label, end)))
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
    }
}
