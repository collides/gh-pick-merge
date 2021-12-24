use crate::helpers::post_github_api;
use std::fs;

use crate::helpers::fetch_github_api_client;
use crate::helpers::github_api_event_repo_url;
use crate::GithubCreatePullRequestResponse;
use crate::GithubGetCommitResponseItem;

const DEFAULT_GITHUB_PULL_REQUEST_TEMPLATE_PATH: &str = "./.github/pull_request_template.md";

pub async fn github_open_pull_request(head: String, base: String, title: String, comment: String) -> i64 {
  let repo_url = github_api_event_repo_url();

  let template_content_json_string = get_pull_request_body();

  let body = format!( 
    r#"{{"head":"{}","base":"{}","title":"{}","body":{}}}"#,
    head, base, title, template_content_json_string
  );

  let url = format!("{}/pulls", repo_url);

  let pr_number = post_github_api(url, body)
    .await
    .json::<GithubCreatePullRequestResponse>()
    .await
    .expect("Failed to parse create pull response")
    .number;

  github_pull_request_push_comment(pr_number, comment).await;

  pr_number
}

pub async fn github_pull_request_push_comment(pr_number: i64, comment: String) {
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

pub async fn github_get_commits_in_pr(pr_number: i64) -> Vec<String> {
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

fn get_pull_request_body() -> String {
  let pull_request_template_content = fs::read_to_string(DEFAULT_GITHUB_PULL_REQUEST_TEMPLATE_PATH);

  match pull_request_template_content {
    Ok(content) => {
      serde_json::to_string(&content).expect("get pull request body convert json to string error")
    }
    Err(error) => {
      println!("Read pull request template content error: {}", error);
      String::from("")
    }
  }
}
