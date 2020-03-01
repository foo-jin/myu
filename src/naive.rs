use super::*;
use lts::Lts;
use mu_calculus as mc;
use std::collections::{BTreeSet, HashMap};

pub fn eval(lts: &Lts, f: &mc::Formula) -> BTreeSet<lts::State> {
    let mut env = HashMap::new();
    eval_inner(lts, f, &mut env)
}

fn eval_inner(
    lts: &Lts,
    f: &mc::Formula,
    env: &mut HashMap<mc::VarName, BTreeSet<lts::State>>,
) -> BTreeSet<lts::State> {
    use mc::Formula::*;

    match f {
        // Var { name } => env.get(&name).cloned().unwrap_or_else(BTreeSet::new),
        Var { name } => env[&name].clone(),
        True => lts.states().clone(),
        False => BTreeSet::new(),
        And { f1, f2 } => eval_inner(lts, f1, env)
            .intersection(&eval_inner(lts, f2, env))
            .cloned()
            .collect(),
        Or { f1, f2 } => eval_inner(lts, f1, env)
            .union(&eval_inner(lts, f2, env))
            .cloned()
            .collect(),
        Diamond { step, f } => {
            let sat = eval_inner(lts, f, env);
            lts.states()
                .iter()
                .cloned()
                .map(|s| {
                    lts.transitions()
                        .get(&(s, step.to_owned()))
                        .cloned()
                        .map(|ts| (s, ts))
                        .unwrap_or((s, vec![]))
                })
                .filter(|(_s, ts)| ts.iter().any(|t| sat.contains(t)))
                .map(|(s, _ts)| s)
                .collect()
        }
        Box { step, f } => {
            let sat = eval_inner(lts, f, env);
            lts.states()
                .iter()
                .cloned()
                .map(|s| {
                    lts.transitions()
                        .get(&(s, step.to_owned()))
                        .cloned()
                        .map(|ts| (s, ts))
                        .unwrap_or((s, vec![]))
                })
                .filter(|(_s, ts)| ts.iter().all(|t| sat.contains(t)))
                .map(|(s, _ts)| s)
                .collect()
        }
        Mu { var, f } => {
            let _ = env.insert(*var, BTreeSet::new());
            loop {
                let new = eval_inner(lts, f, env);
                let prev = env.insert(*var, new).unwrap();
                if prev == env[var] {
                    break prev;
                }
            }
        }
        Nu { var, f } => {
            let _ = env.insert(*var, lts.states().to_owned());
            loop {
                let new = eval_inner(lts, f, env);
                let prev = env.insert(*var, new).unwrap();
                if prev == env[var] {
                    break prev;
                }
            }
        }
    }
}

generate_tests!();
