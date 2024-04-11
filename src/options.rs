use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::PathBuf};

fn default_path() -> PathBuf {
    ".".into()
}

fn default_unstaged_string() -> String {
    "*".into()
}

fn default_staged_string() -> String {
    "+".into()
}

fn default_fetch_icon() -> String {
    "\u{f0954} ".into()
}

fn default_commit_ahead() -> String {
    "\u{ea9a}".into()
}
fn default_commit_behind() -> String {
    "\u{eaa1}".into()
}

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author = "loafey", version = "0.5", about = "
A tool to get the status of your git repos.
Designed to easily be integrated into prompts.", long_about = None)]
pub struct Options {
    /// The folder to check the git status of
    #[serde(default = "default_path")]
    #[arg(short = 'p', long = "path", value_name = "FILE", default_value = ".")]
    pub path: PathBuf,
    /// Show parentheses around the output
    #[serde(default)]
    #[arg(short = 'P', long = "parentheses", default_value = "false")]
    pub parentheses: bool,
    /// Show square brackets around the output
    #[serde(default)]
    #[arg(short = 'S', long = "square-brackets", default_value = "false")]
    pub square_brackets: bool,
    /// Show a custom string when a repository has unstaged changes.
    #[arg(
        short = 'u',
        long = "unstaged-string",
        value_name = "STRING",
        default_value = "*"
    )]
    #[serde(default = "default_unstaged_string")]
    pub unstaged_string: String,
    /// Show a custom string when a repository has staged changes.
    /// Only used when you use the `--sc` flag
    #[arg(
        short = 't',
        long = "staged-string",
        value_name = "STRING",
        default_value = "+"
    )]
    #[serde(default = "default_staged_string")]
    pub staged_string: String,
    /// separate the symbols for staged and unstaged changes.
    #[serde(default)]
    #[arg(long = "sc", value_name = "bool", default_value = "false")]
    pub separate_changes: bool,
    /// Show icons representative of your remote.
    #[serde(default)]
    #[arg(short = 'i', long = "icon", default_value = "false")]
    pub remote_icon: bool,
    /// Print errors to `stderr` instead of silently exiting.
    #[serde(default)]
    #[arg(short = 'E', long = "error", default_value = "false")]
    pub print_error: bool,
    /// Add custom icons for your own git hosts, alternatively override the built in-ones.

    /// Add input `-o "git@|<STRING>", to replace the icon for all `git@` remotes.
    /// Use the option multiple times for multiple icons, `-o "git@|<STRING>" -o "https://github.com|<STRING>"` etc.
    /// Optionally you can add three bytes after to add a color to the icon.
    #[serde(default)]
    #[arg(
        short = 'o',
        long = "icon-override",
        value_name = "STRING|STRING|U8,U8,U8?"
    )]
    pub icon_override: Vec<String>,
    /// Enables the use of custom icon colors.
    #[arg(short = 'c', long = "icon-color", default_value = "false")]
    #[serde(default)]
    pub icon_color: bool,
    /// Show arrows indicating commit status.
    #[arg(short = 'r', long = "commit-arrows", default_value = "false")]
    #[serde(default)]
    pub commit_arrow: bool,
    /// Reminds you to fetch after X minutes if you have not done so in X minutes.
    #[serde(default)]
    #[arg(short = 'f', long = "fetch-time", value_name = "UINT")]
    pub fetch_time: Option<u64>,
    /// Override the icon displayed to remind you to fetch
    #[arg(long = "fi", value_name = "STRING", default_value = "\u{f0954} ")]
    #[serde(default = "default_fetch_icon")]
    pub fetch_icon: String,

    /// Automatically fetch after X minutes has elapsed since last fetch/pull instead of just reminding you.
    /// Does nothing unless you use the `-f` flag.
    /// Warning! Git fetching is not know for being super fast, so be prepared for occasional slow downs!
    #[arg(long = "sf", value_name = "BOOL", default_value = "false")]
    #[serde(default)]
    pub should_fetch: bool,

    /// Override the commit behind arrow.
    #[arg(long = "commit-behind", default_value = "\u{ea9a}")]
    #[serde(default = "default_commit_behind")]
    pub commit_behind: String,
    /// Override the commit ahead arrow
    #[arg(long = "commit-ahead", default_value = "\u{eaa1}")]
    #[serde(default = "default_commit_ahead")]
    pub commit_ahead: String,

    /// Show a more detailed view
    #[arg(long = "di", default_value = "false")]
    #[serde(default)]
    pub detailed_info: bool,

    /// Show if you are in a nix shell. Looks for the IN_NIX_SHELL environment variable.
    #[arg(long = "nix", short = 'n', default_value = "false")]
    #[serde(default)]
    pub detect_nix: bool,

    /// The symbol to show if you are in a nix shell, defaults to "󱄅"
    #[arg(long = "nix-icon", default_value = "\u{f313} ")]
    #[serde(default)]
    pub nix_symbol: String,
}

#[allow(unused)]
fn get_options_file() -> Result<Result<Options, toml::de::Error>> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("grompt")?;
    let config_path = xdg_dirs.place_config_file("config.toml")?;
    let mut f = File::open(config_path)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(toml::from_str(&buf))
}

#[allow(unused)]
pub fn get_options() -> Options {
    match get_options_file() {
        Ok(Ok(options)) => options,
        Ok(Err(e)) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
        Err(_) => Options::parse(),
    }
}
