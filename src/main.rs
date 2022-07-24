use anyhow::anyhow;
use futures::{stream, StreamExt};
use regex::Regex;
use std::{borrow::Cow, env};

lazy_static::lazy_static! {
    static ref LINK_RE: Regex = Regex::new(r#"href ?= ?"([^"]+)""#).unwrap();
}

const CONCURRENCY_LIMIT: usize = 10;

#[tokio::main]
async fn main() {
    async fn main_res() -> anyhow::Result<()> {
        let url = env::args()
            .nth(1)
            .ok_or_else(|| anyhow!("missing argument 1 (the URL to check for dead links)"))?;
        let html = reqwest::get(url).await?.text().await?;
        stream::iter(find_links(&html))
            .for_each_concurrent(CONCURRENCY_LIMIT, |link| async move {
                let link: &str = &link;
                let resp = match reqwest::get(link).await {
                    Ok(resp) => resp,
                    Err(e) => {
                        println!("{link}: {e}");
                        return;
                    }
                };
                let status = resp.status();
                if status != reqwest::StatusCode::OK {
                    println!("{link}: {status}");
                }
            })
            .await;
        Ok(())
    }
    if let Err(e) = main_res().await {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn find_links(s: &str) -> impl Iterator<Item = Cow<str>> {
    LINK_RE
        .captures_iter(s)
        .map(|link| link.get(1).unwrap().as_str())
        .filter(|link| !link.starts_with('#'))
        .map(html_escape::decode_html_entities)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        let s = r#"<p>I learned about extensions when reading the <a href="https://docs.rs/hyper/latest/hyper/struct.Request.html#method.extensions">hyper docs</a>. But"#;
        let link = find_links(s).next().unwrap();
        assert_eq!(
            link,
            "https://docs.rs/hyper/latest/hyper/struct.Request.html#method.extensions"
        )
    }

    #[test]
    fn test_regex_decoded() {
        let s = r#"<a  href="http:&#x2F;&#x2F;127.0.0.1:1111&#x2F;about&#x2F;">About</a>"#;
        let link = find_links(s).next().unwrap();
        assert_eq!(link, "http://127.0.0.1:1111/about/")
    }
}
