use std::path::PathBuf;

use clap::{ArgGroup, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "goto",
    version,
    about = "Save directory bookmarks and jump to them from your shell",
    after_help = "Run `goto init <shell>` to enable directory changes in the current shell.",
    group = ArgGroup::new("shortcut_action")
        .args(["save", "list", "remove", "key"])
        .multiple(false)
)]
pub struct Cli {
    /// Save a directory. Defaults to the current directory when PATH is omitted.
    #[arg(
        short = 's',
        long = "save",
        value_name = "PATH",
        num_args = 0..=1
    )]
    pub save: Option<Option<PathBuf>>,

    /// Name to use with --save.
    #[arg(long, value_name = "NAME", requires = "save")]
    pub name: Option<String>,

    /// List saved workplaces.
    #[arg(short = 'l', long = "list")]
    pub list: bool,

    /// Remove a saved workplace.
    #[arg(short = 'r', long = "remove", value_name = "NAME")]
    pub remove: Option<String>,

    /// Workplace name to resolve. Use `goto -- NAME` when NAME looks like an option or command.
    #[arg(value_name = "NAME")]
    pub key: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Save a directory.
    Save {
        /// Directory to save. Defaults to the current directory.
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        /// Explicit bookmark name. Defaults to the directory name.
        #[arg(long, value_name = "NAME")]
        name: Option<String>,
    },

    /// List saved workplaces.
    List,

    /// Remove a saved workplace.
    Remove {
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Print the integration code for a shell.
    Init {
        #[arg(value_enum)]
        shell: Shell,
    },

    /// Resolve a workplace for the generated shell integration.
    #[command(name = "__resolve", hide = true)]
    Resolve {
        #[arg(value_name = "NAME")]
        name: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Shell {
    #[value(name = "powershell")]
    PowerShell,
    Cmd,
    Bash,
    Zsh,
    Fish,
}
