use std::env;

use serde::{Serialize,Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct GithubActionEnv {
  pub github_api_url: String,
  pub github_repository: String,
  pub github_token: String,
  pub github_actor: String
}

impl GithubActionEnv {
  pub const fn new(github_api_url: String, github_repository: String, github_token: String, github_actor: String) -> Self {
    GithubActionEnv {
      github_api_url,
      github_repository,
      github_token,
      github_actor,
    }
  }
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

pub fn get_github_env_by_gh_action() -> GithubActionEnv {
  let api_url = parse_env("GITHUB_API_URL");
  let repo = parse_env("GITHUB_REPOSITORY");
  let github_token = parse_env("GITHUB_TOKEN");
  let actor = parse_env("GITHUB_ACTOR");


  GithubActionEnv::new(api_url, repo, github_token, actor)
}

pub fn get_github_env() -> GithubActionEnv {
  get_github_env_by_gh_action()
}