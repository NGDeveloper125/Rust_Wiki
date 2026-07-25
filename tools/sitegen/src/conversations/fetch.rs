//! Fetch discussions from the GitHub GraphQL API.
//!
//! Network-touching code is deliberately thin; the JSON→model mapping lives
//! in `parse_page` so it can be unit-tested against a captured response with
//! no network access.

use serde::Deserialize;

use super::{Author, Comment, Conversation};

const GRAPHQL_URL: &str = "https://api.github.com/graphql";
/// Comments fetched per thread. Threads with more are flagged via
/// `total_comment_count` and the reader is pointed to GitHub for the rest.
pub const COMMENTS_PER_THREAD: u32 = 100;
/// Safety cap on pagination so a bad cursor can't loop forever.
const MAX_PAGES: u32 = 40;

const QUERY: &str = r#"
query($owner:String!,$name:String!,$comments:Int!,$after:String){
  repository(owner:$owner,name:$name){
    discussions(first:50, after:$after, orderBy:{field:UPDATED_AT, direction:DESC}){
      pageInfo{ hasNextPage endCursor }
      nodes{
        number title url createdAt
        author{ login url }
        category{ name emoji }
        body
        comments(first:$comments){
          totalCount
          nodes{ author{ login url } createdAt body url }
        }
      }
    }
  }
}"#;

/// Fetch every discussion in `owner/name`, following pagination.
pub fn fetch_all(owner: &str, name: &str, token: &str) -> Result<Vec<Conversation>, String> {
    let mut all = Vec::new();
    let mut after: Option<String> = None;

    for _ in 0..MAX_PAGES {
        let body = serde_json::json!({
            "query": QUERY,
            "variables": {
                "owner": owner,
                "name": name,
                "comments": COMMENTS_PER_THREAD,
                "after": after,
            }
        })
        .to_string();

        let text = post_graphql(token, body)?;
        let (mut page, next) = parse_page(&text)?;
        all.append(&mut page);

        match next {
            Some(cursor) => after = Some(cursor),
            None => return Ok(all),
        }
    }
    eprintln!("conversations: hit MAX_PAGES ({MAX_PAGES}); some threads may be omitted");
    Ok(all)
}

fn post_graphql(token: &str, body: String) -> Result<String, String> {
    let mut resp = ureq::post(GRAPHQL_URL)
        .header("Authorization", &format!("Bearer {token}"))
        .header("User-Agent", "rusty-yellow-pages-sitegen")
        .header("Content-Type", "application/json")
        .send(body.as_bytes())
        .map_err(|e| format!("request error: {e}"))?;

    resp.body_mut()
        .read_to_string()
        .map_err(|e| format!("reading response body: {e}"))
}

/// Parse one GraphQL response page into conversations + the next cursor.
fn parse_page(text: &str) -> Result<(Vec<Conversation>, Option<String>), String> {
    let resp: GqlResponse =
        serde_json::from_str(text).map_err(|e| format!("bad GraphQL JSON: {e}"))?;

    if !resp.errors.is_empty() {
        let joined = resp
            .errors
            .iter()
            .map(|e| e.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("GraphQL errors: {joined}"));
    }

    let discussions = resp
        .data
        .and_then(|d| d.repository)
        .map(|r| r.discussions)
        .ok_or("GraphQL response missing repository.discussions")?;

    let conversations = discussions.nodes.into_iter().map(map_conversation).collect();
    let next = if discussions.page_info.has_next_page {
        discussions.page_info.end_cursor
    } else {
        None
    };
    Ok((conversations, next))
}

fn map_author(a: Option<GqlAuthor>) -> Option<Author> {
    a.map(|a| Author {
        login: a.login,
        url: a.url,
    })
}

fn map_conversation(n: DiscNode) -> Conversation {
    let (category, category_emoji) = n
        .category
        .map(|c| (c.name, c.emoji))
        .unwrap_or_default();
    Conversation {
        number: n.number,
        title: n.title,
        url: n.url,
        author: map_author(n.author),
        created_at: n.created_at,
        category,
        category_emoji,
        body_md: n.body,
        total_comment_count: n.comments.total_count,
        comments: n
            .comments
            .nodes
            .into_iter()
            .map(|c| Comment {
                author: map_author(c.author),
                created_at: c.created_at,
                body_md: c.body,
                url: c.url,
            })
            .collect(),
    }
}

