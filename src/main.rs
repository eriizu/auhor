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
    #[error("No authors in list")]
    NoAuthors,
}

enum Directory {
    GitRepo(PathBuf),
    Bare,
}

#[derive(Default)]
struct Report {
    removed: Vec<String>,
    added: Vec<String>,
    not_added: Vec<String>,
    not_removed: Vec<String>,
}

impl Report {
    fn added(&mut self, value: String) {
        self.added.push(value);
    }

    fn not_added(&mut self, value: String) {
        self.not_added.push(value);
    }

    fn removed(&mut self, value: String) {
        self.removed.push(value);
    }

    fn not_removed(&mut self, value: String) {
        self.not_removed.push(value);
    }
}

struct AuthorManager {
    directory: Directory,
    file: PathBuf,
    report: Report,
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.removed.is_empty() {
            writeln!(f, "{}", format!("-- {}", self.removed.join(", ")).red())?;
        }
        if !self.added.is_empty() {
            writeln!(f, "{}", format!("++ {}", self.added.join(", ")).green())?;
        }
        if !self.not_removed.is_empty() {
            writeln!(
                f,
                "did not remove {} (did not exist)",
                self.not_removed.join(", ")
            )?;
        }
        if !self.not_added.is_empty() {
            writeln!(
                f,
                "did not add {} (already existed)",
                self.not_added.join(", ")
            )?;
        }
        Ok(())
    }
}

fn main() {
    if let Err(err) = run("author.txt") {
        eprintln!("{}", format!("{err}").red());
    }
}

fn run(author_file_name: &str) -> Result<()> {
    let mut args = std::env::args();
    let program = args.next().unwrap_or_else(|| "author".to_string());
    let command = args.next();
    let mut author_manager =
        AuthorManager::find_author_file(std::env::current_dir()?, author_file_name)?;
    println!("operating {}", author_manager);

    let cmd_res = match command.as_deref() {
        None => author_manager.list_authors(),
        Some("add") => author_manager.add_authors(args.collect()),
        Some("remove") => {
            let removals: Vec<String> = args.collect();
            if removals.is_empty() {
                author_manager.prompt_remove()
            } else {
                author_manager.remove_authors(removals)
            }
        }
        Some(other) => Err(AuthorError::UnknownCommand(other.to_string())),
    };
    print!("{}", author_manager.report);
    if let Err(AuthorError::NoAuthors) = cmd_res {
        no_authors_message(&program);
        Ok(())
    } else {
        cmd_res
    }
}

impl std::fmt::Display for AuthorManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.directory {
            Directory::GitRepo(path) => write!(f, "in git repo {}", path.display())?,
            Directory::Bare => write!(f, "not in a repo, on lone {} file", self.file.display())?,
        };
        Ok(())
    }
}

impl AuthorManager {
    fn new(directory: Directory, file: PathBuf) -> Self {
        Self {
            directory,
            file,
            report: Report::default(),
        }
    }
    fn find_author_file(start: PathBuf, author_file_name: &str) -> Result<Self> {
        let mut current = start.as_path();
        loop {
            let directory = if current.join(".git").is_dir() {
                Some(Directory::GitRepo(current.to_path_buf()))
            } else if current.join(author_file_name).is_file() {
                Some(Directory::Bare)
            } else {
                None
            };
            if let Some(directory) = directory {
                return Ok(Self::new(directory, current.join(author_file_name)));
            }
            match current.parent() {
                Some(parent) => current = parent,
                None => return Err(AuthorError::NotInRepo),
            }
        }
    }

    fn list_authors(&self) -> Result<()> {
        let authors = read_authors(&self.file)?;
        if authors.is_empty() {
            return Err(AuthorError::NoAuthors);
        }
        for author in authors {
            println!("{author}");
        }
        Ok(())
    }

    fn add_authors(&mut self, logins: Vec<String>) -> Result<()> {
        if logins.is_empty() {
            return Err(AuthorError::MissingLogins);
        }
        let mut authors = read_authors(&self.file)?;
        for login in logins {
            if authors.insert(login.clone()) {
                self.report.added(login);
            } else {
                self.report.not_added(login);
            }
        }
        write_authors(&self.file, &authors)
    }

    fn remove_authors(&mut self, removals: Vec<String>) -> Result<()> {
        let mut authors = read_authors(&self.file)?;
        for removal in removals {
            if authors.remove(&removal) {
                self.report.removed(removal);
            } else {
                self.report.not_removed(removal);
            }
        }
        write_authors(&self.file, &authors)
    }

    fn prompt_remove(&mut self) -> Result<()> {
        let authors = read_authors(&self.file)?;
        if authors.is_empty() {
            return Ok(());
        }
        let options: Vec<String> = authors.iter().cloned().collect();
        let selections = inquire::MultiSelect::new("Select authors to remove", options).prompt()?;
        if selections.is_empty() {
            return Ok(());
        }
        self.remove_authors(selections)
    }
}

fn no_authors_message(program_name: &str) {
    let prefix = "no authors specified, run ".italic();
    let command = format!("{program_name} add login").bold().italic();
    let suffix = " to add them".italic();
    println!("{prefix}{command}{suffix}");
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
