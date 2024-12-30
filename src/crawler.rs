use std::{path::Path, process::exit};

use reqwest::{Client, Url};
use tokio::{fs, time::sleep};

use crate::{config::{Config, DEFAULT_ARCHETYPE, MAX_CRAWLER_DEPTH, REQUEST_ATTEMPT_SLEEP}, sitemap::{scrape_menus, Page, PageContents, Sitemap}, Args};

pub async fn crawl_site(config: &Config, client: &Client, args: &Args) {
    // TODO: print a prettier error message
    let url = Url::parse(&config.url).expect("invalid url");
    
    let mut sitemap = Sitemap::new(url);
    crawl_page(&mut sitemap.home, Some(&mut sitemap.unsorted), &client, &config, args, 0).await;
}

async fn crawl_page(page: &mut Page, unsorted: Option<&mut Vec<Page>>, client: &Client, config: &Config, args: &Args, depth: usize) {
    let indent = " ".repeat(depth);
    let file = args.target.join(page.path());
    if unsorted.is_some() {
        if depth >= MAX_CRAWLER_DEPTH {
            eprintln!("{indent}- crawling {}: {}", file.to_string_lossy(), page.url.path());
        } else {
            eprintln!("{indent}> crawling {}: {}", file.to_string_lossy(), page.url.path());
        }
    } else {
        eprintln!("{indent}> downloading {}: {}", file.to_string_lossy(), page.url.path());
    }

    let res = match download_page(client, &*page, 5, &indent).await {
        Ok(it) => it,
        Err(true) => return,
        Err(false) => {
            eprintln!("exiting due to network error");
            exit(30);
        }
    };

    let dom = scraper::Html::parse_document(&res);
    if depth == 0 {
        scrape_menus(&dom, page, config, None, 0);
    }

    scrape_contents(config, page, &dom, &indent);
    if !args.dry_run {
        if let Some(dir) = file.parent() {
            // eprintln!("{indent}  making dir {}", dir.to_string_lossy());
            fs::create_dir_all(dir).await.unwrap();
        }
        let text = page.construct_md(config.archetype.as_ref().map(|a| a.as_str()).unwrap_or(DEFAULT_ARCHETYPE));
        fs::write(&file, text.unwrap()).await.unwrap();
        // eprintln!("{indent}  scraped {} into {}", page.url.path(), file.to_string_lossy());
    }
    
    if depth >= MAX_CRAWLER_DEPTH {
        // eprintln!("{indent}! reached max crawler depth: {depth}");
        return;
    }
    for child_page in &mut page.children {
        if child_page.url.host_str() != Url::parse(&config.url).unwrap().host_str() {
            // eprintln!("{indent}! ignoring foreign url: {}", child_page.url);
            continue;
        }
        // eprintln!("{indent}  downloading:");
        Box::pin(crawl_page(child_page, None, client, config, args, depth + 1)).await;
    }
    if let Some(unsorted) = unsorted {
        let mut new_unsorted: Vec<Page> = dom.select(&config.anchor_selector)
        .filter_map(|anchor|
            Page::from_anchor(anchor, Some(Path::new("unsorted")))
        )
        .filter(|child_page| page.find(&child_page.url).is_none())
        .filter(|child_page| unsorted.iter().all(|p| p.url != child_page.url)).collect();
        let mut new_unsorted_children = vec![];
        for unsorted_page in &mut new_unsorted {
            if unsorted_page.url.host_str() != Url::parse(&config.url).unwrap().host_str() {
                eprintln!("{indent}! ignoring foreign url: {}", unsorted_page.url);
                continue;
            }
            Box::pin(crawl_page(unsorted_page, Some(&mut new_unsorted_children), client, config, args, depth + 1)).await;
        }
        new_unsorted.extend(new_unsorted_children);
        unsorted.extend(new_unsorted);
    }
    // println!("{}", sitemap.home.construct_md((config.archetype.as_ref().map(|a| a.as_str())).unwrap_or(DEFAULT_ARCHETYPE)).unwrap_or_default());
}

fn scrape_contents(config: &Config, page: &mut Page, dom: &scraper::Html, indent: &str) {
    let page_text_sources = dom.select(&config.content_selector);
    let page_text = page_text_sources
    // TODO: don't retransform the element's html
    .map(|element| html2md::parse_html(&element.inner_html()))
    .collect::<Vec<String>>().join("\n");

    if page_text.is_empty() {
        eprintln!("{indent}! no content found: {}", page.title);
    }
    
    let mut page_contents = PageContents::from_text(page_text);
    for (param_name, param_selector) in &config.param_selectors {
        let mut param_content = dom.select(param_selector);
        let param_element = if let Some(first) = param_content.next() {
            if let Some(_) = param_content.next() {
                eprintln!("{indent}! ignoring second {param_name} param element");
            }
            first
        } else {
            // eprintln!("{indent}! param {param_name} not found");
            continue;
        };
        let param_value = param_element.text().next().unwrap().to_string();
        page_contents.push_param(param_name.clone(), param_value);
    }
    page.add_contents(page_contents);
}

async fn download_page(client: &Client, page: &Page, retries: usize, indent: &str) -> Result<String, bool> {
    for attempt in 1..=retries {
        if attempt != 1 {
            sleep(REQUEST_ATTEMPT_SLEEP).await;
        }
        let res = match client.get(page.url.clone()).send().await {
            Ok(it) => it,
            Err(err) => {
                eprintln!("{indent}! couldn't connect to site on attempt {attempt}/{retries}: {err}");
                continue;
            },
        };
        if let Err(err) = res.error_for_status_ref() {
            if !err.status().unwrap_or_default().is_client_error() {
                eprintln!("{indent}! received status code {}, skipping page", err.status().unwrap_or_default());
                return Err(true);
            }
            eprintln!("{indent}! received status code {} on attempt {attempt}/{retries}", err.status().unwrap_or_default());
            continue;
        };
        match res.text().await {
            Ok(it) => {
                if attempt != 1 {
                    eprintln!("{indent}✓ downloading succeeded on attempt {attempt}/{retries}")
                }
                return Ok(it);
            },
            Err(err) => {
                eprintln!("{indent}! couldn't download page on attempt {attempt}/{retries}: {err}");
                continue;
            },
        }
    };
    eprintln!("{indent}! failed to download {}, maximum attempts reached", page.url);
    Err(false)
}
