#[macro_use]
mod tests;
mod improved;
mod lts;
mod mu_calculus;
mod naive;

use crate::{lts::Lts, mu_calculus as mc};
use ansi_term::{Colour, Style};
use atty::Stream;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};
use structopt::{clap::AppSettings, StructOpt};

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

    let bold = Style::new().bold();
    print_fancy(&format!("Checking formula ƒ ≔ {}", mcf_str.trim()), bold)?;

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

    writeln!(
        io::stdout(),
        "Checking required {} fixpoint iterations",
        ITERATIONS.load(Ordering::SeqCst)
    )?;

    if result.contains(&lts.init()) {
        print_fancy(
            &format!("Verdict: state {} satisfies ƒ", lts.init()),
            bold.fg(Colour::Green),
        )?
    } else {
        print_fancy(
            &format!("Verdict: state {} does not satisfy ƒ", lts.init()),
            bold.fg(Colour::Red),
        )?;
    }

    Ok(())
}

fn print_fancy(msg: &str, style: Style) -> io::Result<()> {
    if atty::is(Stream::Stdout) {
        writeln!(io::stdout(), "{}", style.paint(msg))
    } else {
        writeln!(io::stdout(), "{}", msg)
    }
}
