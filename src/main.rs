#[macro_use]
mod tests;
mod improved;
mod lts;
mod mu_calculus;
mod naive;

use crate::{lts::Lts, mu_calculus as mc};
use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};
use structopt::StructOpt;

static ITERATIONS: AtomicU32 = AtomicU32::new(0);

/// A model-checker for Labeled Transition Systems using a subset of the modal μ-calculus.
#[derive(StructOpt)]
struct Args {
    /// File specifying the LTS in aldebaran format
    lts: PathBuf,
    /// File specifying the formula to be checked in modal μ-calculus.
    mcf: PathBuf,
    /// Use naive algorithm instead of the Emerson-Lei algorithm
    #[structopt(long)]
    naive: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::from_args();
    let mut lts_file = File::open(args.lts)?;
    let mut lts = String::new();
    lts_file.read_to_string(&mut lts)?;

    let mut mcf_file = File::open(args.mcf)?;
    let mut mcf_str = String::new();
    mcf_file.read_to_string(&mut mcf_str)?;

    let lts = lts.parse::<Lts>().unwrap();
    let mcf = mcf_str.parse::<mc::Formula>()?;

    println!("Checking formula f ≔ {}", mcf_str.trim());

    println!("");
    println!(" ND(f) = {}", mcf.nesting_depth());
    println!(" AD(f) = {}", mcf.alternation_depth());
    println!("dAD(f) = {}", mcf.dependent_ad());
    println!("");

    let result = if args.naive {
        naive::eval(&lts, &mcf)
    } else {
        improved::eval(&lts, &mcf)
    };

    if result.contains(&lts.init()) {
        println!("Verdict: state {} satisfies f", lts.init());
    } else {
        println!("Verdict: state {} does not satisfy f", lts.init());
    }

    println!(
        "Checking required {} fixpoint iterations",
        ITERATIONS.load(Ordering::SeqCst)
    );

    Ok(())
}
