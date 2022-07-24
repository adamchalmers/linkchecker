use anyhow::anyhow;
use futures::{stream, StreamExt};
use regex::Regex;
use reqwest::StatusCode;
use std::{borrow::Cow, collections::HashSet, env};

lazy_static::lazy_static! {
    static ref LINK_RE: Regex = Regex::new(r#"href ?= ?"([^"]+)""#).unwrap();
}

/// How many concurrent HTTP requests can be in flight at once.
const CONCURRENCY_LIMIT: usize = 10;

#[tokio::main]
async fn main() {
    async fn main_res() -> anyhow::Result<()> {
        let url = env::args()
            .nth(1)
            .ok_or_else(|| anyhow!("missing argument 1 (the URL to check for dead links)"))?;

        // Find all links within the webpage.
        let html = reqwest::get(url).await?.text().await?;
        let all_links = find_links(&html);

        // Find which links are broken.
        let broken_links: Vec<_> = stream::iter(all_links)
            // Check the HTTP status of each link.
            .map(|link| async move {
                match reqwest::get(link.to_string()).await {
                    Ok(resp) if resp.status() == StatusCode::OK => None,
                    Ok(_) => Some(link),
                    Err(_) => Some(link),
                }
            })
            // Make the HTTP requests concurrently.
            .buffer_unordered(CONCURRENCY_LIMIT)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect();

        if !broken_links.is_empty() {
            // One big println is faster than doing many small printlns, because each println call
            // has to get a lock for stdout. This prevents lines from interleaving each other.
            println!("{}", broken_links.join("\n"));
            std::process::exit(1);
        }

        Ok(())
    }
    if let Err(e) = main_res().await {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

/// Finds all HTML links within the given string. Ignores fragment links.
fn find_links(s: &str) -> HashSet<Cow<str>> {
    LINK_RE
        .captures_iter(s)
        .map(|link| link.get(1).unwrap().as_str())
        .filter(|link| !link.starts_with('#'))
        .map(html_escape::decode_html_entities)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex() {
        let s = r#"<p>I learned about extensions when reading the <a href="https://docs.rs/hyper/latest/hyper/struct.Request.html#method.extensions">hyper docs</a>. But"#;
        let link = find_links(s);
        assert_eq!(
            link,
            HashSet::from([Cow::Borrowed(
                "https://docs.rs/hyper/latest/hyper/struct.Request.html#method.extensions"
            )])
        )
    }

    #[test]
    fn test_regex_decoded() {
        let s = r#"<a  href="http:&#x2F;&#x2F;127.0.0.1:1111&#x2F;about&#x2F;">About</a>"#;
        let link = find_links(s);
        assert_eq!(
            link,
            HashSet::from([Cow::Borrowed("http://127.0.0.1:1111/about/")])
        )
    }
}
