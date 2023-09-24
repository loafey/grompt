#![feature(let_chains)]
use anyhow::{Error, Result};
use git2::{Remote, Repository, RepositoryOpenFlags, Status};
use options::{get_options, Options};
use std::{fs::File, process::Command};
mod options;

fn main() {
    let options = get_options();
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
fn repo_status_bin(repo: &Repository) -> Result<(HasUnstagedChanges, HasStagedChanges)> {
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

fn repo_status(repo: &Repository) -> Result<(usize, usize)> {
    let statuses = repo.statuses(None)?;
    let (mut unstaged, mut staged) = (0, 0);
    for status in statuses.iter().map(|a| a.status()) {
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
        if unstaged_change {
            unstaged += 1
        }
        if staged_change {
            staged += 1;
        }
    }

    Ok((unstaged, staged))
}

/// Determine if the current HEAD is ahead/behind its remote. The tuple
/// returned will be in the order ahead and then behind.
///
/// If the remote is not set or doesn't exist (like a detached HEAD),
/// (false, false) will be returned.
/// Yoinked from: https://github.com/rust-lang/git2-rs/issues/332#issuecomment-408453956
type AheadRemote = usize;
type BehindRemote = usize;
fn commit_status(repo: &Repository) -> (AheadRemote, BehindRemote) {
    let head = repo.revparse_single("HEAD").unwrap().id();
    if let Ok((upstream, _)) = repo.revparse_ext("@{u}") {
        return match repo.graph_ahead_behind(head, upstream.id()) {
            Ok((commits_ahead, commits_behind)) => (commits_ahead, commits_behind),
            Err(_) => (0, 0),
        };
    }
    (0, 0)
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
        if let Ok(min_since_last) = minutes_since_last(&repo) {
            if min_since_last >= minutes {
                if options.should_fetch {
                    // I could use `git2` but honestly easier this way.
                    Command::new("git").arg("fetch").spawn()?.wait()?;
                } else {
                    fetch_reminder = Some(&options.fetch_icon);
                }
            }
        }
    }

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
            if !options.detailed_info {
                let (unstaged_changes, staged_changes) = repo_status_bin(&repo)?;
                if options.separate_changes {
                    if unstaged_changes {
                        changes += &options.unstaged_string;
                    }
                    if staged_changes {
                        changes += &options.staged_string;
                    }
                } else if unstaged_changes || staged_changes {
                    changes += &options.unstaged_string;
                }
            } else {
                let (unstaged_changes, staged_changes) = repo_status(&repo)?;

                if unstaged_changes > 0 && staged_changes > 0 {
                    changes += &format!(
                        ", {unstaged_changes}{}, {staged_changes}{}",
                        options.unstaged_string, options.staged_string,
                    );
                } else if unstaged_changes > 0 {
                    changes += &format!(", {unstaged_changes}{}", options.unstaged_string);
                } else if staged_changes > 0 {
                    changes += &format!(", {staged_changes}{}", options.staged_string);
                }
            }
            let mut s = if options.detailed_info {
                format!("{current_branch}{changes}")
            } else {
                format!("{changes}{current_branch}")
            };
            if let Some(remote_icon) = remote_icon {
                s = format!("{remote_icon} {s}")
            }
            if let Some(fetch_reminder) = fetch_reminder {
                s = format!("{fetch_reminder} {s}")
            }
            if options.commit_arrow {
                let (is_ahead, is_behind) = commit_status(&repo);
                if !options.detailed_info {
                    if is_ahead > 0 && is_behind > 0 {
                        s = format!("{s} {}/{}", options.commit_ahead, options.commit_behind)
                    } else if is_ahead > 0 {
                        s = format!("{s} {}", options.commit_ahead)
                    } else if is_behind > 0 {
                        s = format!("{s} {}", options.commit_behind)
                    }
                } else if is_ahead > 0 && is_behind > 0 {
                    s = format!(
                        "{s}, {}{is_ahead}/{}{is_behind}",
                        options.commit_ahead, options.commit_behind
                    )
                } else if is_ahead > 0 {
                    s = format!("{s}, {} {is_ahead}", options.commit_ahead)
                } else if is_behind > 0 {
                    s = format!("{s}, {} {is_behind}", options.commit_behind)
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
