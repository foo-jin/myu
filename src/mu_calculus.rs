use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, anychar, space0, space1},
    combinator::{all_consuming, recognize, value, verify},
    sequence::{delimited, separated_pair},
    IResult,
};
use std::{collections::BTreeSet, str::FromStr};

pub type VarName = char;
type ParseResult<'a> = IResult<&'a str, Formula>;

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
        Subformulas {
            children: vec![self],
        }
    }

    pub fn is_open(&self) -> bool {
        let vars = self.variables();
        !vars.used.is_subset(&vars.declared)
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
        let (_, f) = all_consuming(parse_formula)(s.trim()).unwrap();
        Ok(f)
    }
}

fn parse_formula(s: &str) -> ParseResult {
    delimited(
        space0,
        alt((
            parse_true,
            parse_false,
            parse_var,
            parse_and,
            parse_or,
            parse_diamond,
            parse_box,
            parse_mu,
            parse_nu,
        )),
        space0,
    )(s)
}

fn parse_true(s: &str) -> ParseResult {
    value(Formula::True, tag("true"))(s)
}

fn parse_false(s: &str) -> ParseResult {
    value(Formula::False, tag("false"))(s)
}

fn parse_var(s: &str) -> ParseResult {
    let (s, c) = verify(recognize(anychar), |s: &str| {
        s.starts_with(|c: char| c.is_ascii_uppercase())
    })(s)?;
    Ok((
        s,
        Formula::Var {
            name: c.chars().next().unwrap(),
        },
    ))
}

fn binary_operator(op: &'static str) -> impl Fn(&str) -> IResult<&str, (Formula, Formula)> {
    move |s: &str| {
        let (s, _) = tag("(")(s)?;
        let (s, (f1, f2)) = separated_pair(parse_formula, tag(op), parse_formula)(s)?;
        let (s, _) = tag(")")(s)?;
        Ok((s, (f1, f2)))
    }
}

fn parse_and(s: &str) -> ParseResult {
    let (s, (f1, f2)) = binary_operator("&&")(s)?;
    let f = Formula::And {
        f1: Box::new(f1),
        f2: Box::new(f2),
    };
    Ok((s, f))
}

fn parse_or(s: &str) -> ParseResult {
    let (s, (f1, f2)) = binary_operator("||")(s)?;
    let f = Formula::Or {
        f1: Box::new(f1),
        f2: Box::new(f2),
    };
    Ok((s, f))
}

fn parse_diamond(s: &str) -> ParseResult {
    let (s, step) = delimited(tag("<"), alphanumeric1, tag(">"))(s)?;
    let (s, f1) = parse_formula(s)?;
    let f = Formula::Diamond {
        step: step.to_string(),
        f: Box::new(f1),
    };
    Ok((s, f))
}

fn parse_box(s: &str) -> ParseResult {
    let (s, step) = delimited(tag("["), alphanumeric1, tag("]"))(s)?;
    let (s, f1) = parse_formula(s)?;
    let f = Formula::Box {
        step: step.to_string(),
        f: Box::new(f1),
    };
    Ok((s, f))
}

fn fixpoint(sigma: &'static str) -> impl Fn(&str) -> IResult<&str, (VarName, Box<Formula>)> {
    move |s| {
        let (s, _) = tag(sigma)(s)?;
        let (s, _) = space1(s)?;
        let (s, var) = verify(recognize(anychar), |s: &str| {
            s.starts_with(|c: char| c.is_ascii_uppercase())
        })(s)?;
        let (s, _) = tag(".")(s)?;
        let (s, f1) = parse_formula(s)?;
        let var = var.chars().next().unwrap();
        Ok((s, (var, Box::new(f1))))
    }
}

fn parse_mu(s: &str) -> ParseResult {
    let (s, (var, f1)) = fixpoint("mu")(s)?;
    let f = Formula::Mu { var, f: f1 };
    Ok((s, f))
}

fn parse_nu(s: &str) -> ParseResult {
    let (s, (var, f1)) = fixpoint("nu")(s)?;
    let f = Formula::Nu { var, f: f1 };
    Ok((s, f))
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
}
