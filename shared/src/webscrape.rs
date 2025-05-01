use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

/// Configuration for web scraping behavior
#[derive(Debug, Clone)]
pub struct ScraperConfig {
    /// Whether to respect robots.txt (default: true)
    pub respect_robots_txt: bool,
    /// Delay between requests in milliseconds (default: 1000)
    pub request_delay_ms: u64,
    /// User agent to use for requests (default: "Portico WebScraper/1.0")
    pub user_agent: String,
    /// Maximum content length to process in bytes (default: 5MB)
    pub max_content_length: usize,
    /// Whether to follow redirects (default: true)
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow (default: 5)
    pub max_redirects: usize,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            respect_robots_txt: true,
            request_delay_ms: 1000,
            user_agent: "Portico WebScraper/1.0".to_string(),
            max_content_length: 5 * 1024 * 1024, // 5MB
            follow_redirects: true,
            max_redirects: 5,
        }
    }
}

/// Validate and normalize a URL string
pub fn validate_url(url_str: &str) -> Result<Url> {
    let trimmed_url = url_str.trim();

    // Check if URL is empty
    if trimmed_url.is_empty() {
        return Err(anyhow!("URL cannot be empty"));
    }

    // Try to parse as-is first
    let url_result = Url::parse(trimmed_url);

    // If parsing failed, check if it's missing a scheme
    if let Err(url::ParseError::RelativeUrlWithoutBase) = url_result {
        // Try prepending http:// and see if that works
        let with_scheme = format!("http://{}", trimmed_url);
        match Url::parse(&with_scheme) {
            Ok(url) => {
                // Success - return the URL with the added scheme
                return Ok(url);
            }
            Err(e) => {
                // Still failed - return detailed error
                return Err(anyhow!("Invalid URL '{}': {}. Try including 'http://' or 'https://' prefix.", trimmed_url, e));
            }
        }
    }

    // Handle other parse errors
    let url = url_result.map_err(|e| {
        anyhow!("Invalid URL '{}': {}", trimmed_url, e)
    })?;

    // Validate scheme
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(anyhow!(
            "Unsupported URL scheme '{}'. Only http:// and https:// are supported.",
            url.scheme()
        ));
    }

    // Validate host
    if url.host_str().is_none() {
        return Err(anyhow!("URL '{}' is missing a host", trimmed_url));
    }

    Ok(url)
}

/// Check if scraping is allowed by robots.txt
async fn is_scraping_allowed(url: &Url, user_agent: &str) -> bool {
    // Try to get robots.txt
    let robots_url = url.join("/robots.txt").unwrap_or_else(|_| url.clone());

    let client = reqwest::Client::new();
    let response = match client
        .get(robots_url.as_str())
        .header("User-Agent", user_agent)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => return true, // If we can't get robots.txt, assume scraping is allowed
    };

    if !response.status().is_success() {
        return true; // If robots.txt doesn't exist or can't be accessed, assume scraping is allowed
    }

    let robots_txt = match response.text().await {
        Ok(text) => text,
        Err(_) => return true, // If we can't parse robots.txt, assume scraping is allowed
    };

    // Very simple robots.txt parsing
    // This is a simplified implementation and doesn't handle all robots.txt rules
    let path = url.path();
    let mut current_agent = "";
    let mut disallowed_paths = HashSet::new();

    for line in robots_txt.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(agent) = line.strip_prefix("User-agent:") {
            current_agent = agent.trim();
            if current_agent == "*" || current_agent == user_agent {
                disallowed_paths.clear(); // Reset for new agent
            }
        } else if (current_agent == "*" || current_agent == user_agent)
            && line.starts_with("Disallow:")
        {
            if let Some(disallowed) = line.strip_prefix("Disallow:") {
                let disallowed = disallowed.trim();
                if !disallowed.is_empty() {
                    disallowed_paths.insert(disallowed.to_string());
                }
            }
        }
    }

    // Check if the path is allowed
    !disallowed_paths
        .iter()
        .any(|disallowed| path.starts_with(disallowed))
}

