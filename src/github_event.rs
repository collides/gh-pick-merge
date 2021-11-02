use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
#[serde()]
pub struct GithubActionPullRequestBranch {
  #[serde(rename(deserialize = "ref"))]
  pub _ref: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubActionPullRequest {
  pub number: i64,
  pub base: GithubActionPullRequestBranch,
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
  pub id: i64,
}
