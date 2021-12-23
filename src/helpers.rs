use crate::GithubActionPullRequestLabel;
use crate::GithubUserInfo;
use chrono::Utc;
use reqwest::header::HeaderMap;
use reqwest::Body;
use reqwest::Client;
use reqwest::Response;

use crate::github_env::*;

use std::process::{Command, Output};

pub fn match_pick_merge_labels(labels: Vec<GithubActionPullRequestLabel>) -> Vec<String> {
  labels
    .iter()
    .filter(|label| label.name.starts_with("pick-to/"))
    .map(|label| label.name.clone())
    .collect()
}

pub fn generate_pull_request_comment(hash: Vec<String>) -> String {
  format!("If there are empty commits, you need to overwrite the empty commits and manually pick the following commits: {}", hash.join(","))
}

pub fn generate_new_branch_name(to_branch: String) -> String {
  let timestamp: i64 = Utc::now().timestamp();

  format!("bot/auto-pick-{}-{:?}", to_branch, timestamp)
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

// ------ github helpers ------

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

pub async fn get_github_api(url: String) -> Response {
  let client = fetch_github_api_client();

  let response = client
    .get(url.clone())
    .send()
    .await
    .expect("Failed get github api");

  if response.status().is_success() == true {
    println!("Success to get {} github api", url);
  } else {
    panic!("Failed to get github api response {:?}", response);
  }

  response
}

pub async fn post_github_api<T: Into<Body>>(url: String, body: T) -> Response {
  let client = fetch_github_api_client();

  let response = client
    .post(url.clone())
    .body(body)
    .send()
    .await
    .expect("Failed post github api");

  if response.status().is_success() == true {
    println!("Success to post {} github api", url);
  } else {
    panic!("Failed to post github api response {:?}", response);
  }

  response
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
