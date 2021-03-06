use reqwest::{Client, Response, StatusCode};

use chrono::{DateTime, Utc};
use serde::Deserialize;

use lazy_static::lazy_static;
use regex::Regex;
use tokio::{self, task::JoinHandle};

lazy_static! {
    static ref RE: Regex = Regex::new(r#"page=(\d*)>; rel="last""#).unwrap();
}

#[derive(Deserialize, Debug)]
pub(crate) struct StargazerData {
    pub(crate) starred_at: DateTime<Utc>,
}

#[derive(Debug)]
pub(crate) struct DataWithPage {
    pub(crate) page: i32,
    pub(crate) data: Vec<StargazerData>,
}

#[derive(Debug)]
pub(crate) struct Data {
    pub(crate) repo: String,
    pub(crate) data: Vec<DataWithPage>,
    pub(crate) current_num: i32,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct RepoInfo {
    stargazers_count: i32,
    created_at: DateTime<Utc>,
}

async fn get_current_num(repo: String, token: Option<String>) -> RepoInfo {
    let r = Client::new()
        .get(format!("https://api.github.com/repos/{}", repo))
        .header("Accept", "application/vnd.github.v3+json")
        .header(
            "Authorization",
            token
                .map(|x| format!("token {}", x))
                .unwrap_or_else(|| "".to_string()),
        )
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
            (KHTML, like Gecko) Chrome/92.0.4515.159 Safari/537.36",
        )
        .send()
        .await
        .unwrap();
    r.json::<RepoInfo>().await.unwrap()
}

/// 请求github接口获取数据
pub(crate) async fn get_info(
    repo: String,
    token: Option<String>,
    page: i32,
) -> anyhow::Result<Response> {
    Client::new()
        .get(format!(
            "https://api.github.com/repos/{}/stargazers?per_page=100&page={}",
            repo, page
        ))
        .header("Accept", "application/vnd.github.v3.star+json")
        .header(
            "Authorization",
            token
                .map(|x| format!("token {}", x))
                .unwrap_or_else(|| "".to_string()),
        )
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
            (KHTML, like Gecko) Chrome/92.0.4515.159 Safari/537.37",
        )
        .send()
        .await
        .map_err(anyhow::Error::new)
}

/// 1.先获取第一页的数据，然后从header中`link`获取最大的页数
/// 2.当最大页数小于15页时，获取每一页的数据
/// 3.当最大页数大于15页时，随机获取15页的数据
/// 4.当没有`link`时，说明只有1页，此时不执行下面的操作
pub(crate) async fn handle(repo: String, token: Option<String>) -> Data {
    let res = get_info(repo.clone(), token.clone(), 1)
        .await
        .map_err(|e| {
            println!("{}", e.to_string());
            std::process::exit(1);
        })
        .unwrap();

    let page = match res.headers().get("link") {
        None => 1,
        Some(x) => RE
            .captures_iter(x.to_str().unwrap())
            .next()
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse::<i32>()
            .unwrap(),
    };
    let repo_info = get_current_num(repo.clone(), token.clone()).await;
    let mut data = Data {
        data: vec![DataWithPage {
            data: res.json::<Vec<StargazerData>>().await.unwrap(),
            page: 1,
        }],
        repo: repo.clone(),
        current_num: repo_info.stargazers_count,
        created_at: repo_info.created_at,
    };
    let handlers = match page {
        1 => return data,
        page => (2..=15)
            .into_iter()
            .map(|x| {
                if page > 15 {
                    (x as f64 / 15f64 * page as f64 - 1f64).floor() as i32
                } else {
                    x
                }
            })
            .map(|x| {
                let (repo, token) = (repo.clone(), token.clone());
                tokio::spawn(async move {
                    let result = get_info(repo, token, x).await;
                    match result {
                        Ok(response) if response.status() == StatusCode::OK => {
                            let data = response.json::<Vec<StargazerData>>().await.unwrap();
                            DataWithPage { data, page: x }
                        }
                        _ => {
                            println!("failed");
                            std::process::exit(1);
                        }
                    }
                })
            })
            .collect::<Vec<JoinHandle<DataWithPage>>>(),
    };

    data.data.reserve(handlers.len());

    for i in handlers {
        data.data.push(i.await.unwrap());
    }

    data
}

#[cfg(test)]
mod tests {
    use crate::github::{get_current_num, get_info, handle, StargazerData, RE};

    #[test]
    fn test_get_info() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let r = get_info("vuejs/vue".to_string(), None, 1).await.unwrap();
            println!("{:?}", r.headers());
            println!("{:?}", r.json::<Vec<StargazerData>>().await.unwrap());
        });
    }

    #[test]
    fn test_re() {
        let s1 = r#"<https://api.github.com/repositories/152519880/stargazers?per_page=30&page=2>; rel="next", <https://api.github.com/repositories/152519880/stargazers?per_page=30&page=6>; rel="last""#;
        let ques = RE
            .captures_iter(s1)
            .next()
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();
        assert_eq!(ques.parse::<i32>().unwrap(), 6);
        let s1 = r#"<https://api.github.com/repositories/11730342/stargazers?page=2>; rel="next", <https://api.github.com/repositories/11730342/stargazers?page=1334>; rel="last""#;
        let ques = RE
            .captures_iter(s1)
            .next()
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();
        assert_eq!(ques.parse::<i32>().unwrap(), 1334);
    }

    #[test]
    fn test_handle() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let r = handle("vuejs/vue".to_string(), None).await;
            println!("{:?}", r);
        });
    }

    #[test]
    fn test_get_current_num() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let r = get_current_num("vuejs/vue".to_string(), None).await;
            println!("{:?}", r);
        });
    }
}
