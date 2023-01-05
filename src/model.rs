use core::fmt;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use chrono::{NaiveDate};
use rusqlite::Error::FromSqlConversionFailure;
use rusqlite::Row;
use rusqlite::types::Type;
use crate::common::error::{Error};
use crate::model::Availability::{Available, NotAvailable, Preorder};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Availability {
    Available,
    Preorder,
    NotAvailable
}

impl fmt::Display for Availability {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl FromStr for Availability {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Available" => Ok(Available),
            "Preorder" => Ok(Preorder),
            "NotAvailable" => Ok(NotAvailable),
            _ => Err(Error::AvailabilityEnumError(s.into()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Product {
    pub url: String,
    pub title: String,
    pub associated_artist: String,
    pub artists: Vec<String>,
    pub img_url: String,
    pub date_added: NaiveDate, //utc
    pub availability: Availability,
}

impl AsRef<Product> for Product {
    fn as_ref(&self) -> &Product {
        return self;
    }
}

impl Product {
    pub(crate) fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Product::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get::<usize, String>(3)?.split(',').map(|s| s.to_string()).collect(),
            row.get(4)?,
            NaiveDate::from_str(row.get::<usize, String>(5)?.as_str()).unwrap(),
            Availability::from_str(row.get::<usize, String>(6)?.as_str()).or_else(|e| Err(FromSqlConversionFailure(0, Type::Text, Box::new(e))))?
        ))
    }
}

impl Product {
    pub(crate) fn new(url: String, title: String, associated_artist: String, artists: Vec<String>, img_url: String, date_added: NaiveDate, availability: Availability) -> Self {
        Product { url, title, associated_artist, artists, img_url, date_added, availability }
    }
}

impl fmt::Display for Product {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}