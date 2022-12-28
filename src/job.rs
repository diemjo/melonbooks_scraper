use std::thread::sleep;
use crate::db::MelonDB;
use crate::web::melonbooks_scraper::MelonbooksScraper;
use crate::web::WebScraper;
use crate::common::error::Result;
use crate::model::{Availability, Product};

fn get_webscrapers() -> Result<Vec<Box<dyn WebScraper>>> {
    Ok(vec![
        Box::new(MelonbooksScraper::new()?)
    ])
}

pub(crate) fn default_job() -> Result<()> {
    update_products(vec![Availability::Available, Availability::Preorder])?;
    load_products(false)?;
    Ok(())
}

pub(crate) fn load_products(also_unavailable: bool) -> Result<()> {
    println!("[Job] Loading new products");
    for ws in get_webscrapers()? {
        load_products_ws(ws.as_ref(), also_unavailable)?;
    }
    println!("[Job] Loading new products done!");
    Ok(())
}

fn load_products_ws(ws: &dyn WebScraper, also_unavailable: bool) -> Result<()> {
    let site = ws.get_site_name();
    println!("[Site] Loading new products from {}:", site);
    let mut db = MelonDB::new()?;
    let artists = db.get_artists(site)?;
    for (aidx, artist) in artists.iter().enumerate() {
        println!("[Artist] {}/{} Loading products for artist {}:", aidx+1, artists.len(), artist);
        let urls = ws.get_urls(artist.as_str(), also_unavailable)?;
        let new_urls : Vec<&String>= urls.iter().filter(|u| !db.contains_product(u.as_str()).unwrap_or(false) && !db.is_skip_product(u.as_str()).unwrap_or(false)).collect();
        println!("[Search] Found {} total products, {} new", urls.len(), new_urls.len());
        for (pidx, &url) in new_urls.iter().enumerate() {
            let product = ws.get_product(artist.as_str(), url.as_str())?;
            if product.is_some() {
                let product = product.unwrap();
                println!("[Product] {}/{} Adding {} : {}", pidx+1, new_urls.len(), &product.url, &product.title);
                db.store_products(&vec![product], site)?;
            } else {
                println!("[Product] {}/{} Skipping {}", pidx+1, new_urls.len(), &url);
                db.skip_product(url)?;
            }
            sleep(core::time::Duration::from_millis(500));
        }
        sleep(core::time::Duration::from_millis(2000));
    }
    Ok(())
}

pub(crate) fn update_products(types: Vec<Availability>) -> Result<()> {
    for ws in get_webscrapers()? {
        update_products_ws(ws.as_ref(), &types)?;
    }
    Ok(())
}

fn update_products_ws(ws: &dyn WebScraper, types: &Vec<Availability>) -> Result<()> {
    let site = ws.get_site_name();
    let mut db = MelonDB::new()?;
    let products = db.get_products(site)?.into_iter().filter(|p| types.contains(&p.availability)).collect::<Vec<Product>>();
    for (pidx, product) in products.iter().enumerate() {
        println!("[{}/{}] updating product {}", pidx+1, products.len(), &product.url);
        if types.contains(&product.availability) {
            let new_product = ws.get_product(&product.artist, &product.url)?;
            if new_product.is_some() {
                let new_product = new_product.unwrap();
                db.update_product(&new_product)?;
            } else {
                //???
            }
        }
    }
    Ok(())
}

pub(crate) fn add_artist(artist: &str, site: &str) -> Result<()> {
    let mut db = MelonDB::new()?;
    db.store_artists(&vec![artist.to_string()], site)?;
    Ok(())
}

pub(crate) fn remove_artist(artist: &str, site: &str) -> Result<()> {
    let mut db = MelonDB::new()?;
    db.remove_artist(artist, site)?;
    Ok(())
}