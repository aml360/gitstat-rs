use std::{ops::Deref, rc::Rc};

use serde::{ser::SerializeStruct, Serialize};

#[derive(Serialize)]
pub struct Gitstat {
    pub version: String,
    pub projects: Vec<Project>,
}

#[derive(Serialize)]
pub struct Project {
    pub name: String,
    pub commits: Vec<Commit>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    pub hash: String,
    pub author: Signature,
    pub committer: Signature,
    pub message: String,
    pub files: Vec<File>,
    pub is_merge: bool,
}

pub struct RcUser(pub Rc<User>);

impl Deref for RcUser {
    type Target = Rc<User>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for RcUser {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl Serialize for RcUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("user", 2)?;
        s.serialize_field("name", &self.0.name)?;
        s.serialize_field("email", &self.0.email)?;
        s.end()
    }
}

#[derive(Serialize)]
pub struct Signature {
    #[serde(flatten)]
    pub user: RcUser,
    pub time: String,
}

#[derive(Serialize)]
pub struct User {
    pub name: String,
    pub email: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub filepath: String,
    ///Number of file line additions
    pub additions: u64,
    ///Number of file line additions
    pub deletions: i64,
    pub is_binary: bool,
    pub raw_deletions: u64,
    pub raw_additions: i64,
}
