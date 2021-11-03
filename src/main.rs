mod github_event;
mod helpers;

use crate::github_event::GithubActionPullRequestLabel;
use chrono::prelude::*;
use helpers::github_get_commits_in_pr;

use helpers::*;

fn match_pick_merge_label(labels: Vec<GithubActionPullRequestLabel>) -> Vec<String> {
  labels
    .iter()
    .filter(|label| label.name.starts_with("auto-pick/"))
    .map(|label| label.name.clone())
    .collect()
}

#[tokio::main]
async fn main() {
  let github_event = get_event_action();

  let matched_labels = match_pick_merge_label(github_event.pull_request.labels);

  println!("{:?}", matched_labels);

  if matched_labels.len() <= 0 {
    return;
  }
  git_setup();

  for label in matched_labels {
    let dest_branch = label.split("/").last().expect("Not match dest branch");

    pick_pr_to_dest_branch(dest_branch.to_string()).await;
  }
}

async fn pick_pr_to_dest_branch(dest_branch: String) {
  println!("start job");

  let github_event = get_event_action();

  println!("{:?}", github_event);

  let pr_number = github_event.number;

  println!("{:?}", pr_number);

  let new_branch_name = create_new_branch_by_commits(dest_branch.clone(), pr_number)
    .await
    .expect("Create new branch by commit is failed");

  println!("{:?}", new_branch_name);

  let pr_title = format!("chore: auto pick {} to {}", pr_number, dest_branch);
  let body = format!("Auto pick merge by #{}", pr_number);

  println!("{:?},{:?}", pr_title, body);

  let pull_request_id =
    github_open_pull_request(new_branch_name, dest_branch, pr_title, body).await;

  github_pull_request_push_comment(pull_request_id, "test".to_string()).await;
}

fn generate_new_branch_name(to_branch: String) -> String {
  let timestamp: i64 = Utc::now().timestamp();

  format!("bot/auto-pick-{}-{:?}", to_branch, timestamp)
}

async fn create_new_branch_by_commits(to_branch: String, pr_number: i64) -> Option<String> {
  let origin_to_branch_name = format!("origin/{}", to_branch);

  let new_branch_name = generate_new_branch_name(to_branch);

  git(
    [
      "switch",
      "-c",
      new_branch_name.as_str(),
      origin_to_branch_name.as_str(),
    ]
    .to_vec(),
  );
  println!("New branch name:{}", new_branch_name);

  let not_matched_hash = pick_commits(pr_number).await;

  if not_matched_hash.len() > 0 {
    return None;
  }

  git(["push", "-u", "origin", new_branch_name.as_str()].to_vec());

  Some(new_branch_name)
}

async fn pick_commits(pr_number: i64) -> Vec<String> {
  let mut not_matched_hash = Vec::new();
  let commits = github_get_commits_in_pr(pr_number).await;

  for commit_hash in commits {
    let output = git(["cherry-pick", commit_hash.as_str()].to_vec());

    match output {
      Some(_output) => {
        println!("Pick success Commit hash: {:?}", commit_hash);
      }
      None => {
        not_matched_hash.push(commit_hash);
      }
    }
  }

  not_matched_hash
}
