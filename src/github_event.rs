use crate::fetch_github_api_client;
use crate::github_api_event_repo_url;
use crate::github_env::get_github_env;
use crate::github_env::parse_env;
use std::{env, fs};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde()]
pub struct GithubActionPullRequestBranch {
  #[serde(rename(deserialize = "ref"))]
  pub _ref: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubActionPullRequestLabel {
  id: i64,
  pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubActionPullRequest {
  pub number: i64,
  pub base: GithubActionPullRequestBranch,
  pub labels: Vec<GithubActionPullRequestLabel>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubEventAction {
  action: String,
  pub number: String,
  pub pull_request: GithubActionPullRequest,
}

// ------ Response ------

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubGetCommitResponseItem {
  pub sha: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubCreatePullRequestResponse {
  pub number: String,
}

// ------ Impl ------

pub async fn get_event_action() -> GithubEventAction {
  if parse_env("TRAVIS") == "true" {
    return get_event_action_by_api().await;
  }

  get_event_action_by_gh_action()
}

pub fn get_event_action_by_gh_action() -> GithubEventAction {
  let github_event_path = env::var_os("GITHUB_EVENT_PATH").unwrap();
  let github_event_string = fs::read_to_string(github_event_path)
    .expect("Github action read event file to string is failed");

  serde_json::from_str::<GithubEventAction>(&github_event_string)
    .expect("Convert to github action event is failed")
}

pub async fn get_event_action_by_api() -> GithubEventAction {
  let env = get_github_env();
  let repo_url = github_api_event_repo_url();
  let client = fetch_github_api_client();
  let url = format!("{}/pulls/{}", repo_url, env.pull_request_number);

  client
    .get(url)
    .send()
    .await
    .expect("Failed to get pull request event")
    .json::<GithubEventAction>()
    .await
    .expect("Failed into json by pull request event")
}
