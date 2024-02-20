use batmon::NumSamples;
use clap::{Arg, Parser};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
  /// Name of the person to greet
  #[arg(
    short,
    long,
    default_value_t = 1.to_string(),
    help = "measurement interval (in seconds)"
  )]
  pub delta: String,

  /// Number of times to greet
  #[arg(short, long, default_value_t = 60)]
  pub num_samples: NumSamples,
}

pub fn parse() -> Args {
  Args::parse()
}
