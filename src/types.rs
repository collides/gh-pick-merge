pub struct CreateNewBranchResult {
  pub new_branch_name: String,
  pub not_matched_hash: Vec<String>,
}

impl CreateNewBranchResult {
  pub const fn new(name: String, hash: Vec<String>) -> Self {
    CreateNewBranchResult {
      new_branch_name: name,
      not_matched_hash: hash,
    }
  }
}
