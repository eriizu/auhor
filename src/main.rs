use colored::Colorize as _;
use std::collections::BTreeSet;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use colored::Colorize as _;

type Result<T> = std::result::Result<T, AuthorError>;

#[derive(Debug, thiserror::Error)]
enum AuthorError {
    #[error("Not inside a git repository")]
    NotInRepo,
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    #[error("add requires at least one login")]
    MissingLogins,
    #[error(transparent)]
    Inquire(#[from] inquire::error::InquireError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let program = args.next().unwrap_or_else(|| "author".to_string());
    let command = args.next();
    let repo_root = find_repo_root(std::env::current_dir()?)?;
    let author_path = repo_root.join("author.txt");

    match command.as_deref() {
        None => list_authors(&author_path, &program)?,
        Some("add") => add_authors(&author_path, args.collect())?,
        Some("remove") => {
            let removals: Vec<String> = args.collect();
            if removals.is_empty() {
                prompt_remove(&author_path)?;
            } else {
                remove_authors(&author_path, removals)?;
            }
        }
        Some(other) => {
            return Err(AuthorError::UnknownCommand(other.to_string()));
        }
    }

    Ok(())
}

fn find_repo_root(start: PathBuf) -> Result<PathBuf> {
    let mut current = start.as_path();
    loop {
        if current.join(".git").is_dir() {
            return Ok(current.to_path_buf());
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return Err(AuthorError::NotInRepo),
        }
    }
}

fn list_authors(path: &Path, program: &str) -> Result<()> {
    let authors = read_authors(path)?;
    if authors.is_empty() {
        let prefix = "no authors specified, run ".italic();
        let command = format!("{program} add login").bold().italic();
        let suffix = " to add them".italic();
        println!("{prefix}{command}{suffix}");
        return Ok(());
    }
    for author in authors {
        println!("{author}");
    }
    Ok(())
}

fn add_authors(path: &Path, logins: Vec<String>) -> Result<()> {
    if logins.is_empty() {
        return Err(AuthorError::MissingLogins);
    }
    let mut authors = read_authors(path)?;
    for login in logins {
        authors.insert(login);
    }
    write_authors(path, &authors)
}

fn prompt_remove(path: &Path) -> Result<()> {
    let authors = read_authors(path)?;
    if authors.is_empty() {
        return Ok(());
    }
    let options: Vec<String> = authors.iter().cloned().collect();
    let selections = inquire::MultiSelect::new("Select authors to remove", options).prompt()?;
    if selections.is_empty() {
        return Ok(());
    }
    let selection_set: BTreeSet<String> = selections.into_iter().collect();
    let remaining: BTreeSet<String> = authors
        .into_iter()
        .filter(|author| !selection_set.contains(author))
        .collect();
    write_authors(path, &remaining)
}

fn remove_authors(path: &Path, removals: Vec<String>) -> Result<()> {
    let mut authors = read_authors(path)?;
    for removal in removals {
        authors.remove(&removal);
    }
    write_authors(path, &authors)
}

fn read_authors(path: &Path) -> Result<BTreeSet<String>> {
    if !path.exists() {
        return Ok(BTreeSet::new());
    }
    let contents = std::fs::read_to_string(path)?;
    Ok(contents.split_whitespace().map(str::to_string).collect())
}

fn write_authors(path: &Path, authors: &BTreeSet<String>) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::File::create(path)?;
    if !authors.is_empty() {
        let content = authors.iter().cloned().collect::<Vec<String>>().join(" ");
        writeln!(file, "{content}")?;
    }
    Ok(())
}
