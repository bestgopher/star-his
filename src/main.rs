#![deny(private_in_public, unreachable_pub)]

mod github;
mod display;

use std::io;
use crate::github::Data;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let repos = if let Some(repos) = args.get(1) {
        repos
    } else {
        println!("please input the repo name");
        std::process::exit(1);
    };

    let handlers = repos.split(",").map(|repo| {
        let repo = repo.to_string();
        tokio::spawn(async move { github::handle(repo, None).await })
    }).collect::<Vec<JoinHandle<Data>>>();

    let mut data = Vec::with_capacity(handlers.len());

    for handler in handlers {
        data.push(handler.await.unwrap());
    }

    display::display(data).unwrap();
}