/// Scrape a webpage and convert it to a structured JSON representation
/// focusing on the core textual content.
pub async fn scrape_webpage(url_str: &str) -> Result<Value> {
    // Validate the URL
    let url = validate_url(url_str)?;

    // Use default scraper configuration
    let config = ScraperConfig::default();

    // Respect robots.txt if configured
    if config.respect_robots_txt {
        let allowed = is_scraping_allowed(&url, &config.user_agent).await;
        if !allowed {
            return Err(anyhow!("Scraping not allowed by robots.txt for {}", url));
        }
    }

    // Add a delay to be polite
    if config.request_delay_ms > 0 {
        sleep(Duration::from_millis(config.request_delay_ms)).await;
    }

    // Build a client with custom settings
    let client = reqwest::Client::builder()
        .user_agent(&config.user_agent)
        .redirect(if config.follow_redirects {
            reqwest::redirect::Policy::limited(config.max_redirects)
        } else {
            reqwest::redirect::Policy::none()
        })
        .timeout(Duration::from_secs(30))
        .build()?;

    // Fetch the webpage content
    let response = match client.get(url.as_str()).send().await {
        Ok(resp) => resp,
        Err(e) => return Err(anyhow!("Failed to fetch URL '{}': {}", url_str, e)),
    };

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch webpage '{}': HTTP status {}",
            url_str,
            response.status()
        ));
    }

    // Check content type
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("text/html");

    if !content_type.contains("text/html") {
        return Err(anyhow!(
            "Unsupported content type for '{}': {}. Only HTML is supported",
            url_str,
            content_type
        ));
    }

    // Check content length
    let content_length = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    if content_length > config.max_content_length && content_length > 0 {
        return Err(anyhow!(
            "Content too large for '{}': {} bytes (max: {} bytes)",
            url_str,
            content_length,
            config.max_content_length
        ));
    }

    let html_content = match response.text().await {
        Ok(text) => text,
        Err(e) => return Err(anyhow!("Failed to get HTML text from '{}': {}", url_str, e)),
    };

    // Check actual content length
    if html_content.len() > config.max_content_length {
        return Err(anyhow!(
            "Content too large for '{}': {} bytes (max: {} bytes)",
            url_str,
            html_content.len(),
            config.max_content_length
        ));
    }

    // Parse the HTML
    let document = Html::parse_document(&html_content);

    // Extract metadata
    let title = extract_title(&document).unwrap_or_default();
    let metadata = extract_metadata(&document);

    // Extract main content with filtering
    let content = extract_filtered_content(&document);

    // Create the JSON structure
    let result = json!({
        "url": url.as_str(),
        "title": title,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "metadata": metadata,
        "content": content
    });

    Ok(result)
}

/// Extract the title from the HTML document
fn extract_title(document: &Html) -> Option<String> {
    let title_selector = Selector::parse("title").ok()?;
    document
        .select(&title_selector)
        .next()
        .map(|element| element.text().collect::<Vec<_>>().join(""))
}

/// Extract metadata from the HTML document
fn extract_metadata(document: &Html) -> Value {
    let mut metadata = json!({});

    // Try to extract description
    if let Some(description) = extract_meta_content(document, "description") {
        metadata["description"] = Value::String(description);
    }

    // Try to extract keywords
    if let Some(keywords) = extract_meta_content(document, "keywords") {
        let keywords_vec: Vec<String> = keywords.split(',').map(|k| k.trim().to_string()).collect();
        metadata["keywords"] = json!(keywords_vec);
    }

    // Try to extract author
    if let Some(author) = extract_meta_content(document, "author") {
        metadata["author"] = Value::String(author);
    }

    metadata
}

/// Helper function to extract content from meta tags
fn extract_meta_content(document: &Html, name: &str) -> Option<String> {
    let selector = Selector::parse(&format!("meta[name='{}']", name)).ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("content"))
        .map(|s| s.to_string())
}

