use std::str::FromStr;
use chrono::prelude::*;
use crate::common::error::{Result};
use mysql::{params, Pool, PooledConn};
use mysql::prelude::*;
use crate::CONFIGURATION;
use crate::db::sql::*;
use crate::model::{Availability, Product};

mod sql;

pub struct  MelonDB {
    conn: PooledConn
}

impl MelonDB {
    pub(crate) fn new() -> Result<Self> {
        let username = &CONFIGURATION.mysql_username;
        let password = &CONFIGURATION.mysql_password;
        let host = &CONFIGURATION.mysql_host;
        let port = &CONFIGURATION.mysql_port;
        let db = &CONFIGURATION.mysql_db;
        let mut conn = open(format!("mysql://{username}:{password}@{host}:{port}/{db}").as_str())?;
        create_tables(&mut conn)?;
        Ok(MelonDB { conn })
    }

    #[cfg(test)]
    pub(crate) fn new_local() -> Result<Self> {
        let username = &CONFIGURATION.mysql_username;
        let password = &CONFIGURATION.mysql_password;
        let host = &CONFIGURATION.mysql_host;
        let db = &CONFIGURATION.mysql_db;
        let mut conn = open(format!("mysql://{username}:{password}@{host}:3306/{db}").as_str())?;
        create_tables(&mut conn)?;
        Ok(MelonDB { conn })
    }

    // artist --------------------------------------------------------------------------------------
    pub(crate) fn get_artists(&mut self, site: &str) -> Result<Vec<String>> {
        let res: Vec<String> = self.conn.exec(SELECT_ARTISTS, params! {site})?;
        Ok(res)
    }

    pub(crate) fn store_artists(&mut self, artists: &Vec<String>, site: &str) -> Result<()> {
        self.conn.exec_batch(INSERT_ARTIST, artists.iter().map(|artist|
            params! {
                "name" => artist,
                "site" => site
            }
        ))?;
        Ok(())
    }

    pub(crate) fn remove_artist(&mut self, artist: &str, site: &str) -> Result<()> {
        self.conn.exec_drop(REMOVE_ARTIST, params! {
                "name" => artist,
                "site" => site
        })?;
        Ok(())
    }

    // products ------------------------------------------------------------------------------------
    pub(crate) fn contains_product(&mut self, url: &str) -> Result<bool> {
        let res: Vec<u32> = self.conn.exec(SELECT_PRODUCT, params! {"url" => url})?;
        Ok(!res.is_empty())
    }

    pub(crate) fn get_products(&mut self, site: &str) -> Result<Vec<Product>> {
        let res: Vec<(String, String, String, String, String, String)> = self.conn.exec(SELECT_PRODUCTS, params! { "site" => site })?;
        let products = res.into_iter().map(|(url, title, artist, site, date_added, availability)|
            Product::new(url, title, artist, site, NaiveDate::from_str(&date_added).unwrap(), Availability::from_str(&availability).unwrap())
        ).collect::<Vec<Product>>();
        Ok(products)
    }

    pub(crate) fn store_products(&mut self, products: &Vec<Product>, site: &str) -> Result<()> {
        self.conn.exec_batch(INSERT_PRODUCT, products.iter().map(|p|
            params! {
                "url" => &p.url,
                "title" => &p.title,
                "artist" => &p.artist,
                "site" => site,
                "img_url" => &p.img_url,
                "date_added" => &p.date_added.to_string(),
                "availability" => &p.availability.to_string(),
            }
        ))?;
        Ok(())
    }

    /*pub(crate) fn remove_product(&mut self, url: &str) -> Result<()> {
        self.conn.exec_drop(REMOVE_PRODUCT, params! { "url" => url })?;
        Ok(())
    }*/

    pub(crate) fn update_product(&mut self, product: &Product) -> Result<()> {
        self.conn.exec_drop(UPDATE_PRODUCT, params! {
            "url" => &product.url,
            "title" => &product.title,
            "artist" => &product.artist,
            "img_url" => &product.img_url,
            "date_added" => &product.date_added.to_string(),
            "availability" => &product.availability.to_string(),
        })?;
        Ok(())
    }

    // skip ----------------------------------------------------------------------------------------
    pub(crate) fn skip_product(&mut self, url: &str) -> Result<()> {
        self.conn.exec_drop(INSERT_SKIP_PRODUCT, params! { "url" => url })?;
        Ok(())
    }

    pub(crate) fn is_skip_product(&mut self, url: &str) -> Result<bool> {
        let res: Vec<u32> = self.conn.exec(SELECT_SKIP_PRODUCT, params! {"url" => url})?;
        Ok(!res.is_empty())
    }
}

fn open(url: &str) -> Result<PooledConn> {
    let pool = Pool::new(url)?;
    let conn = pool.get_conn()?;

    Ok(conn)
}

