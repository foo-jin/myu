use super::*;
use lts::Lts;
use mu_calculus as mc;
use std::collections::{BTreeSet, HashMap};

pub fn eval(lts: &Lts, f: &mc::Formula) -> BTreeSet<lts::State> {
    let mut env = HashMap::new();
    for g in f.subformulas() {
        match g {
            mc::Formula::Mu { var, .. } => {
                env.insert(*var, BTreeSet::new());
            },
            mc::Formula::Nu { var, .. } => {
                env.insert(*var, lts.states().clone());
            },
            _ => (),
        }
    }
    eval_inner(lts, f, None, &mut env)
}

fn eval_inner(
    lts: &Lts,
    f: &mc::Formula,
    prev_fixpoint: Option<&mc::Formula>,
    env: &mut HashMap<mc::VarName, BTreeSet<lts::State>>,
) -> BTreeSet<lts::State> {
    use mc::Formula::*;

    match f {
        Var { name } => env[&name].clone(),
        True => lts.states().clone(),
        False => BTreeSet::new(),
        And { f1, f2 } => eval_inner(lts, f1, prev_fixpoint, env)
            .intersection(&eval_inner(lts, f2, prev_fixpoint, env))
            .cloned()
            .collect(),
        Or { f1, f2 } => eval_inner(lts, f1, prev_fixpoint, env)
            .union(&eval_inner(lts, f2, prev_fixpoint, env))
            .cloned()
            .collect(),
        Diamond { step, f: g } => {
            let sat = eval_inner(lts, g, prev_fixpoint, env);
            lts.step_transitions(step)
                .filter(|(_s, ts)| ts.iter().any(|t| sat.contains(t)))
                .map(|(s, _ts)| s)
                .collect()
        },
        Box { step, f: g } => {
            let sat = eval_inner(lts, g, prev_fixpoint, env);
            lts.step_transitions(step)
                .filter(|(_s, ts)| ts.iter().all(|t| sat.contains(t)))
                .map(|(s, _ts)| s)
                .collect()
        },
        Mu { var, f: g } => {
            if let Some(Nu { .. }) = prev_fixpoint {
                reset_fixpoints(lts, f, env);
            }
            loop {
                super::ITERATIONS.fetch_add(1, Ordering::SeqCst);
                let new = eval_inner(lts, g, Some(f), env);
                let prev = env.insert(*var, new).unwrap();
                if prev == env[var] {
                    break prev;
                }
            }
        },
        Nu { var, f: g } => {
            if let Some(Mu { .. }) = prev_fixpoint {
                reset_fixpoints(lts, f, env);
            }
            loop {
                super::ITERATIONS.fetch_add(1, Ordering::SeqCst);
                let new = eval_inner(lts, g, Some(f), env);
                let prev = env.insert(*var, new).unwrap();
                if prev == env[var] {
                    break prev;
                }
            }
        },
    }
}

fn reset_fixpoints(
    lts: &Lts,
    f: &mc::Formula,
    env: &mut HashMap<mc::VarName, BTreeSet<lts::State>>,
) {
    use mc::Formula::*;
    match f {
        Mu { .. } => f.subformulas().for_each(|g| match g {
            Mu { var, .. } if g.is_open() => {
                env.insert(*var, BTreeSet::new());
            },
            _ => (),
        }),
        Nu { .. } => f.subformulas().for_each(|g| match g {
            Nu { var, .. } if g.is_open() => {
                env.insert(*var, lts.states().clone());
            },
            _ => (),
        }),
        _ => panic!("Cannot reset non-fixpoint operators."),
    }
}

generate_tests!();
