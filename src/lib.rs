#![feature(generic_associated_types)]

pub mod error;

#[cfg(feature = "cab")]
pub mod cab;
#[cfg(feature = "tar")]
pub mod tar;
#[cfg(feature = "zip")]
pub mod zip;

use std::borrow::Cow;
use std::io::{Read, Seek};
use std::path::Path;

use crate::error::Error;

pub trait LendingIterator {
    type Item<'a> where Self: 'a;

    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>>;
}

pub trait Entry: Read {
    fn path(&self) -> Result<Cow<Path>, Error>;
}

pub enum Entries<'a, R: Read> {
    #[cfg(feature = "cab")]
    Cab(crate::cab::CabEntries<'a, R>),
    #[cfg(feature = "tar")]
    Tar(crate::tar::TarEntries<'a, R>),
    #[cfg(feature = "zip")]
    Zip(crate::zip::ZipEntries<'a, R>),
}

impl<'a, R: Read + Seek> Iterator for Entries<'a, R> {
    type Item = Result<Box<dyn Entry + 'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Entries::Cab(_) => None,
            Entries::Tar(ref mut entries) => entries.next(),
            Entries::Zip(_) => None,
        }
    }
}

impl<'c, R: Read + Seek> LendingIterator for Entries<'c, R> {
    type Item<'a> where R: 'a, 'c: 'a = Result<Box<dyn Entry + 'a>, Error>;

    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>> {
        match self {
            Entries::Cab(ref mut entries) => entries.next(),
            Entries::Tar(_) => None,
            Entries::Zip(ref mut entries) => entries.next(),
        }
    }
}

pub trait Archive<R: Read> {
    fn entries<'a>(&'a mut self) -> Result<Entries<R>, Error>;
}
