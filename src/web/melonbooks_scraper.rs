use std::sync::Arc;
use chrono::{Utc};
use reqwest::blocking::Client;
use reqwest::cookie::Jar;
use reqwest::{Url};
use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name, Predicate};
use crate::common::error::Error::{HtmlParseError};
use crate::model::{Availability, Product};
use crate::web::WebScraper;
use crate::common::error::Result;
use crate::model::Availability::{Available, NotAvailable, Preorder};

const SITE_NAME: &str = "melonbooks";
const ARTIST_URL: &str = "https://www.melonbooks.co.jp/search/search.php?name={artist}&text_type=author&pageno={pageno}";
const ARTIST_URL_ALSO_UNAVAILABLE: &str = "https://www.melonbooks.co.jp/search/search.php?name={artist}&text_type=author&pageno={pageno}&is_end_of_sale[]=1&is_end_of_sale2=1";
const PRODUCT_BASE_URL: &str = "https://www.melonbooks.co.jp{relative_url}";

pub struct MelonbooksScraper {
    client: Client,
}

impl MelonbooksScraper {
    pub fn new() -> Result<Self> {
        let jar = Jar::default();
        jar.add_cookie_str("AUTH_ADULT=1", &"https://www.melonbooks.co.jp".parse::<Url>().unwrap());
        let client = Client::builder()
            .cookie_provider(Arc::new(jar))
            .pool_max_idle_per_host(0)
            .build()?;
        Ok(MelonbooksScraper { client })
    }

    fn grid_parse_url(node: Node) -> Result<String> {
        let rel_url = node.find(Class("product_title"))
            .next()
            .and_then(|p| p.parent())
            .and_then(|a| a.attr("href"))
            .ok_or(HtmlParseError("product_list".to_string()))?;
        let url = PRODUCT_BASE_URL.replace("{relative_url}", rel_url);
        Ok(url)
    }

    fn parse_main_category(node: Node) -> Result<String> {
        let str = node.find(Class("bcs").descendant(Name("span"))).skip(1).next().ok_or(HtmlParseError("product_main_category".to_string()))?;
        let category = str.inner_html();
        Ok(category)
    }

    fn parse_title(node: Node) -> Result<String> {
        let str = node.find(Class("page-header")).next().ok_or(HtmlParseError("product_title".to_string()))?;
        //let str = node.find(Attr("id", "title").descendant(Class("str"))).next().ok_or(HtmlParseError("product_title".to_string()))?;
        let title = str.inner_html();
        Ok(title)
    }

    fn parse_img_url(node: Node) -> Result<String> {
        let img_url = node.find(Class("item-img").descendant(Name("img")))
            .next()
            .and_then(|i| i.attr("src"))
            .ok_or(HtmlParseError("img_url".to_string()))?
            .replace("//", "https://");
        Ok(img_url)
    }

    /*fn parse_date(node: Node) -> Result<NaiveDate> {
        let tr_opt = node.find(Class("stripe").descendant(Name("tr"))).filter(
            |tr| tr.find(Name("th")).next().map_or(String::new(), |n|n.inner_html()) == "発行日"
        ).next();
        match tr_opt {
            Some(tr) => {
                let date_str = tr.find(Name("td")).next().ok_or(HtmlParseError("date".to_string()))?.inner_html();
                let date= NaiveDate::parse_from_str(date_str.as_str(), "%Y/%m/%d");
                return match date {
                    Ok(d) => Ok(d),
                    Err(_) => Self::parse_header_date(node)
                }
            }
            None => Self::parse_header_date(node)
        }
    }

    fn parse_header_date(node: Node) -> Result<NaiveDate> {
        let span_opt = node.find(Class("onsale").child(Name("span"))).next();
        match span_opt {
            Some(span) => {
                let date = span.inner_html();
                lazy_static! {
                            static ref FULL_DATE_RE: Regex = Regex::new(r"^(\d{4})年(\d{2})月(\d{2})日$").unwrap();
                            static ref PARTIAL_DATE_RE: Regex = Regex::new(r"^(\d{4})年(\d{2})月(.)旬?$").unwrap();
                        }
                if FULL_DATE_RE.is_match(date.as_str()) {
                    let captures = FULL_DATE_RE.captures(date.as_str()).unwrap();
                    let year: i32 = captures.get(1).unwrap().as_str().parse().or(Err(HtmlParseError("date".to_string())))?;
                    let month: u32 = captures.get(2).unwrap().as_str().parse().or(Err(HtmlParseError("date".to_string())))?;
                    let day: u32 = captures.get(3).unwrap().as_str().parse().or(Err(HtmlParseError("date".to_string())))?;
                    return Ok(NaiveDate::from_ymd(year, month, day));
                }
                else if PARTIAL_DATE_RE.is_match(date.as_str()) {
                    let captures = PARTIAL_DATE_RE.captures(date.as_str()).unwrap();
                    let year: i32 = captures.get(1).unwrap().as_str().parse().or(Err(HtmlParseError("date".to_string())))?;
                    let month: u32 = captures.get(2).unwrap().as_str().parse().or(Err(HtmlParseError("date".to_string())))?;
                    let day: u32 = match captures.get(3).unwrap().as_str() {
                        "中" => Ok(15),
                        "下" => Ok(30),
                        s => Err(HtmlParseError("date ".to_string() + s))
                    }?;
                    return Ok(NaiveDate::from_ymd(year, month, day));
                }
                else if date.eq("未定") {
                    return Ok(Utc::now().date_naive())
                }
                else {
                    return Err(HtmlParseError("date ".to_string() + date.as_str()));
                }
            },
            None => Ok(Utc::now().date_naive())
        }
    }*/

