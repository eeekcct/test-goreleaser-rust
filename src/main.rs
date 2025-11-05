use clap::{Parser, Subcommand};
use git2::Repository;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "gitinfo")]
#[command(author, version, about = "Simple Git repository information tool", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show repository information
    Info {
        /// Path to the git repository
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// List branches
    Branches {
        /// Path to the git repository
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Show commit history
    Log {
        /// Path to the git repository
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Number of commits to show
        #[arg(short = 'n', default_value = "10")]
        count: usize,
    },
}

fn show_repo_info(path: &PathBuf) -> Result<(), git2::Error> {
    let repo = Repository::open(path)?;

    println!("=== Repository Information ===");
    println!("Path: {}", repo.path().display());

    // Get HEAD
    let head = repo.head()?;
    if let Some(name) = head.shorthand() {
        println!("Current branch: {}", name);
    }

    // Get remote
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            println!("Remote URL: {}", url);
        }
    }

    // Check if repository is bare
    println!("Bare repository: {}", repo.is_bare());

    // Count references
    let refs = repo.references()?;
    let ref_count = refs.count();
    println!("Number of references: {}", ref_count);

    Ok(())
}

fn list_branches(path: &PathBuf) -> Result<(), git2::Error> {
    let repo = Repository::open(path)?;

    println!("=== Branches ===");
    let branches = repo.branches(None)?;

    for branch in branches {
        let (branch, branch_type) = branch?;
        let name = branch.name()?.unwrap_or("<invalid utf-8>");
        let branch_type_str = match branch_type {
            git2::BranchType::Local => "local",
            git2::BranchType::Remote => "remote",
        };

        let prefix = if branch.is_head() { "* " } else { "  " };
        println!("{}{} ({})", prefix, name, branch_type_str);
    }

    Ok(())
}

fn show_log(path: &PathBuf, count: usize) -> Result<(), git2::Error> {
    let repo = Repository::open(path)?;

    println!("=== Commit History (last {} commits) ===", count);

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    for (i, oid) in revwalk.enumerate() {
        if i >= count {
            break;
        }

        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        println!("\nCommit: {}", oid);

        if let Some(author) = commit.author().name() {
            println!("Author: {}", author);
        }

        let time = commit.time();
        println!(
            "Date: {}",
            chrono::DateTime::from_timestamp(time.seconds(), 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Invalid date".to_string())
        );

        if let Some(message) = commit.message() {
            println!("Message: {}", message.lines().next().unwrap_or(""));
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    let result = match args.command {
        Commands::Info { path } => show_repo_info(&path),
        Commands::Branches { path } => list_branches(&path),
        Commands::Log { path, count } => show_log(&path, count),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
