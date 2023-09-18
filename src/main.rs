#![feature(let_chains)]
use anyhow::{Error, Result};
use clap::Parser;
use git2::{Remote, Repository, RepositoryOpenFlags, Status};
use std::{fs::File, path::PathBuf, process::Command};

#[derive(Parser, Debug)]
#[command(author = "loafey", version = "0.1", about = "
A tool to get the status of your git repos.
Designed to easily be integrated into prompts.", long_about = None)]
struct Options {
    /// The folder to check the git status of
    #[arg(short = 'p', long = "path", value_name = "FILE", default_value = ".")]
    path: PathBuf,
    /// Show parentheses around the output
    #[arg(short = 'P', long = "parentheses", default_value = "false")]
    parentheses: bool,
    /// Show square brackets around the output
    #[arg(short = 'S', long = "square-brackets", default_value = "false")]
    square_brackets: bool,
    /// Show a custom string when a repository has unstaged changes.
    #[arg(
        short = 'u',
        long = "unstaged-string",
        value_name = "STRING",
        default_value = "*"
    )]
    unstaged_string: String,
    /// Show a custom string when a repository has staged changes.
    /// Only used when you use the `--sc` flag
    #[arg(
        short = 't',
        long = "staged-string",
        value_name = "STRING",
        default_value = "+"
    )]
    staged_string: String,
    /// Seperate the symbols for staged and unstaged changes.
    #[arg(long = "sc", value_name = "bool", default_value = "false")]
    seperate_changes: bool,
    /// Show icons representative of your remote.
    #[arg(short = 'i', long = "icon", default_value = "false")]
    remote_icon: bool,
    /// Print errors to `stderr` instead of silently exiting.
    #[arg(short = 'E', long = "error", default_value = "false")]
    print_error: bool,
    /// Add custom icons for your own git hosts, alternatively override the built in-ones.
    /// Add input `-o "git@|<STRING>", to replace the icon for all `git@` remotes.
    /// Use the option multiple times for multiple icons, `-o "git@|<STRING>" -o "https://github.com|<STRING>"` etc.
    /// Optionally you can add three bytes after to add a color to the icon.
    #[arg(
        short = 'o',
        long = "icon-override",
        value_name = "STRING|STRING|U8,U8,U8?"
    )]
    icon_override: Vec<String>,
    /// Enables the use of custom icon colors.
    #[arg(short = 'c', long = "icon-color", default_value = "false")]
    icon_color: bool,
    /// Show arrows indicating commit status.
    #[arg(short = 'r', long = "commit-arrows", default_value = "false")]
    commit_arrow: bool,
    /// Reminds you to fetch after X minutes if you have not done so in X minutes.
    #[arg(short = 'f', long = "fetch-time", value_name = "UINT")]
    fetch_time: Option<u64>,
    /// Reminds you to fetch after X minutes if you have not done so in X minutes.
    #[arg(long = "fi", value_name = "STRING", default_value = "\u{f0954} ")]
    fetch_icon: String,

    /// Automatically fetch after X minutes has elapsed since last fetch/pull instead of just reminding you.
    /// Does nothing unless you use the `-f` flag.
    /// Warning! Git fetching is not know for being super fast, so be prepared for occasional slow downs!
    #[arg(long = "sf", value_name = "BOOl", default_value = "false")]
    should_fetch: bool,

    /// Override the commit behind arrow.
    #[arg(long = "commit-behind", default_value = "\u{ea9a}")]
    commit_behind: String,
    /// Override the commit ahead arrow
    #[arg(long = "commit-ahead", default_value = "\u{eaa1}")]
    commit_ahead: String,
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

fn create_icons(icon_override: Vec<String>) -> Vec<(String, String, Option<[u8; 3]>)> {
    let icons = [
        (
            "https://github.com/".to_string(),
            "\u{e708}".to_string(),
            Some([255, 255, 255]),
        ),
        (
            "git@github.com".to_string(),
            "\u{e708}".to_string(),
            Some([255, 255, 255]),
        ),
        (
            "https://gitlab.com".to_string(),
            "\u{f296} ".to_string(),
            Some([252, 109, 38]),
        ),
        (
            "git@gitlab.com".to_string(),
            "\u{f296} ".to_string(),
            Some([252, 109, 38]),
        ),
        (
            "https://bitbucket.org".to_string(),
            "\u{e703}".to_string(),
            Some([38, 132, 255]),
        ),
        (
            "git@bitbucket.org".to_string(),
            "\u{e703}".to_string(),
            Some([38, 132, 255]),
        ),
    ];
    icon_override
        .into_iter()
        .filter_map(|s| {
            let mut splat = s.split('|');
            let uri = splat.next()?.to_string();
            let icon = splat.next()?.to_string();
            if let Some(color) = splat.next() {
                let mut color_splat = color.split(',');
                let r: u8 = color_splat.next()?.parse().unwrap();
                let g: u8 = color_splat.next()?.parse().unwrap();
                let b: u8 = color_splat.next()?.parse().unwrap();
                Some((uri, icon, Some([r, g, b])))
            } else {
                Some((uri, icon, None))
            }
        })
        .chain(icons)
        .collect::<Vec<_>>()
}

