use crate::GithubUserInfo;
use chrono::Utc;
use reqwest::header::HeaderMap;
use reqwest::Client;

use crate::github_env::*;
use crate::github_event::*;

use std::process::{Command, Output};

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

fn get_user_info() -> GithubUserInfo {
  if is_travis() == true {
    return GithubUserInfo::new(
      "dp-github-bot".to_string(),
      "github_bot@datapipeline.com".to_string(),
    );
  }
  return GithubUserInfo::new("github action".to_string(), "action@github.com".to_string());
}

pub fn generate_pull_request_detail(detail: String) -> String {
  format!(
    r#"
### 说明

{}

#### Confluence DIP/Jira Issue 链接

- http://doc.datapipeline.com/pages/viewpage.action?pageId=14549983
- http://jira.datapipeline.com/browse/DP-1000

### Check List

在 Request Review 前，我已完成以下条目：

- [ ] PR 标题遵循 `<jira-issue-id> <type>(<scope>): <subject>` 的格式
- [ ] 已经先自己 Review 一遍，无明显错误
- [ ] 已添加相关的单元测试/集成测试代码
- [ ] 已完成自测
- [ ] 已添加对应版本的 Milestone
- 此 PR 为 Hotfix <!-- 如果不是，则删除此项 -->
  - [ ] 已添加 `need-merge-to-master` `hotfix` 标签
- 此 PR 为 Hotfix Merge <!-- 如果不是，则删除此项 -->
  - [ ] 已添加 `hotfix-merge` 标签
  - [ ] 原 Hotfix 的 PR 链接为：#100（提交到稳定分支：<稳定分支名称>）
  - [ ] 已删除原 Hotfix PR 中的 `need-merge-to-master` 标签"#,
    detail
  )
}

pub fn generate_pull_request_comment(hash: Vec<String>) -> String {
  format!("If there are empty commits, you need to overwrite the empty commits and manually pick the following commits: {}", hash.join(","))
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

pub async fn github_open_pull_request(
  head: String,
  base: String,
  title: String,
  body: String,
) -> i64 {
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

  println!("Github open pull request response:{:?}", response);

  response.number
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
