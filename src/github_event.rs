use std::{env, fs};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde()]
pub struct GithubActionPullRequestBranch {
  #[serde(rename(deserialize = "ref"))]
  pub _ref: String
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
  pub number: i64,
  pub pull_request: GithubActionPullRequest,
}

// ------ Response ------

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubGetCommitResponseItem {
  pub sha: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubCreatePullRequestResponse {
  pub number: i64,
}

// ------ Impl ------

pub fn get_event_action_by_gh_action() -> GithubEventAction {
  let github_event_path = env::var_os("GITHUB_EVENT_PATH").unwrap();
  let github_event_string =
    fs::read_to_string(github_event_path).expect("Github action read event file to string is failed");

  serde_json::from_str::<GithubEventAction>(&github_event_string)
    .expect("Convert to github action event is failed")
}

pub fn get_event_action() -> GithubEventAction {
  get_event_action_by_gh_action()
}

