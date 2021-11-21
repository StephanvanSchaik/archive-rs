use std::borrow::Cow;
use std::io::{Read, Seek};
use std::path::Path;

use crate::{Archive, Entry, Entries, LendingIterator};
use crate::error::Error;

pub struct ZipEntry<'a> {
    entry: zip::read::ZipFile<'a>,
}

impl<'a> Entry for ZipEntry<'a> {
    fn path(&self) -> Result<Cow<Path>, Error> {
        Ok(Cow::Borrowed(&Path::new(self.entry.name())))
    }
}

impl<'a> Read for ZipEntry<'a> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize, std::io::Error> {
        self.entry.read(bytes)
    }
}

pub struct ZipEntries<'a, R> {
    archive: &'a mut zip::ZipArchive<R>,
    index: usize,
}

impl<'c, R: Read + Seek> LendingIterator for ZipEntries<'c, R> {
    type Item<'a> where 'c: 'a = Result<Box<dyn Entry + 'a>, Error>;

    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>> {
        let index = self.index;
        self.index += 1;

        let entry = match self.archive.by_index(index) {
            Ok(entry) => entry,
            Err(e) => return Some(Err(Error::Zip(e))),
        };

        Some(Ok(Box::new(ZipEntry {
            entry,
        })))
    }
}

pub struct ZipArchive<R> {
    archive: zip::ZipArchive<R>,
}

impl<R: Read + Seek> ZipArchive<R> {
    pub fn new(reader: R) -> Result<Self, Error> {
        Ok(Self {
            archive: zip::ZipArchive::new(reader)?,
        })
    }
}

impl<R: Read + Seek> Archive<R> for ZipArchive<R> {
    fn entries<'a>(&'a mut self) -> Result<Entries<'a, R>, Error> {
        Ok(Entries::Zip(ZipEntries {
            archive: &mut self.archive,
            index: 0,
        }))
    }
}
