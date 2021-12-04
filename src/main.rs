//--------Modules--------//
mod consts;
mod json_structs;

//--------Uses--------//
use git2::Error;
use git2::{Commit, Repository};
use json_structs as models;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

fn run() -> Result<(), Error> {
    let repo = Repository::open(".")?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut project = models::Project {
        name: String::from(get_folder_name(&repo).unwrap_or_default()),
        commits: Vec::new(),
    };
    let mut signatures: HashMap<String, Rc<models::User>> = HashMap::new();

    for oid in &mut revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let (author, committer) = (commit.author(), commit.committer());
        let (author_str, committer_str) = (author.to_string(), committer.to_string());

        //Check if exist an user in hashmap to not repeat data
        let is_author_in_hm = signatures.contains_key(&author_str);
        let is_committer_in_hm = signatures.contains_key(&committer_str);
        let mut insert_closure = |key: String, sign: &git2::Signature| {
            signatures.insert(
                key,
                Rc::new(models::User {
                    name: String::from(sign.name().unwrap()),
                    email: String::from(sign.email().unwrap()),
                }),
            );
        };

        //If is a new user, insert into hashmap
        if !is_author_in_hm {
            insert_closure(author_str.clone(), &author);
        }
        if !is_committer_in_hm {
            insert_closure(committer_str.clone(), &committer);
        }

        let auth_hm = signatures.get(&author_str).unwrap();
        let committer_hm = signatures.get(&committer_str).unwrap();
        project.commits.push(models::Commit {
            author: models::Signature {
                user: models::RcUser(Rc::clone(&auth_hm)),
                time: author.when().seconds().to_string(),
            },
            committer: models::Signature {
                user: models::RcUser(Rc::clone(&committer_hm)),
                time: committer.when().seconds().to_string(),
            },
            hash: commit.id().to_string(),
            files: get_commit_files(&repo, &commit),
            is_merge: if commit.parent_count() > 1 {
                true
            } else {
                false
            },
            message: String::from(commit.message().unwrap()),
        });
        // print_commit(&commit);
    }
    let test_struct = models::Gitstat {
        version: String::from("1.0.0"),
        projects: vec![project],
    };
    let json = serde_json::to_string(&test_struct).unwrap();
    let to_file_result = write_to_file(&json);
    println!("{}", &json);
    Ok(())
}

fn get_commit_files(repo: &Repository, commit: &Commit) -> Vec<models::File> {
    let mut files_stats: Vec<models::File> = vec![];
    //If parent commit has a tree
    if let Some(parent_tree) = commit
        .parent(0)
        .ok()
        .and_then(|parent| parent.tree().ok().and_then(|tree| Some(tree)))
    {
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
                        files_stats.push(models::File {
                            filepath: String::from("not implemented actually"),
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
    let mut file = File::create(consts::FILE_NAME)?;
    let result = file.write_all(json_obj.as_bytes());
    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

///
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
        Err(err) => println!("{}", err),
    };
}
