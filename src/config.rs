use std::collections::HashMap;

use scraper::Selector;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    // Site-specific
    pub url: String,
    pub anchor_selector: Selector,
    pub menu_selector: Selector,
    pub menu_anchor_selector: Selector,
    pub submenu_selector: Option<Selector>,
    pub content_selector: Selector,
    pub param_selectors: HashMap<String, Selector>,

    // Output
    #[serde(default)]
    pub params_format: ParamsFormat,
    pub archetype: Option<String>,

    // Process-specific
    #[serde(default = "default_crawler_depth")]
    pub crawler_depth: usize,
    #[serde(default = "default_request_attempt_seconds")]
    pub request_attempt_seconds: u64,
    #[serde(default = "default_request_attempts")]
    pub request_attempts: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            url: "https://gvh.cz".to_string(),
            anchor_selector: Selector::parse("a[href]").unwrap(),
            menu_selector: Selector::parse(".mega-menu > .mega-menu-item").unwrap(),
            menu_anchor_selector: Selector::parse(":scope > .mega-menu-link").unwrap(),
            submenu_selector: Some(Selector::parse(":scope > .mega-sub-menu > .mega-menu-item").unwrap()),
            content_selector: Selector::parse("#main-core").unwrap(),
            param_selectors: HashMap::new(),
            params_format: ParamsFormat::Toml,
            archetype: Some(default_archetype(&ParamsFormat::Toml).to_string()),
            crawler_depth: 10,
            request_attempt_seconds: 5,
            request_attempts: 5,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamsFormat {
    Toml,
    Yaml,
    Json,
}

impl Default for ParamsFormat {
    fn default() -> Self {
        ParamsFormat::Toml
    }
}

pub const fn default_archetype(format: &ParamsFormat) -> &'static str {
    match format {
        ParamsFormat::Toml =>
r#"+++
title = '{TITLE}'
{PARAMS}
+++

# {TITLE}
{CONTENT}
"#,
        ParamsFormat::Yaml =>
r#"---
title: "{TITLE}"
{PARAMS}
---

# {TITLE}
{CONTENT}
"#,
        ParamsFormat::Json =>
r#"{
    "title": "{TITLE}"{PARAMS}
}

# {TITLE}
{CONTENT}
"#,
    }
}
const fn default_crawler_depth() -> usize {
    10
}
const fn default_request_attempt_seconds() -> u64 {
    5
}
const fn default_request_attempts() -> usize {
    10
}