fn get_remote(repo: &Repository) -> Result<Remote<'_>> {
    // If you have multiple remotes this is probably wrong :)
    let remotes = repo.remotes()?;
    let mut remotes = remotes.iter().flatten();
    remotes
        .find_map(|s| repo.find_remote(s).ok())
        .ok_or(Error::msg("Failed to find any remotes!"))
}

fn get_icon(repo: &Repository, icon_override: Vec<String>, icon_color: bool) -> Result<String> {
    // Bloats up the list of available methods for types when using Rust Analyzer,
    // so importing it only here to avoid that.
    use owo_colors::OwoColorize;
    let icons = create_icons(icon_override);
    // Get a suitable icon from the remote
    let remote = get_remote(repo)?;
    let remote_uri = remote.url().unwrap_or_default();

    let icon = icons
        .iter()
        .find(|(start, _, _)| remote_uri.starts_with(start));
    let icon = if let Some((_, sub, c)) = icon {
        if icon_color && let Some([r,g,b]) = c {
            sub.truecolor(*r, *g, *b).to_string()
        } else {
            sub.clone()
        }
    } else {
        "\u{e702}".to_string()
    };
    Ok(icon)
}

type HasUnstagedChanges = bool;
type HasStagedChanges = bool;
fn repo_status(repo: &Repository) -> Result<(HasUnstagedChanges, HasStagedChanges)> {
    let statuses = repo.statuses(None)?;
    let status = statuses
        .iter()
        .map(|a| a.status())
        .reduce(|a, b| a | b)
        .unwrap_or(Status::empty());
    let unstaged_change = status.is_wt_deleted()
        || status.is_wt_modified()
        || status.is_wt_new()
        || status.is_wt_renamed()
        || status.is_wt_typechange();
    let staged_change = status.is_index_deleted()
        || status.is_index_modified()
        || status.is_index_new()
        || status.is_index_renamed()
        || status.is_index_typechange();
    Ok((unstaged_change, staged_change))
}

/// Determine if the current HEAD is ahead/behind its remote. The tuple
/// returned will be in the order ahead and then behind.
///
/// If the remote is not set or doesn't exist (like a detached HEAD),
/// (false, false) will be returned.
/// Yoinked from: https://github.com/rust-lang/git2-rs/issues/332#issuecomment-408453956
type AheadRemote = bool;
type BehindRemote = bool;
fn commit_status(repo: &Repository) -> (AheadRemote, BehindRemote) {
    let head = repo.revparse_single("HEAD").unwrap().id();
    if let Ok((upstream, _)) = repo.revparse_ext("@{u}") {
        return match repo.graph_ahead_behind(head, upstream.id()) {
            Ok((commits_ahead, commits_behind)) => (commits_ahead > 0, commits_behind > 0),
            Err(_) => (false, false),
        };
    }
    (false, false)
}

fn minutes_since_last(repo: &Repository) -> Result<u64> {
    let mut p = repo.path().to_owned();
    p.push("FETCH_HEAD");
    let f = File::open(p)?;
    let modified_time = f.metadata()?.modified()?.elapsed()?;
    Ok(modified_time.as_secs() / 60)
}

fn format_status(options: Options) -> Result<String> {
    let path = options.path;
    let repo = Repository::open_ext(
        path,
        RepositoryOpenFlags::CROSS_FS,
        &[] as &[&std::ffi::OsStr],
    )?;

    let mut fetch_reminder = None;
    if let Some(minutes) = options.fetch_time {
        let min_since_last = minutes_since_last(&repo)?;
        if min_since_last >= minutes {
            if options.should_fetch {
                // I could use `git2` but honestly easier this way.
                Command::new("git").arg("fetch").spawn()?.wait()?;
            } else {
                fetch_reminder = Some(&options.fetch_icon);
            }
        }
    }

    let (unstaged_changes, staged_changes) = repo_status(&repo)?;
    let mut s = match repo.head() {
        Ok(head) => {
            let current_branch = head
                .shorthand()
                .ok_or(Error::msg("Failed to get branch name"))?;
            let remote_icon = options.remote_icon.then(|| {
                get_icon(&repo, options.icon_override, options.icon_color)
                    .unwrap_or("\u{f071a}".to_string())
            });
            let mut changes = String::new();
            if options.seperate_changes {
                if unstaged_changes {
                    changes += &options.unstaged_string;
                }
                if staged_changes {
                    changes += &options.staged_string;
                }
            } else if unstaged_changes || staged_changes {
                changes += &options.unstaged_string;
            }
            let mut s = format!("{changes}{current_branch}");
            if let Some(remote_icon) = remote_icon {
                s = format!("{remote_icon} {s}")
            }
            if let Some(fetch_reminder) = fetch_reminder {
                s = format!("{fetch_reminder} {s}")
            }
            if options.commit_arrow {
                let (is_ahead, is_behind) = commit_status(&repo);
                if is_ahead && is_behind {
                    s = format!("{s} {}/{}", options.commit_ahead, options.commit_behind)
                } else if is_ahead {
                    s = format!("{s} {}", options.commit_ahead)
                } else if is_behind {
                    s = format!("{s} {}", options.commit_behind)
                }
            }
            s
        }
        Err(_) => "no head".to_string(),
    };

    if options.parentheses {
        s = format!("({s})")
    }
    if options.square_brackets {
        s = format!("[{s}]")
    }

    Ok(s)
}