/// Identify if an element is likely to be part of the main content
/// Used to help find the primary content container in a webpage
fn is_main_content(element: &scraper::ElementRef) -> bool {
    // Check for common content identifiers
    let id = element.value().id();
    let class = element.value().attr("class");

    // Check for common content identifiers in id and class
    let content_indicators = [
        "content",
        "main",
        "article",
        "post",
        "entry",
        "body",
        "text",
        "story",
        "container",
    ];

    // Check for common navigation/sidebar/footer identifiers to exclude
    let non_content_indicators = [
        "nav",
        "navigation",
        "menu",
        "sidebar",
        "footer",
        "header",
        "comment",
        "share",
        "social",
        "widget",
        "ad",
        "advertisement",
        "promo",
        "related",
        "recommended",
    ];

    // Check if the element has a content indicator
    let has_content_indicator = id
        .map(|id| {
            content_indicators
                .iter()
                .any(|&indicator| id.contains(indicator))
        })
        .unwrap_or(false)
        || class
            .map(|class| {
                content_indicators
                    .iter()
                    .any(|&indicator| class.contains(indicator))
            })
            .unwrap_or(false);

    // Check if the element has a non-content indicator
    let has_non_content_indicator = id
        .map(|id| {
            non_content_indicators
                .iter()
                .any(|&indicator| id.contains(indicator))
        })
        .unwrap_or(false)
        || class
            .map(|class| {
                non_content_indicators
                    .iter()
                    .any(|&indicator| class.contains(indicator))
            })
            .unwrap_or(false);

    // Element is likely main content if it has a content indicator and no non-content indicator
    has_content_indicator && !has_non_content_indicator
}

/// Extract the main content from the HTML document with filtering
fn extract_filtered_content(document: &Html) -> Vec<Value> {
    let mut content = Vec::new();

    // Try to find the main content container
    let main_selectors = [
        "main",
        "#content",
        ".content",
        "#main",
        ".main",
        "article",
        ".article",
        "#post",
        ".post",
        ".entry",
        ".entry-content",
    ];

    // Try each selector to find a main content container
    let mut main_container = None;
    for selector_str in main_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                // Use is_main_content to verify this is likely a content container
                if is_main_content(&element) {
                    main_container = Some(element);
                    break;
                }
            }
        }
    }

    // If we found a main container, extract content from it
    // Otherwise, fall back to the whole document
    let target_document = main_container
        .map(|container| container)
        .unwrap_or_else(|| {
            // If no main container found, use the body
            if let Ok(body_selector) = Selector::parse("body") {
                document.select(&body_selector).next().unwrap_or_else(|| {
                    // If no body found, use the document root
                    document.root_element()
                })
            } else {
                document.root_element()
            }
        });

    // Extract headings (h1-h6)
    for level in 1..=6 {
        if let Ok(heading_selector) = Selector::parse(&format!("h{}", level)) {
            for element in target_document.select(&heading_selector) {
                let text = clean_text(&element.text().collect::<Vec<_>>().join(""));
                if !text.is_empty() {
                    content.push(json!({
                        "type": "heading",
                        "level": level,
                        "text": text
                    }));
                }
            }
        }
    }

    // Extract paragraphs
    if let Ok(p_selector) = Selector::parse("p") {
        for element in target_document.select(&p_selector) {
            // Skip paragraphs in navigation, sidebar, etc.
            if is_in_non_content_area(&element) {
                continue;
            }

            let text = clean_text(&element.text().collect::<Vec<_>>().join(""));
            if !text.is_empty() && text.split_whitespace().count() > 3 {
                content.push(json!({
                    "type": "paragraph",
                    "text": text
                }));
            }
        }
    }

    // Extract lists
    if let (Ok(ul_selector), Ok(li_selector)) = (Selector::parse("ul"), Selector::parse("li")) {
        for ul in target_document.select(&ul_selector) {
            // Skip lists in navigation, sidebar, etc.
            if is_in_non_content_area(&ul) {
                continue;
            }

            let mut items = Vec::new();
            for li in ul.select(&li_selector) {
                let text = clean_text(&li.text().collect::<Vec<_>>().join(""));
                if !text.is_empty() {
                    items.push(Value::String(text));
                }
            }

            if !items.is_empty() {
                content.push(json!({
                    "type": "list",
                    "items": items
                }));
            }
        }
    }

    // Extract tables
    if let (Ok(table_selector), Ok(tr_selector), Ok(th_selector), Ok(td_selector)) = (
        Selector::parse("table"),
        Selector::parse("tr"),
        Selector::parse("th"),
        Selector::parse("td"),
    ) {
        for table in target_document.select(&table_selector) {
            // Skip tables in navigation, sidebar, etc.
            if is_in_non_content_area(&table) {
                continue;
            }

            let mut headers = Vec::new();
            let mut rows = Vec::new();

            for tr in table.select(&tr_selector) {
                // Check if this is a header row
                let ths: Vec<_> = tr.select(&th_selector).collect();
                if !ths.is_empty() {
                    for th in ths {
                        let text = clean_text(&th.text().collect::<Vec<_>>().join(""));
                        headers.push(Value::String(text));
                    }
                } else {
                    // This is a data row
                    let mut row = Vec::new();
                    for td in tr.select(&td_selector) {
                        let text = clean_text(&td.text().collect::<Vec<_>>().join(""));
                        row.push(Value::String(text));
                    }

                    if !row.is_empty() {
                        rows.push(Value::Array(row));
                    }
                }
            }

            if !rows.is_empty() {
                content.push(json!({
                    "type": "table",
                    "headers": headers,
                    "rows": rows
                }));
            }
        }
    }

    // Extract links (only those likely to be important content links, not navigation)
    if let Ok(a_selector) = Selector::parse("a") {
        for element in target_document.select(&a_selector) {
            // Skip links in navigation, sidebar, etc.
            if is_in_non_content_area(&element) {
                continue;
            }

            let text = clean_text(&element.text().collect::<Vec<_>>().join(""));
            let href = element.value().attr("href").unwrap_or_default();

            // Only include links with meaningful text and valid href
            if !text.is_empty() && !href.is_empty() && text.split_whitespace().count() > 1 {
                content.push(json!({
                    "type": "link",
                    "text": text,
                    "href": href
                }));
            }
        }
    }

    content
}

