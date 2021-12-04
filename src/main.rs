//--------Modules--------//
mod json_structs;

//--------Uses--------//
use git2::Error;
use git2::{Commit, Repository};
use json_structs as models;
use std::collections::HashMap;
use std::rc::Rc;

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
            // TODO: Obtain files with functional rust or for loop that returns
            files: vec![],
            is_merge: if commit.parent_count() > 1 {
                true
            } else {
                false
            },
            message: String::from(commit.message().unwrap()),
        });
        print_commit(&commit);
    }
    let test_struct = models::Gitstat {
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
