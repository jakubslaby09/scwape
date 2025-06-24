use std::{collections::HashMap, path::{Path, PathBuf}};

use reqwest::Url;
use scraper::{selectable::Selectable, ElementRef, Selector};

use crate::{config::ParamsFormat, Config};

#[derive(Debug)]
pub struct Sitemap {
    pub home: Page,
    pub unsorted: Vec<Page>,
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
            unsorted: vec![],
        }
    }
}

#[derive(Clone)]
pub struct Page {
    pub title: String,
    pub slug: PathBuf,
    pub url: Url,
    pub children: Vec<Page>,
    pub contents: Option<PageContents>,
}

impl Page {
    fn new(title: String, url: Url, slug: Option<PathBuf>) -> Self {
        Self {
            slug: slug.unwrap_or(slug_from_title(&title)),
            title,
            url,
            children: vec![],
            contents: None,
        }
    }
    pub fn from_anchor(element: ElementRef, slug_parent: Option<&Path>) -> Option<Self> {
        let title = element.text().next()?.to_string();
        Some(Page {
            slug: slug_parent.map(|s| s.to_path_buf()).unwrap_or_default().join(slug_from_title(&title)),
            title,
            url: Url::parse(element.attr("href")?).ok()?,
            children: vec![],
            contents: None,
        })
    }
    pub fn contents(&self) -> Option<&PageContents> {
        self.contents.as_ref()
    }
    pub fn find(&self, url: &Url) -> Option<&Page> {
        if self.url == *url {
            Some(&self)
        } else {
            self.children.iter().find_map(|child| child.find(url))
        }
    }
    fn push(&mut self, child_page: Page) -> Option<&mut Page> {
        if let Some(_) = self.find(&child_page.url) {
            // return Err(existing);
            None
        } else {
            self.children.push(child_page);
            Some(self.children.last_mut().expect("should be there since we've just pushed it"))
        }
    }
    pub fn push_new(&mut self, name: String, url: Url, slug_name: Option<&str>) -> Option<&mut Self> {
        let slug_name = match slug_name {
            Some(it) => PathBuf::from(it),
            None => slug_from_title(&name),
        };
        self.push(Self::new(name, url, Some(self.slug.join(slug_name))))
    }
    pub fn add_contents(&mut self, contents: PageContents) {
        debug_assert!(self.contents.is_none(), "page contents already set");

        self.contents = Some(contents);
    }
    pub fn construct_md(&self, archetype: &str, format: &ParamsFormat) -> Option<String> {
        let params: String = match format {
            ParamsFormat::Toml => "",
            ParamsFormat::Yaml => "",
            ParamsFormat::Json => ",\n",
        }.to_string() + &self.contents()?.params.iter()
        // TODO: escape toml
        .map(
            |(name, value)| match format {
                ParamsFormat::Toml => format!("{name} = '{value}'"),
                ParamsFormat::Yaml => format!("{name}: \"{value}\""),
                ParamsFormat::Json => format!("    \"{name}\": \"{value}\""),
            })
        .collect::<Vec<String>>().join(match format {
            ParamsFormat::Toml |
            ParamsFormat::Yaml => "\n",
            ParamsFormat::Json => ",\n",
        });
        Some(archetype
        .replace("{PARAMS}", &params)
        .replace("{TITLE}", &self.title)
        .replace("{CONTENT}", &self.contents()?.text))
    }
    pub fn path(&self) -> PathBuf {
        if self.children.is_empty() && !self.slug.as_os_str().is_empty() {
            self.slug.with_extension("md")
        } else {
            self.slug.join("_index.md")
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
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

pub fn slug_from_title(title: &str) -> PathBuf {
    title.to_lowercase()
    .replace(" - ", "-")
    .replace(" – ", "-")
    .replace("?", "")
    .replace(" ", "-").into()
}

pub fn scrape_menus<'s>(node: impl Selectable<'s> + Copy, page: &mut Page, config: &Config, submenu_selector: Option<&Selector>, depth: usize) {
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
        let child_page = match Page::from_anchor(anchor, Some(&page.slug)) {
            Some(it) => it,
            None => {
                eprintln!("{indent}! menu item link is invalid");
                continue;
            },
        };
        eprintln!("{indent}- {}: {}", child_page.title, child_page.url);
        if depth >= config.crawler_depth {
            return;
        }
        if let Some(child_page) = page.push(child_page) {
            scrape_menus(menu_item, child_page, config, config.submenu_selector.as_ref(), depth + 1);
        };
    }
}