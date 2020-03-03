use combine::{
    between, choice, eof,
    error::ParseError,
    parser,
    parser::{
        char::{char, space, spaces, string, upper},
        regex::find,
    },
    skip_many1,
    stream::{position, RangeStream},
    EasyParser, Parser,
};
use regex::Regex;
use std::{collections::BTreeSet, str::FromStr};

pub type VarName = char;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Formula {
    False,
    True,
    Var { name: VarName },
    And { f1: Box<Formula>, f2: Box<Formula> },
    Or { f1: Box<Formula>, f2: Box<Formula> },
    Diamond { step: String, f: Box<Formula> },
    Box { step: String, f: Box<Formula> },
    Mu { var: VarName, f: Box<Formula> },
    Nu { var: VarName, f: Box<Formula> },
}

#[derive(Clone, Debug)]
pub struct Subformulas<'a> {
    children: Vec<&'a Formula>,
}

#[derive(Clone, Debug, Default)]
struct Variables {
    declared: BTreeSet<VarName>,
    used: BTreeSet<VarName>,
}

impl Formula {
    pub fn subformulas(&self) -> Subformulas {
        Subformulas { children: vec![self] }
    }

    pub fn is_open(&self) -> bool {
        let vars = self.variables();
        !vars.used.is_subset(&vars.declared)
    }

    pub fn nesting_depth(&self) -> u16 {
        use Formula::*;
        match self {
            True | False | Var { .. } => 0,
            Box { f, .. } | Diamond { f, .. } => f.nesting_depth(),
            And { f1, f2 } | Or { f1, f2 } => {
                u16::max(f1.nesting_depth(), f2.nesting_depth())
            }
            Mu { f, .. } | Nu { f, .. } => 1 + f.nesting_depth(),
        }
    }

    pub fn alternation_depth(&self) -> u16 {
        use Formula::*;
        match self {
            True | False | Var { .. } => 0,
            Box { f, .. } | Diamond { f, .. } => f.alternation_depth(),
            And { f1, f2 } | Or { f1, f2 } => {
                u16::max(f1.alternation_depth(), f2.alternation_depth())
            }
            Mu { f, .. } => 1.max(f.alternation_depth()).max(
                1 + f
                    .subformulas()
                    .filter(|g| g.is_nu())
                    .map(|g| g.alternation_depth())
                    .max()
                    .unwrap_or(0),
            ),
            Nu { f, .. } => 1.max(f.alternation_depth()).max(
                1 + f
                    .subformulas()
                    .filter(|g| g.is_mu())
                    .map(|g| g.alternation_depth())
                    .max()
                    .unwrap_or(0),
            ),
        }
    }

    pub fn dependent_ad(&self) -> u16 {
        use Formula::*;
        match self {
            True | False | Var { .. } => 0,
            Box { f, .. } | Diamond { f, .. } => f.dependent_ad(),
            And { f1, f2 } | Or { f1, f2 } => {
                u16::max(f1.dependent_ad(), f2.dependent_ad())
            }
            Mu { var, f } => 1.max(f.dependent_ad()).max(
                1 + f
                    .subformulas()
                    .filter(|g| g.is_nu() && g.variables().used.contains(&var))
                    .map(|g| g.dependent_ad())
                    .max()
                    .unwrap_or(0),
            ),
            Nu { var, f } => 1.max(f.dependent_ad()).max(
                1 + f
                    .subformulas()
                    .filter(|g| g.is_mu() && g.variables().used.contains(&var))
                    .map(|g| g.dependent_ad())
                    .max()
                    .unwrap_or(0),
            ),
        }
    }

    pub fn is_mu(&self) -> bool {
        match self {
            Formula::Mu { .. } => true,
            _ => false,
        }
    }

    pub fn is_nu(&self) -> bool {
        match self {
            Formula::Nu { .. } => true,
            _ => false,
        }
    }

    fn variables(&self) -> Variables {
        use Formula::*;
        let mut vars = Variables::default();
        match self {
            Var { name } => {
                vars.used.insert(*name);
            }
            And { f1, f2 } | Or { f1, f2 } => {
                vars = f1.variables();
                vars.union(f2.variables());
            }
            Diamond { f, .. } | Box { f, .. } => vars = f.variables(),
            Mu { var, f } | Nu { var, f } => {
                vars = f.variables();
                vars.declared.insert(*var);
            }
            _ => (),
        }
        vars
    }
}

impl Variables {
    fn union(&mut self, mut other: Variables) {
        self.declared.append(&mut other.declared);
        self.used.append(&mut other.used);
    }
}

impl<'a> Iterator for Subformulas<'a> {
    type Item = &'a Formula;

