use rusqlite::{Connection, named_params};
use crate::common::error::{Result};
use crate::CONFIGURATION;
use crate::db::sql::*;
use crate::model::{Product, Availability};

mod sql;

pub struct  MelonDB {
    conn: Connection
}

impl MelonDB {
    pub(crate) fn new() -> Result<Self> {
        let db_path = &CONFIGURATION.db_path;
        let mut conn = Connection::open(db_path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        create_tables(&mut conn)?;
        Ok(MelonDB { conn })
    }

    #[cfg(test)]
    pub(crate) fn new_local() -> Result<Self> {
        let db_path = "./melonbooks.db";
        let mut conn = Connection::open(db_path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        create_tables(&mut conn)?;
        Ok(MelonDB { conn })
    }

    // artist --------------------------------------------------------------------------------------
    pub(crate) fn get_artists(&self, site: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(SELECT_ARTISTS)?;
        let rows:  Vec<std::result::Result<String, rusqlite::Error>> = stmt.query_map(named_params! {
            ":site": site
        }, |row|
            row.get::<usize, String>(0)
        )?.collect();
        let res: std::result::Result<Vec<String>, rusqlite::Error> = rows.into_iter().collect();
        Ok(res?)
    }

    pub(crate) fn insert_artists(&mut self, artists: &Vec<String>, site: &str) -> Result<()> {
        let transaction = self.conn.transaction()?;
        {
            let mut stmt = transaction.prepare(INSERT_ARTIST)?;
            let mut skip_stmt = transaction.prepare(REMOVE_SKIP_PRODUCTS)?;
            for artist in artists {
                stmt.insert(named_params! {
                    ":name": artist,
                    ":site": site
                })?;
                skip_stmt.execute(named_params! {
                    ":artist": artist
                })?;
            }
        }
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn remove_artist(&mut self, artist: &str, site: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(REMOVE_ARTIST)?;
        stmt.execute(named_params! {
            ":name": artist,
            ":site": site
        })?;
        Ok(())
    }

    // products ------------------------------------------------------------------------------------
    pub(crate) fn contains_product(&self, url: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare(SELECT_PRODUCT)?;
        let res = stmt.exists(named_params! {
            ":url": url
        })?;
        Ok(res)
    }

    pub(crate) fn is_product_unavailable(&self, url: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare(SELECT_AVAILABILITY_PRODUCT)?;
        let res = stmt.exists(named_params! {
            ":url": url,
            ":availability": Availability::NotAvailable.to_string()
        })?;
        let res2 = stmt.exists(named_params! {
            ":url": url,
            ":availability": Availability::Deleted.to_string()
        })?;
        Ok(res || res2)
    }

    pub(crate) fn get_products(&self, site: &str) -> Result<Vec<Product>> {
        let mut stmt = self.conn.prepare(SELECT_PRODUCTS)?;
        let rows: Vec<std::result::Result<Product, rusqlite::Error>> = stmt.query_map(named_params! {
            ":site": site
        }, |row|
            Product::from_row(row)
        )?.collect();
        let res: std::result::Result<Vec<Product>, rusqlite::Error> = rows.into_iter().collect();
        Ok(res?)
    }

    pub(crate) fn store_products<T: AsRef<Product>>(&mut self, products: &[T], site: &str) -> Result<()> {
        let transaction = self.conn.transaction()?;
        {
            let mut stmt = transaction.prepare(INSERT_PRODUCT)?;
            for product in products {
                let product = product.as_ref();
                stmt.insert(named_params! {
                    ":url": product.url,
                    ":title": product.title,
                    ":artist": product.associated_artist,
                    ":site": site,
                    ":img_url": product.img_url,
                    ":date_added": product.date_added.to_string(),
                    ":availability": product.availability.to_string()
                })?;
                let mut stmt = transaction.prepare(INSERT_PRODUCT_ARTIST)?;
                for artist in &product.artists {
                    stmt.insert(named_params! {
                        ":url": product.url,
                        ":artist": artist
                    })?;
                }
            }
        }
        transaction.commit()?;
        Ok(())
        /*self.conn.exec_batch(INSERT_PRODUCT, products.iter().map(|p|
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
        Ok(())*/
    }

    pub(crate) fn update_availability(&mut self, product: &Product, availability: &Availability) -> Result<()> {
        let mut stmt = self.conn.prepare(UPDATE_PRODUCT_AVAILABILITY)?;
        stmt.execute(named_params! {
             ":url": product.url,
            ":availability": availability.to_string()
        })?;
        Ok(())
        /*self.conn.exec_drop(UPDATE_PRODUCT, params! {
            "url" => &product.url,
            "title" => &product.title,
            "artist" => &product.artist,
            "img_url" => &product.img_url,
            "date_added" => &product.date_added.to_string(),
            "availability" => &product.availability.to_string(),
        })?;
        Ok(())*/
    }

    #[cfg(test)]
    pub(crate) fn remove_product(&mut self, url: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(REMOVE_PRODUCT)?;
        stmt.execute(named_params! {
            ":url": url
        })?;
        Ok(())
    }

    // skip ----------------------------------------------------------------------------------------
    pub(crate) fn skip_product(&mut self, product: Product) -> Result<()> {
        let transaction = self.conn.transaction()?;
        {
            let mut stmt = transaction.prepare(INSERT_SKIP_PRODUCT)?;
            for artist in &product.artists {
                stmt.insert(named_params! {
                    ":url": product.url,
                    ":artist": artist
                })?;
            }
        }
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn is_skip_product(&self, url: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare(SELECT_SKIP_PRODUCT)?;
        let res = stmt.exists(named_params! {
            ":url": url
        })?;
        Ok(res)
    }

    #[cfg(test)]
    pub(crate) fn title_skip_sequence(&mut self, artist: &str, site: &str, sequence: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(INSERT_TITLE_SKIP_SEQUENCE)?;
        stmt.insert(named_params! {
            ":artist": artist,
            ":site": site,
            ":sequence": sequence
        })?;
        Ok(())
    }

    pub(crate) fn title_contains_skip_sequence(&self, artist: &str, site: &str, title: &str) -> Result<bool> {
        let mut stmt = self.conn.prepare(SELECT_TITLE_CONTAINS_SKIP_SEQUENCES)?;
        let res = stmt.exists(named_params! {
            ":artist": artist,
            ":site": site,
            ":title": title
        })?;
        Ok(res)
    }
}

fn create_tables(conn : &mut Connection) -> Result<()> {
    conn.execute_batch(CREATE_TABLES)?;
    #[cfg(feature = "notification")]
    conn.execute_batch(CREATE_NOTIFICATION_TABLE)?;
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
        remove_artists(&mut db);
        db.insert_artists(&artists, melonbooks().as_str()).unwrap();
        let res = db.get_artists(melonbooks().as_str()).unwrap();
        assert_eq_unsorted(artists, res);
        //println!("res={}", res.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(",\n"));
        Ok(())
    }

    #[test]
    fn test_product() -> Result<()> {
        let mut db = MelonDB::new_local().unwrap();
        remove_products(&mut db);
        let artists = vec![ mafuyu(), kantoku() ];
        remove_artists(&mut db);
        db.insert_artists(&artists, melonbooks().as_str()).unwrap();
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
        remove_products(&mut db);
        let artists = vec![ mafuyu(), kantoku() ];
        remove_artists(&mut db);
        db.insert_artists(&artists, melonbooks().as_str()).unwrap();
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
        remove_products(&mut db);
        let artists = vec![ mafuyu(), kantoku() ];
        remove_artists(&mut db);
        db.insert_artists(&artists, melonbooks().as_str()).unwrap();
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
        remove_products(&mut db);
        let artists = vec![ mafuyu(), kantoku() ];
        remove_artists(&mut db);
        db.insert_artists(&artists, melonbooks().as_str()).unwrap();
        let products = vec![prod1(), prod2()];
        db.store_products(&products, melonbooks().as_str()).unwrap();
        db.update_availability(&prod1_v2(), &prod1_v2().availability).unwrap();
        let res = db.get_products(melonbooks().as_str()).unwrap();
        assert_eq_unsorted(vec![prod1_v2(), prod2()], res);
        Ok(())
    }

    #[test]
    fn test_title_skip_sequence() -> Result<()> {
        let mut db = MelonDB::new_local().unwrap();
        remove_products(&mut db);
        let artists = vec![ mafuyu(), kantoku() ];
        remove_artists(&mut db);
        db.insert_artists(&artists, melonbooks().as_str()).unwrap();
        db.title_skip_sequence(&mafuyu(), &melonbooks(), "leo")?;
        db.title_skip_sequence(&kantoku(), &melonbooks(), "pii-chan")?;
        let products = vec![prod1(), prod2(), prod3(), prod4()];
        let skipped_products = products.iter()
            .filter(|p| db.title_contains_skip_sequence(&p.associated_artist, &melonbooks(), &p.title).unwrap())
            .collect::<Vec<&Product>>();
        assert_eq_unsorted(vec![&prod2(), &prod4()], skipped_products);
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
            vec![mafuyu()],
            "url1".to_string(),
            NaiveDate::from_ymd(2022, 09, 13),
            Availability::Available
        )
    }

    fn prod1_v2() -> Product {
        Product::new(
            "url123".to_string(),
            "title1".to_string(),
            mafuyu(),
            vec![mafuyu()],
            "url1".to_string(),
            NaiveDate::from_ymd(2022, 09, 13),
            Availability::NotAvailable
        )
    }

    fn prod2() -> Product {
        Product::new(
            "url456".to_string(),
            "mafuyu leo badge".to_string(),
            mafuyu(),
            vec![mafuyu()],
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
            vec![kantoku()],
            "url55".to_string(),
            NaiveDate::from_ymd(2022, 03, 13),
            Availability::Preorder
        )
    }

    fn prod4() -> Product {
        Product::new(
            "url101112".to_string(),
            "sasaki to pii-chan e4".to_string(),
            kantoku(),
            vec![kantoku()],
            "url007".to_string(),
            NaiveDate::from_ymd(2020, 03, 13),
            Availability::NotAvailable
        )
    }

    fn remove_artists(db: &mut MelonDB) {
        db.remove_artist(kantoku().as_str(), melonbooks().as_str()).unwrap();
        db.remove_artist(mafuyu().as_str(), melonbooks().as_str()).unwrap();
    }

    fn remove_products(db: &mut MelonDB) {
        db.remove_product(prod1().url.as_str()).unwrap();
        db.remove_product(prod2().url.as_str()).unwrap();
        db.remove_product(prod1_v2().url.as_str()).unwrap();
        db.remove_product(prod3().url.as_str()).unwrap();
        db.remove_product(prod4().url.as_str()).unwrap();
    }

    fn assert_eq_unsorted<T: Ord+Debug>(v1: Vec<T>, v2: Vec<T>) {
        let mut v1s = Vec::from(v1);
        v1s.sort();
        let mut v2s = Vec::from(v2);
        v2s.sort();
        assert_eq!(v1s, v2s);
    }
}