/// Check if an element is in a non-content area like navigation, sidebar, footer, etc.
fn is_in_non_content_area(element: &scraper::ElementRef) -> bool {
    // These selectors are used to identify non-content areas by tag name
    // We'll check against them directly in the code below
    let _non_content_selectors = [
        "nav",
        "header",
        "footer",
        "aside",
        ".sidebar",
        ".navigation",
        ".menu",
        ".comment",
        ".widget",
        ".ad",
        ".advertisement",
        ".social",
        ".share",
    ];

    // Check if the element or any of its ancestors match non-content selectors
    let mut current = Some(*element);
    while let Some(el) = current {
        // Check element's tag, id, and class
        let tag_name = el.value().name();
        let id = el.value().id();
        let class = el.value().attr("class");

        // Check if the element is a non-content element by tag name
        if tag_name == "nav" || tag_name == "header" || tag_name == "footer" || tag_name == "aside"
        {
            return true;
        }

        // Check if the element has a non-content id or class
        let non_content_indicators = [
            "nav",
            "navigation",
            "menu",
            "sidebar",
            "footer",
            "header",
            "comment",
            "share",
            "social",
            "widget",
            "ad",
            "advertisement",
        ];

        if id
            .map(|id| {
                non_content_indicators
                    .iter()
                    .any(|&indicator| id.contains(indicator))
            })
            .unwrap_or(false)
            || class
                .map(|class| {
                    non_content_indicators
                        .iter()
                        .any(|&indicator| class.contains(indicator))
                })
                .unwrap_or(false)
        {
            return true;
        }

        // Move to parent element
        current = el
            .parent()
            .and_then(|parent| scraper::ElementRef::wrap(parent));
    }

    false
}

/// Helper function to clean up text by removing extra whitespace and normalizing
fn clean_text(text: &str) -> String {
    // Remove extra whitespace
    let cleaned = text
        .trim()
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    // Replace multiple spaces with a single space
    let mut result = String::new();
    let mut last_was_space = false;

    for c in cleaned.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}
