#[macro_use]
mod tests;
mod improved;
mod lts;
mod mu_calculus;
mod naive;

use crate::{lts::Lts, mu_calculus as mc};
use ansi_term::Colour;
use anyhow::Context;
use atty::Stream;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};
use structopt::{clap::AppSettings, StructOpt};
use thiserror::Error;

static ITERATIONS: AtomicU32 = AtomicU32::new(0);

/// A model-checker for Labeled Transition Systems using a subset of the modal μ-calculus.
#[derive(StructOpt)]
#[structopt(global_settings(&[AppSettings::ColoredHelp]))]
struct Args {
    /// File specifying the LTS to be verified in aldebaran format
    lts: PathBuf,
    /// File specifying the formula to check in modal μ-calculus.
    mcf: PathBuf,
    /// Use naive algorithm instead of the Emerson-Lei algorithm
    #[structopt(long)]
    naive: bool,
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum MyuError {
    #[error("failed to parse μ-calculus formula: {0}")]
    McfParseError(String),
    #[error("failed to parse labeled transition system: {0}")]
    LtsParseError(String),
}

fn run() -> anyhow::Result<()> {
    let args = Args::from_args();
    let mut lts_file = File::open(&args.lts)
        .with_context(|| format!("failed to open {:#?}", &args.lts))?;
    let mut lts = String::new();
    lts_file
        .read_to_string(&mut lts)
        .with_context(|| format!("failed to read from {:#?}", &args.lts))?;

    let mut mcf_file = File::open(&args.mcf)
        .with_context(|| format!("failed to open {:#?}", &args.mcf))?;
    let mut mcf_str = String::new();
    mcf_file
        .read_to_string(&mut mcf_str)
        .with_context(|| format!("failed to read from {:#?}", &args.mcf))?;

    let lts = lts.parse::<Lts>()?;
    let mcf =
        mcf_str.parse::<mc::Formula>().map_err(MyuError::McfParseError)?;

    writeln!(io::stdout(), "Begin checking {:?}...", &args.mcf)?;
    writeln!(io::stdout(), "Let ƒ ≔ {}", mcf)?;

    writeln!(
        io::stdout(),
        "ND(ƒ) = {}    AD(ƒ) = {}    dAD(ƒ) = {}",
        mcf.nesting_depth(),
        mcf.alternation_depth(),
        mcf.dependent_ad()
    )?;

    let result = if args.naive {
        naive::eval(&lts, &mcf)
    } else {
        improved::eval(&lts, &mcf)
    };

    write!(io::stdout(), "ƒ = {{")?;
    let mut first = true;
    for s in result.iter().take(20) {
        if !first {
            write!(io::stdout(), ", ")?;
        }
        write!(io::stdout(), "{}", s)?;
        first = false;
    }
    if result.len() > 20 {
        write!(io::stdout(), ", and {} more", result.len() - 20)?;
    }
    writeln!(io::stdout(), "}}")?;

    writeln!(
        io::stdout(),
        "Checking required {} fixpoint iterations",
        ITERATIONS.load(Ordering::SeqCst)
    )?;

    if result.contains(&lts.init()) {
        print_fancy(
            &format!("Verdict: state {} satisfies ƒ", lts.init()),
            Colour::Green,
        )?
    } else {
        print_fancy(
            &format!("Verdict: state {} does not satisfy ƒ", lts.init()),
            Colour::Red,
        )?;
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        if atty::is(Stream::Stderr) {
            eprint!("{}: {:#}", Colour::Red.paint("[myu error]"), e)
        } else {
            eprint!("[myu error]: {:#}", e)
        }
    }
}

fn print_fancy(msg: &str, c: Colour) -> io::Result<()> {
    if atty::is(Stream::Stdout) {
        writeln!(io::stdout(), "{}", c.paint(msg))
    } else {
        writeln!(io::stdout(), "{}", msg)
    }
}
