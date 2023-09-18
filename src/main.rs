#![feature(let_chains)]
use anyhow::{Error, Result};
use clap::Parser;
use git2::{Repository, RepositoryOpenFlags};
use owo_colors::OwoColorize;
use std::path::PathBuf;

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
    #[arg(short = 's', long = "square-brackets", default_value = "false")]
    square_brackets: bool,
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

fn get_icon(repo: &Repository, icon_override: Vec<String>, icon_color: bool) -> Result<String> {
    let icons = create_icons(icon_override);
    let remotes = repo.remotes()?;
    // If you have multiple remotes this is probably wrong :)
    // Get a suitable icon from the remote
    let res = remotes
        .iter()
        .flatten()
        .filter_map(|s| repo.find_remote(s).ok())
        .map(|r| r.url().map(|s| s.to_string()).unwrap_or_default())
        .filter_map(|s| {
            let sub = icons.iter().find(|(start, _, _)| s.starts_with(start));
            if let Some((_, sub, c)) = sub {
                if icon_color &&  let Some([r, g, b]) = c {
            Some(sub.truecolor(*r, *g, *b).to_string())
        } else {
            Some(sub[..].into())
        }
            } else {
                None
            }
        })
        .next()
        .unwrap_or("\u{e702}".into());
    Ok(res)
}

fn format_status(options: Options) -> Result<String> {
    let path = options.path;
    let repo = git2::Repository::open_ext(
        path,
        RepositoryOpenFlags::CROSS_FS,
        &[] as &[&std::ffi::OsStr],
    )?;
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
    let remote_icon = options.remote_icon.then(|| {
        get_icon(&repo, options.icon_override, options.icon_color)
            .expect("Failed to get remote icon!")
    });
    let mut s = format!(
        "{}{}",
        if dirty { &options.dirty_string } else { "" },
        current_branch
    );
    if let Some(remote_icon) = remote_icon {
        s = format!("{remote_icon} {s}")
    }
    if options.parentheses {
        s = format!("({s})")
    }
    if options.square_brackets {
        s = format!("[{s}]")
    }

    Ok(s)
}
