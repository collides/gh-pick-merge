mod github_env;
mod github_event;
mod helpers;
mod types;

use crate::github_event::*;
use crate::helpers::*;
use types::*;

fn match_pick_merge_labels(labels: Vec<GithubActionPullRequestLabel>) -> Vec<String> {
  labels
    .iter()
    .filter(|label| label.name.starts_with("auto-pick/"))
    .map(|label| label.name.clone())
    .collect()
}

#[tokio::main]
async fn main() {
  let github_event = get_event_action().await;

  let matched_labels = match_pick_merge_labels(github_event.pull_request.labels);

  if matched_labels.len() <= 0 {
    return;
  }

  git_setup();

  for label in matched_labels {
    println!("dest branch: {}", label);

    let dest_branch = label.split("/").last().expect("Not match dest branch");

    pick_pr_to_dest_branch(github_event.number.clone(), dest_branch.to_string()).await;
  }
}

async fn pick_pr_to_dest_branch(pr_number: String, dest_branch: String) {
  println!("Start job pick to: {}", dest_branch);
 
  let create_branch_result =
    create_new_branch_by_commits(dest_branch.clone(), pr_number.clone()).await;

  let pr_title = format!("chore: auto pick #{} to {}", pr_number, dest_branch);
  let body = format!("Auto pick merge by #{}", pr_number);

  let pull_request_id = github_open_pull_request(
    create_branch_result.new_branch_name,
    dest_branch,
    pr_title,
    body,
  )
  .await;

  if create_branch_result.not_matched_hash.len() > 0 {
    github_pull_request_push_comment(
      pull_request_id,
      generate_pull_request_comment(create_branch_result.not_matched_hash),
    )
    .await;
  }

  println!("End job");
}

async fn create_new_branch_by_commits(
  to_branch: String,
  pr_number: String,
) -> CreateNewBranchResult {
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

  git(["push", "-u", "origin", new_branch_name.as_str()].to_vec());

  CreateNewBranchResult::new(new_branch_name, not_matched_hash)
}

async fn pick_commits(pr_number: String) -> Vec<String> {
  let mut not_matched_hash = Vec::new();
  let commits = github_get_commits_in_pr(pr_number.clone()).await;

  for commit_hash in commits {
    let output = git(["cherry-pick", commit_hash.as_str()].to_vec());

    match output {
      Some(output) => {
        println!(
          "Pick success Commit hash: {:?}, output: {:?}",
          commit_hash, output
        );
      }
      None => {
        not_matched_hash.push(commit_hash);
      }
    }
  }

  not_matched_hash
}
