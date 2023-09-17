use anyhow::Result;

fn main() -> Result<()> {
    let repo = git2::Repository::open(".")?;
    let statuses = repo
        .statuses(None)?
        .iter()
        .map(|s| s.status())
        .collect::<Vec<_>>();
    println!("Hello, world! {:#?}", statuses);

    Ok(())
}
