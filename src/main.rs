use anyhow::{Error, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author = "loafey", version = "0.1", about = "
A tool to get the status of your GIT repos.
Designed to easily be integrated into prompts.", long_about = None)]
struct Options {
    /// The folder to check the GIT status of
    #[arg(short = 'p', long = "path", value_name = "FILE", default_value = ".")]
    path: PathBuf,
    /// Show parentheses around the output
    #[arg(short = 'P', long = "parentheses", default_value = "false")]
    parentheses: bool,
    /// Show a custom string when a repository is dirty.
    #[arg(
        short = 'd',
        long = "dirty-string",
        value_name = "STRING",
        default_value = "*"
    )]
    dirty_string: String,
    /// Show icons representative of your remote.
    #[arg(short = 'i', long = "icon", default_value = "false")]
    remote_icon: bool,
    /// Print errors to `stderr` instead of silently exiting.
    #[arg(short = 'E', long = "error", default_value = "false")]
    print_error: bool,
}

fn main() {
    let options = Options::parse();
    let print_error = options.print_error;
    match format_status(options) {
        Err(e) => {
            if print_error {
                eprintln!("{e}");
            }
            std::process::exit(1);
        }
        Ok(res) => println!("{res}"),
    }
}

fn format_status(options: Options) -> Result<String> {
    let path = options.path;
    let substitues = [
        ("https://github.com/", "\u{e708}"),
        ("https://bitbucket.org", "\u{e703}"),
        ("https://gitlab.com", "\u{f296}"),
    ];

    let repo = git2::Repository::open(path)?;
    let dirty = repo
        .statuses(None)?
        .iter()
        .map(|s| s.status())
        .filter(|s| !s.is_ignored())
        .fold(0, |a, _| a + 1)
        > 0;
    let head = repo.head()?;
    let current_branch = head
        .shorthand()
        .ok_or(Error::msg("Failed to get branch name"))?;
    let remotes = repo.remotes()?;
    // If you have multiple remotes this is probably wrong :)
    let remote_icon = remotes
        .iter()
        .flatten()
        .filter_map(|s| repo.find_remote(s).ok())
        .map(|r| r.url().map(|s| s.to_string()).unwrap_or_default())
        .filter_map(|s| {
            let sub = substitues.iter().find(|(start, _)| s.starts_with(start));
            if let Some((_, sub)) = sub {
                Some(*sub)
            } else {
                None
            }
        })
        .next()
        .unwrap_or("\u{e702}");
    let mut s = format!(
        "{} {}{}",
        if options.remote_icon {
            &remote_icon
        } else {
            ""
        },
        if dirty { &options.dirty_string } else { "" },
        current_branch
    );
    if options.parentheses {
        s = format!("({s})")
    }

    Ok(s)
}
