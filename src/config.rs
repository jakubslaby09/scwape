use std::collections::HashMap;

use scraper::{error::SelectorErrorKind, Selector};
use serde::{de::Visitor, Deserialize, Serialize};
use scraper::selector::ToCss;

pub const DEFAULT_ARCHETYPE: &str =
r#"+++
title = {TITLE}
{PARAMS}
+++

# {TITLE}
{CONTENT}
"#;
pub const MAX_CRAWLER_DEPTH: usize = 10;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub url: String,
    pub menu_selector: Selector,
    pub menu_anchor_selector: Selector,
    pub submenu_selector: Option<Selector>,
    // pub content_selector: Selector,
    pub archetype: Option<String>,
    pub param_selectors: HashMap<String, Selector>,
    pub content_selector: Selector,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            url: "https://gvh.cz".to_string(),
            menu_selector: Selector::parse(".mega-menu > .mega-menu-item").unwrap(),
            menu_anchor_selector: Selector::parse(":scope > .mega-menu-link").unwrap(),
            submenu_selector: Some(Selector::parse(":scope > .mega-sub-menu > .mega-menu-item").unwrap()),
            archetype: Some(DEFAULT_ARCHETYPE.to_string()),
            param_selectors: HashMap::new(),
            content_selector: Selector::parse("#main-core").unwrap(),
        }
    }
}
