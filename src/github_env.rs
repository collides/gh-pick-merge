use crate::get_event_action_by_gh_action;
use std::env;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubActionEnv {
  pub github_api_url: String,
  pub github_repository: String,
  pub github_token: String,
  pub github_actor: String,
  pub pull_request_number: i64,
}

impl GithubActionEnv {
  pub const fn new(
    github_api_url: String,
    github_repository: String,
    github_token: String,
    github_actor: String,
    pull_request_number: i64,
  ) -> Self {
    GithubActionEnv {
      github_api_url,
      github_repository,
      github_token,
      github_actor,
      pull_request_number,
    }
  }
}

pub fn is_travis() -> bool {
  env::var_os("TRAVIS").is_some()
}

pub fn parse_env(key: &str) -> String {
  env::var_os(key)
    .expect("Environment variable is undefined")
    .into_string()
    .expect("Environment into string is failed")
}

// pub fn is_github_action () -> bool {
//   parse_env("GITHUB_ACTION") != ""
// }

pub fn get_github_env_by_travis() -> GithubActionEnv {
  let api_url = "https://api.github.com".to_string();
  let repo = parse_env("TRAVIS_PULL_REQUEST_SLUG");
  let github_token = parse_env("GITHUB_TOKEN");
  let actor = "dp-github-bot".to_string();
  let pr_number = parse_env("TRAVIS_PULL_REQUEST").parse::<i64>().expect("Travis pr number is valid");

  GithubActionEnv::new(api_url, repo, github_token, actor, pr_number)
}

pub fn get_github_env_by_gh_action() -> GithubActionEnv {
  let api_url = parse_env("GITHUB_API_URL");
  let repo = parse_env("GITHUB_REPOSITORY");
  let github_token = parse_env("GITHUB_ACTION_TOKEN");
  let actor = parse_env("GITHUB_ACTOR");

  let event_info = get_event_action_by_gh_action();

  GithubActionEnv::new(api_url, repo, github_token, actor, event_info.number)
}

pub fn get_github_env() -> GithubActionEnv {
  if is_travis() == true  {
    return get_github_env_by_travis();
  }

  get_github_env_by_gh_action()
}
