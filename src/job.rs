use std::thread::sleep;

use crate::db::MelonDB;
use crate::notification;
use crate::web::melonbooks_scraper::MelonbooksScraper;
use crate::web::WebScraper;
use crate::common::error::Result;
use crate::model::{Availability, Product};

fn get_webscrapers() -> Result<Vec<Box<dyn WebScraper>>> {
    Ok(vec![
        Box::new(MelonbooksScraper::new()?)
    ])
}

pub(crate) async fn default_job() -> Result<()> {
    update_products(vec![Availability::Available, Availability::Preorder]).await?;
    load_products(false).await?;
    Ok(())
}

pub(crate) async fn load_products(also_unavailable: bool) -> Result<()> {
    println!("[Job] Loading new products");
    for ws in get_webscrapers()? {
        load_products_ws(ws.as_ref(), also_unavailable).await?;
    }
    println!("[Job] Loading new products done!");
    Ok(())
}

async fn load_products_ws(ws: &dyn WebScraper, also_unavailable: bool) -> Result<()> {
    let site = ws.get_site_name();
    println!("[Site] Loading new products from {}:", site);
    let mut db = MelonDB::new()?;
    let artists = db.get_artists(site)?;
    for (aidx, artist) in artists.iter().enumerate() {
        println!("[Artist] {}/{} Loading products for artist {}:", aidx+1, artists.len(), artist);
        let urls = ws.get_urls(artist.as_str(), also_unavailable)?;
        let total_count = urls.len();
        let (old_urls, new_urls) : (Vec<String>, Vec<String>) = urls.into_iter()
            .filter(|u| !db.is_skip_product(u.as_str()).unwrap_or(false))
            .partition(|u| db.contains_product(u.as_str()).unwrap_or(true));
        let old_urls = old_urls.into_iter()
            .filter(|u| db.is_product_unavailable(u).unwrap_or(false))
            .collect::<Vec<String>>();
        let mut products: Vec<Product> = vec![];
        println!("[Search] Found {} total products, {} new{}", total_count, new_urls.len(), if !also_unavailable { format!(", {} available again", old_urls.len()) } else { String::new() });
        for (pidx, url) in new_urls.iter().enumerate() {
            let product = ws.get_product(artist.as_str(), url.as_str())?;
            if product.artists.contains(artist) {
                println!("[Product] {}/{} Adding {} : {}", pidx+1, new_urls.len(), &product.url, &product.title);
                db.store_products(&vec![&product], site)?;
                products.push(product);
            } else {
                println!("[Product] {}/{} Skipping {}, artist \"{}\" not in {:?}", pidx+1, new_urls.len(), &url, artist, product.artists);
                db.skip_product(product)?;
            }
            sleep(core::time::Duration::from_millis(500));
        }
        notification::notify_new_products(&products, artist).await?;
        if !also_unavailable {
            let mut products: Vec<Product> = vec![];
            for (pidx, url) in old_urls.iter().enumerate() {
                let product = ws.get_product(artist.as_str(), url.as_str())?;
                if product.availability != Availability::NotAvailable {
                    println!("[Product] {}/{} Updating {} : {}", pidx+1, old_urls.len(), &product.url, &product.title);
                    db.update_availability(&product, &product.availability)?;
                    if !db.title_contains_skip_sequence(&product.associated_artist, site, &product.title)? {
                        products.push(product);
                    }
                }
                sleep(core::time::Duration::from_millis(500));
            }
            notification::notify_product_reruns(&products, artist).await?;
        }
        sleep(core::time::Duration::from_millis(500));
    }
    Ok(())
}

pub(crate) async fn update_products(types: Vec<Availability>) -> Result<()> {
    for ws in get_webscrapers()? {
        update_products_ws(ws.as_ref(), &types).await?;
    }
    Ok(())
}

async fn update_products_ws(ws: &dyn WebScraper, types: &Vec<Availability>) -> Result<()> {
    let site = ws.get_site_name();
    let mut db = MelonDB::new()?;
    let products = db.get_products(site)?.into_iter().filter(|p| types.contains(&p.availability)).collect::<Vec<Product>>();
    for (idx, product) in products.iter().enumerate() {
        println!("[{}/{}] updating product {}", idx+1, products.len(), &product.url);
        if types.contains(&product.availability) {
            match update_single_product(ws, &mut db, product).await {
                Ok(()) => {},
                Err(crate::common::error::Error::WebError(we)) => {
                    if we.is_timeout() {
                        // retry once
                        println!("timeout, skipping product and waiting 30s");
                        tokio::time::sleep(core::time::Duration::from_secs(30)).await;
                    } else {
                        return Err(crate::common::error::Error::WebError(we));
                    }
                },
                e @ Err(_) => { return e; }
            }
        }
        if (idx+1)%64==0 {
            println!("processed {} products, waiting 30s to prevent overloading the server", idx+1);
            tokio::time::sleep(core::time::Duration::from_secs(30)).await;
        }
    }
    Ok(())
}

async fn update_single_product(ws: &dyn WebScraper, db: &mut MelonDB, product: &Product) -> Result<()> {
    let new_product = match ws.get_product(&product.associated_artist, &product.url) {
        Ok(new_product) => new_product,
        Err(crate::common::error::Error::WebError(e)) => {
            if e.is_timeout() {
                println!("warning, error occurred: {}\nRetrying once", e);
                ws.get_product(&product.associated_artist, &product.url)?
            } else if e.status().unwrap_or(reqwest::StatusCode::OK) == 404 {
                db.update_availability(&product, &Availability::Deleted)?;
                return Ok(());
            } else {
                return Err(crate::common::error::Error::WebError(e));
            }
        },
        Err(e) => { return Err(e); }
    };
    db.update_availability(&new_product, &new_product.availability)?;
    /* this cannot not happen when updating only available/preorder products
    if vec![Availability::Available, Availability::Preorder].contains(&new_product.availability) && product.availability==Availability::NotAvailable {
        notification::notify_product_rerun(&new_product).await?;
    }*/
    Ok(())
}

pub(crate) fn add_artist(artist: &str, site: &str) -> Result<()> {
    let mut db = MelonDB::new()?;
    db.insert_artists(&vec![artist.to_string()], site)?;
    Ok(())
}

pub(crate) fn remove_artist(artist: &str, site: &str) -> Result<()> {
    let mut db = MelonDB::new()?;
    db.remove_artist(artist, site)?;
    Ok(())
}