    fn parse_availability(node: Node) -> Result<Availability> {
        let span = node.find(Class("state-instock")).next().ok_or(HtmlParseError("availability".to_string()))?;
        let availability = match span.inner_html().as_str() {
            "-" => Ok(NotAvailable),
            "好評受付中" => Ok(Preorder),
            "残りわずか" => Ok(Available),
            "在庫あり" => Ok(Available),
            "発売中" => Ok(Available),
            other => Err(HtmlParseError("availability_type".to_string() + other))
        }?;
        Ok(availability)
    }

    fn parse_product(&self, artist: &str, product_url: &str, html: Document) -> Result<Option<Product>> {
        let main_part = html.find(Class("item-page")).next().ok_or(HtmlParseError("product_main_part".to_string()))?;

        /*let main_category = MelonbooksScraper::parse_main_category(main_part)?;
        if vec!["同人DL音楽・ソフト", "電子書籍"].contains(&main_category.as_str()) {
            return Ok(None);
        }*/

        let title = Self::parse_title(main_part)?;
        let img_url = Self::parse_img_url(main_part)?;
        //let date_added = Self::parse_date(main_part)?;
        let date_added = Utc::now().date_naive();
        let availability = Self::parse_availability(main_part)?;

        let product = Product::new(product_url.to_string(), title, artist.to_string(), img_url, date_added, availability);
        //println!("{}", product);
        Ok(Some(product))
    }
}

impl WebScraper for MelonbooksScraper {
    fn get_site_name(&self) -> &'static str {
        return SITE_NAME;
    }

    fn get_urls(&self, artist: &str, also_unavailable: bool) -> Result<Vec<String>> {
        let mut product_urls: Vec<String> = Vec::with_capacity(100);
        let mut pageno = 1;

        while pageno>0 {
            let mut items_on_page = 0;
            let search_url = match also_unavailable {
                true => ARTIST_URL_ALSO_UNAVAILABLE.replace("{artist}", artist).replace("{pageno}", pageno.to_string().as_str()),
                false => ARTIST_URL.replace("{artist}", artist).replace("{pageno}", pageno.to_string().as_str())
            };
            let response = self.client.get(search_url).send()?.error_for_status()?;
            let body = response.text()?;
            let html = Document::from(body.as_str());
            let items = html.find(Class("item-list").descendant(Name("li")));
            for node in items {
                if node.attr("class").unwrap_or("").eq("item-list__placeholder") {
                    continue;
                }
                let product_url = MelonbooksScraper::grid_parse_url(node)?;
                product_urls.push(product_url);
                items_on_page+=1;
            }
            println!("[Search] Found {} products...", 100*(pageno-1)+items_on_page);
            pageno = if items_on_page==100 {pageno+1} else {-1};
        }
        Ok(product_urls)
    }

    fn get_product(&self, artist: &str, product_url: &str) -> Result<Option<Product>> {
        let response = self.client.get(product_url).send()?.error_for_status()?;
        let body = response.text()?;
        let html = Document::from(body.as_str());
        let product = self.parse_product(artist, product_url, html);
        match product {
            Ok(p) => Ok(p),
            Err(e) => {
                println!("Error parsing product {} : {:?}", product_url, e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::web::melonbooks_scraper::MelonbooksScraper;
    use crate::web::WebScraper;

    #[test]
    fn test_get() {
        let ws = MelonbooksScraper::new().unwrap();
        let urls = ws.get_urls("カントク", true).unwrap();
        for url in urls.iter().take(3) {
            let product = ws.get_product("カントク", url).unwrap().unwrap();
            println!(" {}, {}", product.date_added, product.title);
        }
    }

    #[test]
    fn test_get_urls() {
        let ws = MelonbooksScraper::new().unwrap();
        let urls = ws.get_urls("カントク", true).unwrap();
        println!("urls: {}", urls.len());
        assert_ne!(urls.len(), 0);
    }

    #[test]
    fn test_get_product() {
        let ws = MelonbooksScraper::new().unwrap();
        let url = "https://www.melonbooks.co.jp/detail/detail.php?product_id=1727239";
        let product = ws.get_product("カントク", url).unwrap();
        println!("{:?}", product);
    }

    #[test]
    fn test_get_electronic_item() {
        let ws = MelonbooksScraper::new().unwrap();
        let url = "https://www.melonbooks.co.jp/detail/detail.php?product_id=1374037";
        let product = ws.get_product("カントク", url).unwrap();
        println!("{:?}", product);
    }

    #[test]
    fn test_get_all_kantoku_products() {
        let ws = MelonbooksScraper::new().unwrap();
        let urls = ws.get_urls("カントク", true).unwrap();
        for url in urls.iter().skip(370) {
            let product = ws.get_product("カントク", url).unwrap().unwrap();
            println!(" {}, {}", product.date_added, product.title);
        }
    }
}