use scraper::{selectable::Selectable, Selector};

use crate::Config;

pub fn scrape_menus<'s>(node: impl Selectable<'s>, config: &Config, submenu_selector: Option<&Selector>, depth: usize) {
    let indent = "  ".repeat(depth);
    let menu_selector = submenu_selector.unwrap_or(&config.menu_selector);
    for menu_item in node.select(menu_selector) {
        let mut anchors = menu_item.select(&config.menu_anchor_selector);
        let anchor = match anchors.next() {
            Some(first) => {
                if let Some(_) = anchors.next() {
                    eprintln!("{indent}! ignoring second link in menu item");
                }
                first
            },
            None => {
                eprintln!("{indent}! menu item has no link");
                continue;
            },
        };
        eprintln!("{indent}- {}: {:?}", anchor.text().next().unwrap(), anchor.attr("href").unwrap());
        scrape_menus(menu_item, config, config.submenu_selector.as_ref(), depth + 1);
    }
}