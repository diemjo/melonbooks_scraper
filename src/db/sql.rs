pub const CREATE_TABLES : &str =
    r"CREATE TABLE IF NOT EXISTS artists (
        name VARCHAR(64) NOT NULL,
        site VARCHAR(32) NOT NULL,
        PRIMARY KEY (name, site)
    );

    CREATE TABLE IF NOT EXISTS products (
        url VARCHAR(128) NOT NULL,
        title VARCHAR(256) NOT NULL,
        artist VARCHAR(64) NOT NULL,
        site VARCHAR(32) NOT NULL,
        imgUrl VARCHAR(128) NOT NULL,
        dateAdded CHAR(10) NOT NULL,
        availability CHAR(16),
        PRIMARY KEY (url),
        CONSTRAINT `fk_artist_name`
            FOREIGN KEY (artist, site) REFERENCES artists (name, site)
            ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS skip_products (
        url VARCHAR(128) NOT NULL,
        PRIMARY KEY (url)
    );
";

#[cfg(feature = "notification")]
const CREATE_NOTIFICATION_TABLE: &str =
    r"CREATE TABLE IF NOT EXISTS notifications
        artist VARCHAR(64),
        site VARCHAR(32),
        receiver VARCHAR(128),
        method VARCHAR(32),
        PRIMARY KEY (artist, site, receiver),
        CONSTRAINT `fk_artist_name`
            FOREIGN KEY (artist, site) REFERENCES artists (name, site)
            ON DELETE CASCADE
    );
";

pub const SELECT_ARTISTS: &str =
    r"SELECT name FROM artists WHERE site = (:site) ORDER BY name ASC";

pub const INSERT_ARTIST: &str =
    r"INSERT IGNORE INTO artists (name, site) VALUES (:name, :site)";

pub const REMOVE_ARTIST: &str =
    r"DELETE FROM artists WHERE name=(:name) AND site=(:site)";

pub const SELECT_PRODUCT: &str =
    r"SELECT 1 FROM products WHERE url = (:url)";

pub const SELECT_PRODUCTS: &str =
    r"SELECT url, title, artist, imgUrl, dateAdded, availability FROM products WHERE site=(:site) ORDER BY dateAdded DESC, artist ASC";

pub const INSERT_PRODUCT: &str =
    r"INSERT IGNORE INTO products (url, title, artist, site, imgUrl, dateAdded, availability) VALUES (:url, :title, :artist, :site, :img_url, :date_added, :availability)";

/*pub const REMOVE_PRODUCT: &str =
    r"DELETE FROM products WHERE url=(:url)";*/

pub const UPDATE_PRODUCT: &str =
    r"UPDATE products SET title = (:title), artist = (:artist), imgUrl = (:img_url), dateAdded = (:date_added), availability = (:availability) WHERE url = (:url)";

pub const INSERT_SKIP_PRODUCT: &str =
    r"INSERT IGNORE INTO skip_products (url) VALUES (:url)";

pub const SELECT_SKIP_PRODUCT: &str =
    r"SELECT 1 FROM skip_products WHERE url = (:url)";