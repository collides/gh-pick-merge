use crate::GithubUserInfo;
use chrono::Utc;
use reqwest::header::HeaderMap;
use reqwest::Client;

use crate::github_env::*;
use crate::github_event::*;

use std::process::{Command, Output};

pub fn is_travis() -> bool {
  parse_env("TRAVIS") == "true"
}

fn get_user_info() -> GithubUserInfo {
  if is_travis() == true {
    return GithubUserInfo::new(
      "dp-github-bot".to_string(),
      "github_bot@datapipeline.com".to_string(),
    );
  }
  return GithubUserInfo::new("github action".to_string(), "action@github.com".to_string());
}

pub fn git_setup() {
  let env = get_github_env();
  let user_info = get_user_info();

  let url = format!(
    "https://{}:{}@github.com/{}.git",
    env.github_actor, env.github_token, env.github_repository
  );

  git(["remote", "set-url", "--push", "origin", url.as_str()].to_vec());

  git(["config", "user.email", user_info.email.as_str()].to_vec());
  git(["config", "user.name", user_info.user_name.as_str()].to_vec());
}

pub fn generate_pull_request_comment(hash: Vec<String>) -> String {
  format!("Not pick commit: {:?}", hash)
}

pub fn generate_new_branch_name(to_branch: String) -> String {
  let timestamp: i64 = Utc::now().timestamp();

  format!("bot/auto-pick-{}-{:?}", to_branch, timestamp)
}

pub fn git(args: Vec<&str>) -> Option<Output> {
  let output = Command::new("git")
    .args(args.clone())
    .output()
    .expect("Git command failed");

  if output.status.success() == false {
    println!(
      "Git command failed: {:?}, {:?}, args: {:?}",
      output.status,
      String::from_utf8(output.stderr),
      args,
    );

    return None;
  }

  Some(output)
}

pub fn github_api_event_repo_url() -> String {
  let env = get_github_env();
  format!("{}/repos/{}", env.github_api_url, env.github_repository)
}

pub fn fetch_github_api_client() -> Client {
  let headers = get_github_api_headers();

  reqwest::ClientBuilder::new()
    .default_headers(headers)
    .build()
    .expect("Initial github api client is failed")
}

pub fn get_github_api_headers() -> HeaderMap {
  let env = get_github_env();

  let mut headers: HeaderMap = HeaderMap::new();

  let authorization = format!("token {}", env.github_token);

  headers.append("User-Agent", "gh-pick-merge-action".parse().unwrap());
  headers.append("Authorization", authorization.parse().unwrap());
  headers.append("content-type", "application/json".parse().unwrap());
  headers.append("accept", "application/vnd.github.v3+json".parse().unwrap());
  headers
}

pub async fn github_pull_request_push_comment(pr_number: String, comment: String) {
  let client = fetch_github_api_client();
  let repo_url = github_api_event_repo_url();

  let body = format!(r#"{{"body":"{}"}}"#, comment);

  let url = format!("{}/issues/{}/comments", repo_url, pr_number);

  let response = client
    .post(url)
    .body(body)
    .send()
    .await
    .expect("Failed to create pull request comment");

  println!(
    "Create comment: {}",
    response
      .text()
      .await
      .expect("Failed to create pull comment")
  );
}

pub async fn github_open_pull_request(
  head: String,
  base: String,
  title: String,
  body: String,
) -> String {
  let client = fetch_github_api_client();

  let repo_url = github_api_event_repo_url();

  let body = format!(
    r#"{{"head":"{}","base":"{}","title":"{}","body":"{}"}}"#,
    head, base, title, body
  );

  let url = format!("{}/pulls", repo_url);

  let response = client
    .post(url)
    .body(body)
    .send()
    .await
    .expect("Failed to create pull request")
    .json::<GithubCreatePullRequestResponse>()
    .await
    .expect("Failed to create pull request");

  response.number
}

pub async fn github_get_commits_in_pr(pr_number: String) -> Vec<String> {
  let repo_url = github_api_event_repo_url();
  let client = fetch_github_api_client();
  let mut commits = Vec::new();

  let url = format!("{}/pulls/{}/commits", repo_url, pr_number);

  let response = client
    .get(url)
    .send()
    .await
    .expect("Failed to get commits")
    .json::<Vec<GithubGetCommitResponseItem>>()
    .await
    .expect("Failed into json by commit");

  for commit in response {
    commits.push(commit.sha);
  }
  commits
}
