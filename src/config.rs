use scraper::{error::SelectorErrorKind, Selector};
use serde::{de::Visitor, Deserialize, Serialize};
use scraper::selector::ToCss;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub menu_selector: Selector,
    pub menu_anchor_selector: Selector,
    pub submenu_selector: Option<Selector>,
    // pub content_selector: Selector,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            menu_selector: Selector::parse(".mega-menu > .mega-menu-item").unwrap(),
            menu_anchor_selector: Selector::parse(":scope > .mega-menu-link").unwrap(),
            submenu_selector: Some(Selector::parse(":scope > .mega-sub-menu > .mega-menu-item").unwrap()),
        }
    }
}
