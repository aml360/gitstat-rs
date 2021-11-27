//--------Modules--------//
mod json_structs;

//--------Uses--------//
use git2::Error;
use git2::{Commit, Repository};
use json_structs as models;
use std::collections::HashMap;

fn run() -> Result<(), Error> {
    let repo = Repository::open(".")?;
    // let branches = repo.branches(None)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut project = models::Project {
        // TODO: Get project from repo variable
        name: String::from("dsadsa"),
        commits: Vec::new(),
    };
    let mut signatures: HashMap<String, models::Signature> = HashMap::new();

    for oid in &mut revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        let author = commit.author();
        let author_str = author.to_string();
        let committer = commit.committer();
        let committer_str = committer.to_string();

        let signature_contains_author = signatures.contains_key(&author_str);
        let signature_contains_commiter = signatures.contains_key(&committer_str);
        let mut insert_closure = |key: String, sign: git2::Signature| {
            signatures.insert(
                key,
                models::Signature {
                    name: String::from(sign.name().unwrap()),
                    email: String::from(sign.email().unwrap()),
                    // TODO: function that serializes commit time or returns "" if error occurs
                    time: String::from("DateToStringNotImplemented"),
                },
            );
        };
        if !signature_contains_author {
            insert_closure(author_str.clone(), author);
        }
        if !signature_contains_commiter {
            insert_closure(committer_str.clone(), committer);
        }

        let auth_hm = signatures.get(&author_str).unwrap() as *const models::Signature;
        let committer_hm = signatures.get(&committer_str).unwrap() as *const models::Signature;
        unsafe {
            project.commits.push(models::Commit {
                author: &(*auth_hm),
                committer: &(*committer_hm),
                hash: commit.id().to_string(),
                // TODO: Obtain files with functional rust or for loop that returns
                files: vec![],
                isMerge: if commit.parent_count() > 1 {
                    true
                } else {
                    false
                },
                message: String::from(commit.message().unwrap()),
            });
        }
        print_commit(&commit);
    }
    let test_struct = json_structs::Gitstat {
        version: String::from("1.0.0"),
        projects: vec![project],
    };
    println!("{}", serde_json::to_string_pretty(&test_struct).unwrap());
    Ok(())
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
