use reqwest::header::HeaderMap;
use reqwest::Client;

use crate::github_event::*;
use std::{
  env, fs,
  process::{Command, Output},
};

pub fn parse_env(key: &str) -> String {
  env::var_os(key)
    .expect("Environment variable is undefined")
    .into_string()
    .expect("Environment into string is failed")
}

pub fn get_event_action() -> GithubEventAction {
  let github_event_path = env::var_os("GITHUB_EVENT_PATH").unwrap();
  let github_event_string =
    fs::read_to_string(github_event_path).expect("read to string is failed");

  serde_json::from_str::<GithubEventAction>(&github_event_string)
    .expect("convert to github event is failed")
}

pub fn github_event_repo_url() -> String {
  let repo = parse_env("GITHUB_REPOSITORY");
  let api_url = parse_env("GITHUB_API_URL");

  format!("{}/repos/{}", api_url, repo)
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

pub fn fetch_github_api_client() -> Client {
  let headers = get_github_api_headers();

  reqwest::ClientBuilder::new()
    .default_headers(headers)
    .build()
    .expect("Initial github api client is failed")
}

pub fn git_setup() {
  let github_token = parse_env("GITHUB_TOKEN");
  let repo = parse_env("GITHUB_REPOSITORY");
  let actor = parse_env("GITHUB_ACTOR");

  let url = format!("https://{}:{}@github.com/{}.git", actor, github_token, repo);

  git(["remote", "set-url", "--push", "origin", url.as_str()].to_vec());

  git(["config", "user.email", "action@github.com"].to_vec());
  git(["config", "user.name", "github action"].to_vec());
}

pub fn get_github_api_headers() -> HeaderMap {
  let token = parse_env("GITHUB_TOKEN");

  let mut headers: HeaderMap = HeaderMap::new();

  let authorization = format!("token {}", token);

  headers.append("User-Agent", "gh-pick-merge-action".parse().unwrap());
  headers.append("Authorization", authorization.parse().unwrap());
  headers.append("content-type", "application/json".parse().unwrap());
  headers.append("accept", "application/vnd.github.v3+json".parse().unwrap());
  headers
}

pub async fn github_pull_request_push_comment(pr_number: i64, comment: String) {
  let client = fetch_github_api_client();
  let repo_url = github_event_repo_url();

  let body = format!(r#"{{"body":"{}"}}"#, comment);

  let url = format!("{}/issues/${}/comments", repo_url, pr_number);

  let response = client
    .post(url)
    .body(body)
    .send()
    .await
    .expect("Failed to create pull request comment");

  println!(
    "create comment: {}",
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
) -> i64 {
  let client = fetch_github_api_client();

  let repo_url = github_event_repo_url();

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

  println!("{:?}", response);

  response.number
}

pub async fn github_get_commits_in_pr(pr_number: i64) -> Vec<String> {
  let repo_url = github_event_repo_url();
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
