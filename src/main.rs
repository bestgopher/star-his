#![deny(private_in_public, unreachable_pub)]

mod display;
mod github;

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

    let token = args.get(2);

    let repos = repos
        .split(',')
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    // 判断仓库的个数
    if repos.len() > 5 {
        println!("the number of repo greater than 5.");
        std::process::exit(1);
    }

    let handlers = repos
        .into_iter()
        .map(|repo| {
            let token = token.cloned();
            tokio::spawn(async move { github::handle(repo, token).await })
        })
        .collect::<Vec<JoinHandle<Data>>>();

    let mut data = Vec::with_capacity(handlers.len());

    for handler in handlers {
        data.push(handler.await.unwrap());
    }

    display::display(data).unwrap();
}