// ---- GraphQL response shapes (only the fields we read) ----

#[derive(Deserialize)]
struct GqlResponse {
    data: Option<GqlData>,
    #[serde(default)]
    errors: Vec<GqlError>,
}

#[derive(Deserialize)]
struct GqlError {
    #[serde(default)]
    message: String,
}

#[derive(Deserialize)]
struct GqlData {
    repository: Option<GqlRepo>,
}

#[derive(Deserialize)]
struct GqlRepo {
    discussions: GqlDiscussions,
}

#[derive(Deserialize)]
struct GqlDiscussions {
    #[serde(rename = "pageInfo")]
    page_info: GqlPageInfo,
    nodes: Vec<DiscNode>,
}

#[derive(Deserialize)]
struct GqlPageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Deserialize)]
struct DiscNode {
    number: u64,
    title: String,
    url: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    author: Option<GqlAuthor>,
    category: Option<GqlCategory>,
    body: String,
    comments: GqlComments,
}

#[derive(Deserialize)]
struct GqlAuthor {
    login: String,
    #[serde(default)]
    url: String,
}

#[derive(Deserialize)]
struct GqlCategory {
    name: String,
    #[serde(default)]
    emoji: String,
}

#[derive(Deserialize)]
struct GqlComments {
    #[serde(rename = "totalCount")]
    total_count: u64,
    nodes: Vec<GqlComment>,
}

#[derive(Deserialize)]
struct GqlComment {
    author: Option<GqlAuthor>,
    #[serde(rename = "createdAt")]
    created_at: String,
    body: String,
    #[serde(default)]
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "data": {
        "repository": {
          "discussions": {
            "pageInfo": { "hasNextPage": false, "endCursor": "Y3Vyc29y" },
            "nodes": [
              {
                "number": 7,
                "title": "How should I structure errors?",
                "url": "https://github.com/NGDeveloper125/Rust_Wiki/discussions/7",
                "createdAt": "2026-07-20T10:00:00Z",
                "author": { "login": "alice", "url": "https://github.com/alice" },
                "category": { "name": "Q&A", "emoji": ":pray:" },
                "body": "I keep going back and forth on `thiserror` vs hand-rolled.",
                "comments": {
                  "totalCount": 2,
                  "nodes": [
                    { "author": { "login": "bob", "url": "https://github.com/bob" },
                      "createdAt": "2026-07-21T09:00:00Z",
                      "body": "Use `thiserror` for libraries.",
                      "url": "https://github.com/NGDeveloper125/Rust_Wiki/discussions/7#discussioncomment-1" },
                    { "author": null,
                      "createdAt": "2026-07-22T09:00:00Z",
                      "body": "Agreed.",
                      "url": "https://github.com/NGDeveloper125/Rust_Wiki/discussions/7#discussioncomment-2" }
                  ]
                }
              }
            ]
          }
        }
      }
    }"#;

    #[test]
    fn parses_a_page() {
        let (convos, next) = parse_page(SAMPLE).expect("parse ok");
        assert_eq!(next, None, "hasNextPage false => no cursor");
        assert_eq!(convos.len(), 1);
        let c = &convos[0];
        assert_eq!(c.number, 7);
        assert_eq!(c.category, "Q&A");
        assert_eq!(c.author.as_ref().unwrap().login, "alice");
        assert_eq!(c.total_comment_count, 2);
        assert_eq!(c.comments.len(), 2);
        assert!(c.comments[1].author.is_none(), "ghost author => None");
    }

    #[test]
    fn surfaces_graphql_errors() {
        let err_json = r#"{"data":null,"errors":[{"message":"Bad credentials"}]}"#;
        let err = parse_page(err_json).unwrap_err();
        assert!(err.contains("Bad credentials"), "got: {err}");
    }

    #[test]
    fn follows_pagination_cursor() {
        let more = r#"{"data":{"repository":{"discussions":{
            "pageInfo":{"hasNextPage":true,"endCursor":"NEXT"},"nodes":[]}}}}"#;
        let (_, next) = parse_page(more).expect("parse ok");
        assert_eq!(next.as_deref(), Some("NEXT"));
    }
}
