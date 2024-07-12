use clap::{command, Parser, Subcommand};

/// NIH-Plug CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new NIH-Plug project.
    New {
        /// Enabling this will skip all user input and simply create a project with defaults.
        #[arg(short, long, requires = "name")]
        defaults: bool,

        /// Optionally, provide the project (NOT plugin) name here. If you enabled the defaults flag, you MUST supply something here.
        #[arg(long)]
        name: Option<String>,

        /// Use this flag if you want to skip initial compilation.
        #[arg(short, long)]
        skip_build: bool,
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
