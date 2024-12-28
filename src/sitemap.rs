use std::{collections::HashMap, path::PathBuf};

use reqwest::Url;
use scraper::{selectable::Selectable, Selector};

use crate::{config::DEFAULT_ARCHETYPE, Config};

#[derive(Debug)]
pub struct Sitemap {
    pub home: Page,
    // pub unsorted: Vec<Page>,
}

impl Sitemap {
    pub fn new(url: Url) -> Self {
        Self {
            home: Page {
                title: "root".to_string(),
                url: url.clone(),
                slug: PathBuf::new(),
                children: vec![],
                contents: None,
            },
            // unsorted: vec![],
        }
    }
}

#[derive(Clone)]
pub struct Page {
    pub title: String,
    pub slug: PathBuf,
    pub url: Url,
    pub children: Vec<Page>,
    contents: Option<PageContents>,
}

impl Page {
    fn new(name: String, slug: PathBuf, url: Url) -> Self {
        Self {
            title: name,
            slug,
            url,
            children: vec![],
            contents: None,
        }
    }
    pub fn contents(&self) -> Option<&PageContents> {
        self.contents.as_ref()
    }
    fn find(&self, url: &Url) -> Option<&Page> {
        if self.url == *url {
            Some(&self)
        } else {
            self.children.iter().find_map(|child| child.find(url))
        }
    }
    fn push(&mut self, child_page: Page) -> bool {
        if let Some(existing) = self.find(&child_page.url) {
            // return Err(existing);
            false
        } else {
            self.children.push(child_page);
            true
        }
    }
    pub fn push_new(&mut self, name: String, url: Url, slug_name: Option<&str>) -> Option<&mut Self> {
        let slug_name = match slug_name {
            Some(it) => it,
            None => &name
            .to_lowercase()
            .replace(" - ", "-")
            .replace(" – ", "-")
            .replace(" ", "-"),
        };
        if self.push(Self::new(name, self.slug.join(slug_name), url)) {
            Some(self.children.last_mut().expect("should be there since we've just pushed it"))
        } else {
            None
        }
    }
    pub fn add_contents(&mut self, contents: PageContents) {
        debug_assert!(self.contents.is_none(), "page contents already set");

        self.contents = Some(contents);
    }
    pub fn construct_md(&self, archetype: &str) -> Option<String> {
        let params: String = self.contents()?.params.iter()
        // TODO: escape toml
        .map(|(name, value)| format!("{name}: \"{value}\"\n"))
        .collect();
        Some(archetype
        .replace("{PARAMS}", &params)
        .replace("{TITLE}", &self.title)
        .replace("{CONTENT}", &self.contents()?.text))
    }
    pub fn path(&self) -> PathBuf {
        if self.children.is_empty() && !self.slug.as_os_str().is_empty() {
            PathBuf::from(&self.slug).with_extension("md")
        } else {
            PathBuf::from(&self.slug).join("_index.md")
        }
    }
}

impl std::fmt::Debug for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Page")
        .field("name", &self.title)
        .field("url", &self.url.to_string())
        .field("slug", &self.slug)
        .field("children", &self.children)
        // .field("md", &self.construct_md(DEFAULT_ARCHETYPE).unwrap_or_default())
        .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct PageContents {
    params: HashMap<String, String>,
    text: String,
}

impl PageContents {
    pub fn from_text(text: String) -> Self {
        Self {
            params: HashMap::new(),
            text,
        }
    }
    pub fn push_param(&mut self, name: String, value: String) {
        debug_assert!(!self.params.contains_key(&name), "param already set");
        self.params.insert(name, value);
    }
}

pub fn scrape_menus<'s>(node: impl Selectable<'s>, page: &mut Page, config: &Config, submenu_selector: Option<&Selector>, depth: usize) {
    let indent = "  ".repeat(depth);

    let menu_selector = submenu_selector.unwrap_or(&config.menu_selector);
    for menu_item in node.select(menu_selector) {
        let mut anchors = menu_item.select(&config.menu_anchor_selector);
        let anchor = if let Some(first) = anchors.next() {
            if let Some(_) = anchors.next() {
                eprintln!("{indent}! ignoring second link in menu item");
            }
            first
        } else {
            eprintln!("{indent}! menu item has no link");
            continue;
        };
        let name = anchor.text().next().unwrap();
        let href = anchor.attr("href").unwrap();
        eprintln!("{indent}- {}: {:?}", name, href);
        if let Some(child_page) = page.push_new(name.to_string(), Url::parse(&href).unwrap(), None) {
            scrape_menus(menu_item, child_page, config, config.submenu_selector.as_ref(), depth + 1);
        };
    }
}