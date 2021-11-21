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
    #[cfg(all(feature = "tar", feature = "bzip2"))]
    Bzip2Tar(crate::tar::TarEntries<'a, bzip2::read::BzDecoder<R>>),
    #[cfg(all(feature = "tar", feature = "gzip"))]
    GzipTar(crate::tar::TarEntries<'a, flate2::read::GzDecoder<R>>),
    #[cfg(all(feature = "tar", feature = "lzma"))]
    LzmaTar(crate::tar::TarEntries<'a, lzma::reader::LzmaReader<R>>),
    #[cfg(feature = "zip")]
    Zip(crate::zip::ZipEntries<'a, R>),
}

impl<'a, R: Read + Seek> Iterator for Entries<'a, R> {
    type Item = Result<Box<dyn Entry + 'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Entries::Cab(_) => None,
            Entries::Tar(ref mut entries) => entries.next(),
            Entries::Bzip2Tar(ref mut entries) => entries.next(),
            Entries::GzipTar(ref mut entries) => entries.next(),
            Entries::LzmaTar(ref mut entries) => entries.next(),
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
            Entries::Bzip2Tar(_) => None,
            Entries::GzipTar(_) => None,
            Entries::LzmaTar(_) => None,
            Entries::Zip(ref mut entries) => entries.next(),
        }
    }
}

pub trait Archive<R: Read> {
    fn entries<'a>(&'a mut self) -> Result<Entries<R>, Error>;
}

use regex::bytes::Regex;
use ouroboros::self_referencing;
use std::io::Cursor;
use std::fs::File;

#[self_referencing]
pub struct ArchiveIter {
    re: Regex,
    bytes: Vec<u8>,
    #[not_covariant]
    #[borrows(re, bytes)]
    matches: regex::bytes::Matches<'this, 'this>,
}

impl LendingIterator for ArchiveIter {
    type Item<'a> = Result<Box<dyn Archive<Cursor<&'a [u8]>> + 'a>, Error>;

    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>> {
        let m = self.with_matches_mut(|matches| {
            matches.next().map(|m| m.range())
        });

        let m = match m {
            Some(m) => m,
            None => return None,
        };

        let file = Cursor::new(&self.borrow_bytes()[m.start..]);
        let archive = match crate::cab::CabArchive::new(file) {
            Ok(archive) => archive,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok(Box::new(archive)))
    }
}

pub enum Archives<R: Read> {
    #[cfg(feature = "cab")]
    Cab(crate::cab::CabArchive<R>),
    #[cfg(feature = "tar")]
    Tar(crate::tar::TarArchive<R>),
    #[cfg(all(feature = "tar", feature = "bzip2"))]
    Bzip2Tar(crate::tar::Bzip2TarArchive<R>),
    #[cfg(all(feature = "tar", feature = "gzip"))]
    GzipTar(crate::tar::GzipTarArchive<R>),
    #[cfg(all(feature = "tar", feature = "lzma"))]
    LzmaTar(crate::tar::LzmaTarArchive<R>),
    #[cfg(feature = "zip")]
    Zip(crate::zip::ZipArchive<R>),
    Multiple(ArchiveIter),
}

impl<R: Read + Seek> Archive<R> for Archives<R> {
    fn entries<'a>(&'a mut self) -> Result<Entries<R>, Error> {
        match self {
            Self::Cab(ref mut archive) => archive.entries(),
            Self::Tar(ref mut archive) => archive.entries(),
            Self::Bzip2Tar(ref mut archive) => archive.entries(),
            Self::GzipTar(ref mut archive) => archive.entries(),
            Self::LzmaTar(ref mut archive) => archive.entries(),
            Self::Zip(ref mut archive) => archive.entries(),
            _ => unreachable!(),
        }
    }
}

pub fn open<P>(path: P) -> Result<Archives<File>, Error>
where
    P: AsRef<Path>
{
    let info = infer::get_from_path(path.as_ref())?;

    if let Some(info) = info {
        match info.mime_type() {
            #[cfg(feature = "cab")]
            "application/vnd.ms-cab-compressed" => {
                let file = File::open(path)?;

                return Ok(Archives::Cab(crate::cab::CabArchive::new(file)?));
            }
            #[cfg(feature = "tar")]
            "application/x-tar" => {
                let file = File::open(path)?;

                return Ok(Archives::Tar(crate::tar::TarArchive::new(file)?));
            }
            #[cfg(all(feature = "tar", feature = "bzip2"))]
            "application/x-bzip2" => {
                let file = File::open(path)?;

                return Ok(Archives::Bzip2Tar(crate::tar::Bzip2TarArchive::new(file)?));
            }
            #[cfg(all(feature = "tar", feature = "gzip"))]
            "application/gzip" => {
                let file = File::open(path)?;

                return Ok(Archives::GzipTar(crate::tar::GzipTarArchive::new(file)?));
            }
            #[cfg(all(feature = "tar", feature = "lzma"))]
            "application/x-xz" => {
                let file = File::open(path)?;

                return Ok(Archives::LzmaTar(crate::tar::LzmaTarArchive::new(file)?));
            }
            #[cfg(feature = "zip")]
            "application/zip" => {
                let file = File::open(path)?;

                return Ok(Archives::Zip(crate::zip::ZipArchive::new(file)?));
            }
            _ => (),
        }
    }

    // The file format is not known, make a best effort and look for archives that can be embedded
    // within other file formats.
    let bytes = std::fs::read(path)?;

    let re = Regex::new(r"MSCF").unwrap();

    Ok(Archives::Multiple(ArchiveIterBuilder {
        re,
        bytes,
        matches_builder: |re, bytes| re.find_iter(bytes),
    }.build()))
}
