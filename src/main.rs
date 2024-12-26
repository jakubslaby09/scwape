use reqwest::Client;
use scraper::{error::SelectorErrorKind, selectable::Selectable, Selector};

#[tokio::main]
async fn main() {
    let client = Client::new();
    let config = Config {
        menu_selector: Selector::parse(".mega-menu > .mega-menu-item").unwrap(),
        menu_anchor_selector: Selector::parse(":scope > .mega-menu-link").unwrap(),
        submenu_selector: Some(Selector::parse(":scope > .mega-sub-menu > .mega-menu-item").unwrap()),
    };
    let res = client.get("https://gvh.cz")
    .send().await.expect("couldn't connect to site")
    .text().await.expect("couldn't download home page");

    scrape_page(&res, &config);
}

fn scrape_page(page: &str, config: &Config) {
    let dom = scraper::Html::parse_document(page);

    scrape_menus(&dom, config, None, 0);

}

fn scrape_menus<'s>(node: impl Selectable<'s>, config: &Config, submenu_selector: Option<&Selector>, depth: usize) {
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

struct Config {
    pub menu_selector: Selector,
    pub menu_anchor_selector: Selector,
    pub submenu_selector: Option<Selector>,
    // pub content_selector: Selector,
}