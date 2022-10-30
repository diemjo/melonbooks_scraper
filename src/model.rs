use core::fmt;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use chrono::{NaiveDate};
use crate::common::error::Error;
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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
    pub artist: String,
    pub img_url: String,
    pub date_added: NaiveDate, //utc
    pub availability: Availability,
}

impl Product {
    pub(crate) fn new(url: String, title: String, artist: String, img_url: String, date_added: NaiveDate, availability: Availability) -> Self {
        Product { url, title, artist, img_url, date_added, availability }
    }
}

impl fmt::Display for Product {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}