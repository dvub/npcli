use clap::{command, Parser, Subcommand};

/// NIH-Plug CLI.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new NIH-Plug project through a step-by-step, interactive CLI.
    New {
        /// Use this flag if you want an initial compilation. This is the same as running bundle <package>.
        #[arg(short = 'b', long)]
        first_build: bool,
        /// If first-build is enabled, any other arguments supported by cargo may be supplied here.
        /// To pass these arguments, you must first include `--`. E.g. `-- --release`
        #[arg(raw = true, requires = "first_build")]
        other_args: Vec<String>,
    },
    /// Compile an existing NIH-Plug project
    Bundle {
        /// Package(s) to compile.
        packages: Vec<String>,

        /// Any other arguments supported by cargo, such as profile arguments (`--release`), may be supplied here.
        /// To pass these arguments, you must first include `--`. E.g. `-- --release --profile ...`
        #[arg(raw = true)]
        other_args: Vec<String>,
    },

    BundleUniversal {
        packages: String,
    },
}
