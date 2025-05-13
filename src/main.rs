use clap::Parser;
use git2::{Error, Repository};
use serde::Serialize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Start of range (commit hash, tag, branch, or refspec)
    #[clap(long, short)]
    from: String,

    /// End of range (commit hash, tag, branch, or refspec)
    #[clap(long, short, default_value = "HEAD")]
    to: String,

    /// GitHub repository in the form owner/repo
    #[clap(long)]
    github_repo: String,

    /// GitHub API token
    #[clap(long)]
    github_token: String,

    /// Only include issues/PRs with this label (optional)
    #[arg(long)]
    label_filter: Option<String>,

    #[clap(long, short, default_value = ".")]
    directory: String,
}

#[derive(Serialize)]
struct Commit {
    summary: String,
    author: String,
    date: i64,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let repo = Repository::open(args.directory)?;
    let from = repo.revparse_single(&args.from)?.id();
    let to = repo.revparse_single(&args.to)?.id();

    let mut revwalk = repo.revwalk()?;
    revwalk.push(to)?;
    revwalk.hide(from)?;

    let mut commits = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let summary = commit.summary().unwrap_or("<no summary>").to_string();
        let author = commit.author().name().unwrap_or("<unknown>").to_string();
        let date = commit.time().seconds();
        commits.push(Commit {
            summary,
            author,
            date,
        });
    }

    println!("# Changelog - {}\n", args.from);
    for commit in &commits {
        println!(
            "- {} by {} ({})",
            commit.summary, commit.author, commit.date
        );
    }
    Ok(())
}
