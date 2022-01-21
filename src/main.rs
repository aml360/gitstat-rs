//--------Modules--------//
mod consts;
mod json_structs;
#[cfg(test)]
mod tests;

//--------Uses--------//
use git2::{Commit, Repository};
use git2::{Error, Oid};
use json_structs as models;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

struct MyRepository(Repository);

impl std::ops::Deref for MyRepository {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Sync for MyRepository {}

type SignaturesHm = HashMap<String, Arc<models::User>>;

fn run() -> Result<(), Error> {
    let repo = Repository::open(".")?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut project = models::Project {
        name: String::from(get_folder_name(&repo).unwrap_or_default()),
        commits: Vec::new(),
    };

    let signatures: Arc<RwLock<SignaturesHm>> = Arc::from(RwLock::from(HashMap::new()));

    let oids: Vec<Oid> = revwalk.into_iter().map(|oid| oid.unwrap()).collect();

    let myrepo = MyRepository(repo);

    let commits = oids
        .par_iter()
        .map(|oid| {
            let commit = myrepo.find_commit(*oid).unwrap();
            let (author, committer) = (commit.author(), commit.committer());
            let (author_str, committer_str) = (author.to_string(), committer.to_string());

            let insert_closure =
                |key: String,
                 sign: &git2::Signature,
                 mut rwlock: RwLockWriteGuard<SignaturesHm>| {
                    rwlock.insert(
                        key,
                        Arc::new(models::User {
                            name: String::from(sign.name().unwrap_or_default()),
                            email: String::from(sign.email().unwrap_or_default()),
                        }),
                    );
                };
            //Check if exist an user in hashmap to not repeat data
            let (is_author_in_hm, is_committer_in_hm) = {
                let rdlock = signatures.read().unwrap();
                (
                    rdlock.contains_key(&author_str),
                    rdlock.contains_key(&committer_str),
                )
            };
            //If is a new user, insert into hashmap
            if !is_author_in_hm {
                let rwlock = signatures.write().unwrap();
                insert_closure(author_str.clone(), &author, rwlock);
            }
            if !is_committer_in_hm {
                let rwlock = signatures.write().unwrap();
                insert_closure(committer_str.clone(), &committer, rwlock);
            }
            let rdlock = signatures.read().unwrap();
            let (auth_hm, committer_hm) = {
                (
                    rdlock.get(&author_str).unwrap(),
                    rdlock.get(&committer_str).unwrap(),
                )
            };

            models::Commit {
                author: models::Signature {
                    user: models::RcUser(Arc::clone(auth_hm)),
                    time: seconds_to_unix_time(author.when().seconds()),
                },
                committer: models::Signature {
                    user: models::RcUser(Arc::clone(committer_hm)),
                    time: seconds_to_unix_time(author.when().seconds()),
                },
                hash: commit.id().to_string(),
                files: get_commit_files(&myrepo, &commit),
                is_merge: if commit.parent_count() > 1 {
                    true
                } else {
                    false
                },
                message: get_commit_msg(&commit),
            }
        })
        .collect::<Vec<models::Commit>>();

    project.commits = commits;

    let test_struct = models::Gitstat {
        version: String::from("1.0.0"),
        projects: vec![project],
    };
    let json = serde_json::to_string(&test_struct).unwrap();
    let _to_file_result = write_to_file(&json);
    // println!("{}", &json);
    Ok(())
}

///returns the msg of a commit without trailing \n in unix-like systems and without \n\r in windows ones
fn get_commit_msg(commit: &Commit) -> String {
    let mut msg = String::from(commit.message().unwrap_or_default());
    if msg.ends_with('\n') {
        msg.pop();
        if msg.ends_with('\r') {
            msg.pop();
        }
    }
    msg
}

fn seconds_to_unix_time(sec: i64) -> String {
    use chrono::{NaiveDateTime, Utc};

    let date: chrono::DateTime<Utc> =
        chrono::DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(sec, 0), Utc);
    date.to_rfc3339()
}

fn get_commit_files(repo: &Repository, commit: &Commit) -> Vec<models::File> {
    let mut files_stats: Vec<models::File> = vec![];
    //If parent commit has a tree
    if let Some(parent_tree) = commit.parent(0).ok().and_then(|parent| parent.tree().ok()) {
        let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit.tree().unwrap()), None);
        if let Ok(diff) = diff {
            // --------------- Deltas ---------------
            for delta in diff.deltas() {
                let (new_file, old_file) = (delta.new_file(), delta.old_file());
                let (new_file_id, old_file_id) = (new_file.id(), old_file.id());
                let (new_file_blob_opt, old_file_blob_opt) =
                    (repo.find_blob(new_file_id), repo.find_blob(old_file_id));

                let line_stats_from_blobs =
                    |new_file_blob: &git2::Blob, old_file_blob: &git2::Blob| {
                        return git2::Patch::from_blobs(
                            old_file_blob,
                            None,
                            new_file_blob,
                            None,
                            None,
                        )
                        .ok()
                        .unwrap()
                        .line_stats()
                        .unwrap();
                    };

                match (new_file_blob_opt.ok(), old_file_blob_opt.ok()) {
                    // File changed
                    (Some(new_file_blob), Some(old_file_blob)) => {
                        let line_stats = line_stats_from_blobs(&new_file_blob, &old_file_blob);
                        let file_path = new_file
                            .path()
                            .and_then(|path| Some(String::from(path.to_str().unwrap_or("default"))))
                            .unwrap_or_default();
                        files_stats.push(models::File {
                            filepath: file_path,
                            additions: line_stats.1 as u64,
                            deletions: line_stats.2 as i64,
                            is_binary: false,
                            raw_deletions: line_stats.1 as u64,
                            raw_additions: line_stats.2 as i64,
                        })
                    }
                    // Deleted file
                    (None, Some(_)) => {}
                    // new file created
                    // Find a way to get line stats from single blob
                    (Some(_), None) => {}
                    // This should only happen if the 2 blobs are binary files
                    (None, None) => {}
                };
            }
        }
    }
    files_stats
}

fn get_folder_name(repo: &git2::Repository) -> Option<String> {
    let path_component = repo.path().components().rev().skip(1).next();
    path_component.and_then(|wat| {
        wat.as_os_str()
            .to_str()
            .and_then(|os_str| Some(String::from(os_str)))
    })
}

fn write_to_file(json_obj: &String) -> Result<(), std::io::Error> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(consts::FILE_NAME)?;
    file.write_all(json_obj.as_bytes())?;
    Ok(())
}

#[allow(dead_code)]
/// Prints a commit in stdout
fn print_commit(commit: &Commit) {
    println!("commit {}", commit.id());

    if commit.parents().len() > 1 {
        print!("Merge:");
        for id in commit.parent_ids() {
            print!(" {:.8}", id);
        }
        println!();
    }

    let author = commit.author();
    println!("Author: {}", author);
    println!();

    for line in String::from_utf8_lossy(commit.message_bytes()).lines() {
        println!("    {}", line);
    }
    println!();
}

fn main() {
    // let args = Args::from_args();
    match run() {
        Ok(()) => {}
        Err(err) => match err.class() {
            git2::ErrorClass::Repository => match err.code() {
                git2::ErrorCode::NotFound => {
                    println!("Repository not found");
                    println!("{}", err)
                }
                _ => {}
            },

            _ => println!("{}", err),
        },
    };
}
