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
        CONSTRAINT fk_artist_name
            FOREIGN KEY (artist, site) REFERENCES artists (name, site)
            ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS product_artists (
        url VARCHAR(128) NOT NULL,
        artist VARCHAR(64) NOT NULL,
        PRIMARY KEY (url, artist),
        CONSTRAINT fk_url
            FOREIGN KEY (url) REFERENCES products (url)
            ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS skip_products (
        url VARCHAR(128) NOT NULL,
        artist VARCHAR(64) NOT NULL,
        PRIMARY KEY (url, artist)
    );

    CREATE TABLE IF NOT EXISTS title_skip_sequences (
        artist VARCHAR(64) NOT NULL,
        site VARCHAR(32) NOT NULL,
        sequence VARCHAR(256) NOT NULL,
        PRIMARY KEY (artist, site, sequence),
        CONSTRAINT fk_artist_name
            FOREIGN KEY (artist, site) REFERENCES artists (name, site)
            ON DELETE CASCADE
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
    r"SELECT name
    FROM artists
    WHERE site = (:site)
    ORDER BY name ASC";

pub const INSERT_ARTIST: &str =
    r"INSERT INTO artists (name, site)
    VALUES (:name, :site)";

pub const REMOVE_ARTIST: &str =
    r"DELETE FROM artists
    WHERE name=(:name)
    AND site=(:site)";

pub const SELECT_PRODUCT: &str =
    r"SELECT 1
    FROM products
    WHERE url = (:url)";

pub const SELECT_AVAILABILITY_PRODUCT: &str =
    r"SELECT 1
    FROM products
    WHERE url = (:url)
    AND availability = (:availability)";

pub const SELECT_PRODUCTS: &str =
    r"SELECT p.url, p.title, p.artist, group_concat(pa.artist), p.imgUrl, p.dateAdded, p.availability
    FROM products p
    JOIN product_artists pa ON p.url = pa.url 
    WHERE site=(:site)
    GROUP BY p.url
    ORDER BY p.dateAdded DESC, p.artist ASC";

pub const INSERT_PRODUCT: &str =
    r"INSERT INTO products (url, title, artist, site, imgUrl, dateAdded, availability)
    VALUES (:url, :title, :artist, :site, :img_url, :date_added, :availability)";

pub const INSERT_PRODUCT_ARTIST: &str =
    r"INSERT OR IGNORE INTO product_artists (url, artist)
    VALUES (:url, :artist)";


#[cfg(test)]
pub const REMOVE_PRODUCT: &str = 
    r"DELETE FROM products
    WHERE url=(:url)";

pub const UPDATE_PRODUCT_AVAILABILITY: &str =
    r"UPDATE products
    SET availability = (:availability)
    WHERE url = (:url)";

pub const INSERT_SKIP_PRODUCT: &str =
    r"INSERT INTO skip_products (url, artist)
    VALUES (:url, :artist)";

pub const SELECT_SKIP_PRODUCT: &str =
    r"SELECT 1 FROM skip_products
    WHERE url = (:url)";

pub const REMOVE_SKIP_PRODUCTS: &str =
    r"DELETE FROM skip_products
    WHERE url in (
        SELECT url
        FROM skip_products
        WHERE artist = (:artist)
    )";

#[cfg(test)]
pub const INSERT_TITLE_SKIP_SEQUENCE: &str =
    r"INSERT INTO title_skip_sequences(artist, site, sequence)
    VALUES (:artist, :site, :sequence)";

pub const SELECT_TITLE_CONTAINS_SKIP_SEQUENCES: &str =
    r"SELECT 1 FROM title_skip_sequences
    WHERE artist = (:artist)
    AND site = (:site)
    AND (:title) like '%' || sequence || '%'";