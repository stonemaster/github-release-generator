use std::{
    collections::{BTreeMap, HashMap, HashSet},
    hash::{Hash, Hasher},
};

use chrono::DateTime;
use clap::Parser;
use git2::Repository;
use regex::Regex;
use reqwest::{
    blocking::Client,
    header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};

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
    filter_label: Option<String>,

    #[clap(long, short, default_value = ".")]
    directory: String,
}

#[derive(Serialize, Debug)]
struct Commit {
    id: String,
    summary: String,
    author: String,
    date: DateTime<chrono::Utc>,
    linked_issue: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct Issue {
    id: i64,
    number: i64,
    title: String,
    labels: Vec<IssueLabel>,
}

impl Issue {
    fn lower_case_labels(&self) -> HashSet<String> {
        self.labels
            .iter()
            .map(|label| label.name.to_lowercase())
            .collect()
    }
}

impl Hash for Issue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for Issue {}

impl PartialEq for Issue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Deserialize)]
struct IssueLabel {
    name: String,
}

fn generate_labeled_categories<'a>(
    filter_label: Option<String>,
    commits: &[Commit],
    issues: &'a HashMap<i64, Issue>,
) -> BTreeMap<String, HashSet<&'a Issue>> {
    let mut categories = BTreeMap::new();
    let mut category_mapping: HashMap<String, String> = HashMap::new();
    category_mapping.insert("bug".to_string(), "Fixed Bugs 🐛".to_string());
    let fallback_category = "Issues Closed";
    let filter_label = filter_label.map(|label| label.to_lowercase());
    for commit in commits {
        if let Some(issue) = &commit.linked_issue {
            if !issues.contains_key(issue) {
                continue;
            }

            let issue = issues.get(issue).unwrap();

            if filter_label.is_some()
                && !issue
                    .lower_case_labels()
                    .contains(filter_label.as_ref().unwrap())
            {
                continue;
            }

            for label in issue.lower_case_labels() {
                if let Some(category_title) = category_mapping.get(&label) {
                    categories
                        .entry(category_title.clone())
                        .or_insert_with(HashSet::new)
                        .insert(issue);
                } else {
                    categories
                        .entry(fallback_category.to_string())
                        .or_insert_with(HashSet::new)
                        .insert(issue);
                }
            }
        }
    }

    categories
}

/// Returns issue number (not internal ID) mapping to actual Issue data.
///
/// Using pagination to fetch all issues. Just filters for closed issues.
fn fetch_issues(github_repo: &str, token: &str) -> anyhow::Result<HashMap<i64, Issue>> {
    let url = format!(
        "https://api.github.com/repos/{}/issues?state=all",
        github_repo
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github.v3+json"),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );
    headers.insert(
        "User-Agent",
        HeaderValue::from_static("changelog-generator"),
    );

    let client = Client::new();
    let mut page = 1;
    let mut all_issues = Vec::new();

    eprintln!("Fetching issues from: {}", url);

    loop {
        let paged_url = format!("{}&per_page=100&page={}", url, page);
        let resp = client
            .get(&paged_url)
            .headers(headers.clone())
            .send()?
            .json::<Vec<Issue>>()?;

        if resp.is_empty() {
            break;
        }

        all_issues.extend(resp);
        page += 1;
    }

    Ok(all_issues
        .into_iter()
        .map(|issue| (issue.number, issue))
        .collect())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let repo = Repository::open(args.directory)?;
    let from = repo.revparse_single(&args.from)?.id();
    let to = repo.revparse_single(&args.to)?.id();

    let mut revwalk = repo.revwalk()?;
    revwalk.push(to)?;
    revwalk.hide(from)?;

    let issue_regex = Regex::new(r"#([0-9]+)")?;
    let mut commits = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let summary = commit.summary().unwrap_or("<no summary>").to_string();
        let author = commit.author().name().unwrap_or("<unknown>").to_string();
        let date = DateTime::from_timestamp(commit.time().seconds(), 0).unwrap();
        let linked_issue = if commit.message_raw().is_some() {
            issue_regex
                .captures(commit.message_raw().unwrap())
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().parse::<i64>().unwrap())
        } else {
            None
        };

        commits.push(Commit {
            id: oid.to_string()[..7].to_string(),
            summary,
            author,
            date,
            linked_issue,
        });
    }

    let issues = fetch_issues(&args.github_repo, &args.github_token)?;

    let categories = generate_labeled_categories(args.filter_label.clone(), &commits, &issues);

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    println!("# {} ({})", args.to, today);

    for title in categories.keys() {
        println!("\n## {}\n", title);
        for issue in &categories[title] {
            println!("- {} - #{}", issue.title, issue.number,);
        }
    }

    println!("\n## Full changelog of commits\n");

    for commit in &commits {
        println!(
            "- {} - {} by {} ({})",
            commit.id, commit.summary, commit.author, commit.date
        );
    }
    Ok(())
}
