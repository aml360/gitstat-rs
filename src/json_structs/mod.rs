use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Gitstat<'a> {
    pub version: String,
    pub projects: Vec<Project<'a>>,
}

#[derive(Serialize)]
pub struct Project<'a> {
    pub name: String,
    pub commits: Vec<Commit<'a>>,
}

#[derive(Serialize)]
pub struct Commit<'a> {
    pub hash: String,
    pub author: &'a Signature,
    pub committer: &'a Signature,
    pub message: String,
    pub files: Vec<File>,
    pub isMerge: bool,
}

#[derive(Serialize)]
pub struct Signature {
    pub name: String,
    pub email: String,
    pub time: String,
}

#[derive(Serialize)]
pub struct File {
    // TODO: Add file properties
}
