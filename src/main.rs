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

        let signature_contains_author = !signatures.contains_key(&author_str);
        let signature_contains_commiter = !signatures.contains_key(&committer_str);

        //TODO: Fix mutable reference error.
        let hmref = &mut signatures;
        let mut insert_closure = || {
            hmref.insert(
                author_str.clone(),
                models::Signature {
                    // TODO: Pass value to closure that implements name and email methods (author and committer)
                    name: String::from(author.name().unwrap()),
                    email: String::from(author.email().unwrap()),
                    time: String::from("DateToStringNotImplemented"),
                    // time: String::from(commit.time()),
                },
            );
        };

        if signature_contains_author {
            insert_closure()
        }

        if signature_contains_commiter {
            insert_closure()
        }

        // TODO: Solve reference problems, fails because hashmap is mutable, the method get returns a reference,
        // in the next iteration because hm is mutable the data pointed by the reference could be invalidated if we remove that data

        let auth_hm = signatures.get(&author_str).unwrap();
        let committer_hm = signatures.get(&committer_str).unwrap();

        project.commits.push(models::Commit {
            author: auth_hm,
            committer: committer_hm,
            hash: commit.id().to_string(),
            files: vec![],
            isMerge: if commit.parent_count() > 1 {
                true
            } else {
                false
            },
            message: String::from(commit.message().unwrap()),
        });
        println!("{}", author);
        print_commit(&commit);
    }

    let amli = models::Signature {
        name: String::from("Amli"),
        email: String::from("amli@gmail.com"),
        time: String::from("dsadsa"),
    };
    project.commits.push(models::Commit {
        author: &amli,
        isMerge: false,
        hash: String::from("hashTest"),
        message: String::from("msg de commit"),
        files: vec![models::File {}],
        committer: &amli,
    });
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
