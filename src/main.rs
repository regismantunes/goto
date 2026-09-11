use std::{path::Path, process::ExitCode};

use anyhow::Result;
use clap::{CommandFactory, Parser};
use goto::{
    cli::{Cli, Command},
    config::{ConfigStore, Workplaces},
    shell::init_script,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return run_command(command);
    }
    if let Some(path) = cli.save {
        let store = ConfigStore::discover()?;
        return save(&store, path.as_deref(), cli.name.as_deref());
    }
    if cli.list {
        let store = ConfigStore::discover()?;
        return list(&store);
    }
    if let Some(name) = cli.remove {
        let store = ConfigStore::discover()?;
        return remove(&store, &name);
    }
    if let Some(name) = cli.key {
        let store = ConfigStore::discover()?;
        println!("{}", store.resolve(&name)?.display());
        return Ok(());
    }

    Cli::command().print_help()?;
    println!();
    Ok(())
}

fn run_command(command: Command) -> Result<()> {
    match command {
        Command::Init { shell } => {
            print!("{}", init_script(shell));
            Ok(())
        }
        Command::Save { path, name } => {
            let store = ConfigStore::discover()?;
            save(&store, path.as_deref(), name.as_deref())
        }
        Command::List => list(&ConfigStore::discover()?),
        Command::Remove { name } => remove(&ConfigStore::discover()?, &name),
        Command::Resolve { name } => {
            let store = ConfigStore::discover()?;
            println!("{}", store.resolve(&name)?.display());
            Ok(())
        }
    }
}

fn save(store: &ConfigStore, path: Option<&Path>, name: Option<&str>) -> Result<()> {
    let (name, path) = store.save_inferred(path, name)?;
    println!("Saved {name} -> {path}");
    Ok(())
}

fn list(store: &ConfigStore) -> Result<()> {
    let workplaces = store.load()?;
    if workplaces.is_empty() {
        println!("No workplaces saved.");
        return Ok(());
    }

    print_workplaces(&workplaces);
    Ok(())
}

fn print_workplaces(workplaces: &Workplaces) {
    let mut entries: Vec<_> = workplaces.iter().collect();
    entries.sort_by(|(left, _), (right, _)| {
        left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase())
    });
    let width = entries
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0);
    for (name, path) in entries {
        println!("{name:width$}  {path}");
    }
}

fn remove(store: &ConfigStore, name: &str) -> Result<()> {
    let (name, path) = store.remove(name)?;
    println!("Removed {name} -> {path}");
    Ok(())
}