fn create_tables(conn : &mut PooledConn) -> Result<()> {
    conn.query_drop(CREATE_TABLES)?;
    #[cfg(feature = "notification")]
    conn.query_drop(CREATE_NOTIFICATION_TABLE)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::fmt::Debug;
    use crate::common::error::Result;
    use crate::model::{Product, Availability};
    use chrono::NaiveDate;
    use crate::db::MelonDB;

    #[test]
    fn test_artist() -> Result<()>{
        let mut db = MelonDB::new_local().unwrap();
        let artists = vec![ mafuyu(), kantoku() ];
        db.store_artists(&artists, melonbooks().as_str()).unwrap();
        let res = db.get_artists(melonbooks().as_str()).unwrap();
        assert_eq_unsorted(artists, res);
        //println!("res={}", res.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(",\n"));
        Ok(())
    }

    #[test]
    fn test_product() -> Result<()> {
        let mut db = MelonDB::new_local().unwrap();
        remove_products(&mut db).unwrap();
        let artists = vec![ mafuyu(), kantoku() ];
        db.store_artists(&artists, melonbooks().as_str()).unwrap();
        let products = vec![ prod1(), prod2(), prod3(), prod4() ];
        db.store_products(&products, melonbooks().as_str()).unwrap();
        let res = db.get_products(melonbooks().as_str()).unwrap();
        assert_eq_unsorted(products, res);
        //println!("res={}", res.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",\n"));
        Ok(())
    }

    #[test]
    fn test_remove_product() -> Result<()> {
        let mut db = MelonDB::new_local().unwrap();
        remove_products(&mut db).unwrap();
        let artists = vec![ mafuyu(), kantoku() ];
        db.store_artists(&artists, melonbooks().as_str()).unwrap();
        let products = vec![prod1(), prod2(), prod3(), prod4()];
        let less_products = vec![prod1(), prod2(), prod4()];
        db.store_products(&products, melonbooks().as_str()).unwrap();
        db.remove_product(prod3().url.as_str()).unwrap();
        let res = db.get_products(melonbooks().as_str()).unwrap();
        assert_eq_unsorted(less_products, res);
        //println!("res={}", res.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",\n"));
        Ok(())
    }

    #[test]
    fn test_remove_artist() -> Result<()> {
        let mut db = MelonDB::new_local().unwrap();
        remove_products(&mut db).unwrap();
        let artists = vec![ mafuyu(), kantoku() ];
        db.store_artists(&artists, melonbooks().as_str()).unwrap();
        let products = vec![prod1(), prod2(), prod3(), prod4()];
        db.store_products(&products, melonbooks().as_str()).unwrap();
        let res = db.get_products(melonbooks().as_str()).unwrap();
        assert_eq!(res.len(), 4);
        db.remove_artist(kantoku().as_str(), melonbooks().as_str()).unwrap();
        let res = db.get_products(melonbooks().as_str()).unwrap();
        assert_eq!(res.len(), 2);
        Ok(())
    }

    #[test]
    fn test_update_product() -> Result<()> {
        let mut db = MelonDB::new_local().unwrap();
        remove_products(&mut db).unwrap();
        let artists = vec![ mafuyu(), kantoku() ];
        db.store_artists(&artists, melonbooks().as_str()).unwrap();
        let products = vec![prod1(), prod2()];
        db.store_products(&products, melonbooks().as_str()).unwrap();
        db.update_product(&prod1_v2()).unwrap();
        let res = db.get_products(melonbooks().as_str()).unwrap();
        assert_eq_unsorted(vec![prod1_v2(), prod2()], res);
        Ok(())
    }

    fn mafuyu() -> String {
        "mafuyu".to_string()
    }

    fn kantoku() -> String {
        "kantoku".to_string()
    }

    fn melonbooks() -> String {
        "melonbooks".to_string()
    }

    fn prod1() -> Product {
        Product::new(
            "url123".to_string(),
            "title1".to_string(),
            mafuyu(),
            "url1".to_string(),
            NaiveDate::from_ymd(2022, 09, 13),
            Availability::Available
        )
    }

    fn prod1_v2() -> Product {
        Product::new(
            "url123".to_string(),
            "title1_v2".to_string(),
            mafuyu(),
            "url1".to_string(),
            NaiveDate::from_ymd(2022, 09, 13),
            Availability::NotAvailable
        )
    }

    fn prod2() -> Product {
        Product::new(
            "url456".to_string(),
            "title2".to_string(),
            mafuyu(),
            "url44".to_string(),
            NaiveDate::from_ymd(2021, 12, 01),
            Availability::Available
        )
    }

    fn prod3() -> Product {
        Product::new(
            "url789".to_string(),
            "title1".to_string(),
            kantoku(),
            "url55".to_string(),
            NaiveDate::from_ymd(2022, 03, 13),
            Availability::Preorder
        )
    }

    fn prod4() -> Product {
        Product::new(
            "url101112".to_string(),
            "title55".to_string(),
            kantoku(),
            "url007".to_string(),
            NaiveDate::from_ymd(2020, 03, 13),
            Availability::NotAvailable
        )
    }

    fn remove_products(db: &mut MelonDB) -> Result<()> {
        db.remove_product(prod1().url.as_str()).unwrap();
        db.remove_product(prod2().url.as_str()).unwrap();
        db.remove_product(prod1_v2().url.as_str()).unwrap();
        db.remove_product(prod3().url.as_str()).unwrap();
        db.remove_product(prod4().url.as_str()).unwrap();
        Ok(())
    }

    fn assert_eq_unsorted<T: Ord+Debug>(v1: Vec<T>, v2: Vec<T>) {
        let mut v1s = Vec::from(v1);
        v1s.sort();
        let mut v2s = Vec::from(v2);
        v2s.sort();
        assert_eq!(v1s, v2s);
    }
}