    fn next(&mut self) -> Option<Self::Item> {
        use Formula::*;
        let item = self.children.pop();
        if let Some(f) = item {
            match f {
                And { f1, f2 } => self.children.extend_from_slice(&[f1, f2]),
                Or { f1, f2 } => self.children.extend_from_slice(&[f1, f2]),
                Box { f, .. } => self.children.push(f),
                Diamond { f, .. } => self.children.push(f),
                Mu { f, .. } => self.children.push(f),
                Nu { f, .. } => self.children.push(f),
                _ => (),
            }
        }
        item
    }
}

impl FromStr for Formula {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Formula, Self::Err> {
        let ((f, _), _) =
            formula().and(eof()).easy_parse(position::Stream::new(s)).unwrap();
        Ok(f)
    }
}

parser! {
    fn formula['a, I]()(I) -> Formula
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
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals() {
        let f = "false".parse::<Formula>();
        assert_eq!(f, Ok(Formula::False));

        let f = "true".parse::<Formula>();
        assert_eq!(f, Ok(Formula::True));
    }

    #[test]
    fn binary_operators() {
        let f = "(false &&  true)".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::And {
                f1: Box::new(Formula::False),
                f2: Box::new(Formula::True),
            }),
        );

        let f = "( false || (true &&true))".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::Or {
                f1: Box::new(Formula::False),
                f2: Box::new(Formula::And {
                    f1: Box::new(Formula::True),
                    f2: Box::new(Formula::True),
                }),
            }),
        );

        let f = "( ( false || false) && (true|| false))".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::And {
                f1: Box::new(Formula::Or {
                    f1: Box::new(Formula::False),
                    f2: Box::new(Formula::False),
                }),
                f2: Box::new(Formula::Or {
                    f1: Box::new(Formula::True),
                    f2: Box::new(Formula::False),
                }),
            }),
        );
    }

    #[test]
    fn modal_operators() {
        let f = "[tau]true".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::Box {
                step: "tau".to_string(),
                f: Box::new(Formula::True),
            })
        );

        let f = "<tau>false".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::Diamond {
                step: "tau".to_string(),
                f: Box::new(Formula::False),
            })
        );

        let f = "[tau]<tau>true".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::Box {
                step: "tau".to_string(),
                f: Box::new(Formula::Diamond {
                    step: "tau".to_string(),
                    f: Box::new(Formula::True)
                }),
            })
        );

        let f = "<tau>[tau]false".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::Diamond {
                step: "tau".to_string(),
                f: Box::new(Formula::Box {
                    step: "tau".to_string(),
                    f: Box::new(Formula::False)
                }),
            })
        );
    }

    #[test]
    fn fixpoints() {
        let f = "mu X. X".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::Mu {
                var: 'X',
                f: Box::new(Formula::Var { name: 'X' }),
            })
        );

        let f = "nu Y. Y".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::Nu {
                var: 'Y',
                f: Box::new(Formula::Var { name: 'Y' }),
            })
        );

        let f = "mu X. <tau>X".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::Mu {
                var: 'X',
                f: Box::new(Formula::Diamond {
                    step: "tau".to_string(),
                    f: Box::new(Formula::Var { name: 'X' })
                }),
            })
        );

        let f = "mu X. nu Y. (X || Y)".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::Mu {
                var: 'X',
                f: Box::new(Formula::Nu {
                    var: 'Y',
                    f: Box::new(Formula::Or {
                        f1: Box::new(Formula::Var { name: 'X' }),
                        f2: Box::new(Formula::Var { name: 'Y' })
                    })
                }),
            })
        );

        let f = "nu X. (X && mu Y. Y)".parse::<Formula>();
        assert_eq!(
            f,
            Ok(Formula::Nu {
                var: 'X',
                f: Box::new(Formula::And {
                    f1: Box::new(Formula::Var { name: 'X' }),
                    f2: Box::new(Formula::Mu {
                        var: 'Y',
                        f: Box::new(Formula::Var { name: 'Y' })
                    })
                })
            })
        );
    }

    #[test]
    fn depth_measures() {
        let f = "(mu X.nu Y.(X||Y)&& mu V. mu W. (V && mu Z.(true || Z)))"
            .parse::<Formula>()
            .unwrap();
        assert_eq!(f.nesting_depth(), 3);

        let f = "(mu X.nu Y.(X||Y)&& mu V. nu W. (V && mu Z.(true || Z)))"
            .parse::<Formula>()
            .unwrap();
        assert_eq!(f.alternation_depth(), 3);
        assert_eq!(f.dependent_ad(), 2);
    }
}
