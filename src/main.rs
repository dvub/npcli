mod cli;
// naming is hard :(
mod create;

use anyhow::Result;
use clap::Parser;
use cli::*;

use create::create_project;
// use nih_plug_xtask::build;
// use nih_plug_xtask::bundle;

// TODO:
// - add more comments - WIP
// - add documentation - WIP
// - finish bundle/bundle-universal
// - finish new() - DONE!
//      - just have to refactor now

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::New {
            skip_build,
            defaults,
            name,
        } => create_project(name, defaults, skip_build)?,
        /*
        Commands::Bundle {
            packages,
            other_args,
        } => {
            // chdir_workspace_root()?;
            let cargo_metadata = cargo_metadata::MetadataCommand::new()
                .manifest_path("./Cargo.toml")
                .exec()
                .unwrap(); // TODO
            let target_dir = cargo_metadata.target_directory.as_std_path();

            build(&packages, &other_args)?;

            bundle(target_dir, &packages[0], &other_args, false)?;
            for package in packages.into_iter().skip(1) {
                bundle(target_dir, &package, &other_args, false)?;
            }
        }
        */
        _ => todo!(),
    };

    Ok(())
}
