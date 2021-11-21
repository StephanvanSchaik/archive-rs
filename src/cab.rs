use std::borrow::Cow;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

use crate::{Archive, Entry, Entries, LendingIterator};
use crate::error::Error;

pub struct FileEntry {
    path: PathBuf,
    name: String,
}

pub struct CabEntry<'a, R> {
    entry: &'a FileEntry,
    reader: cab::FileReader<'a, R>,
}

impl<'a, R: Read + Seek> Entry for CabEntry<'a, R> {
    fn path(&self) -> Result<Cow<Path>, Error> {
        Ok(Cow::Borrowed(&self.entry.path))
    }
}

impl<'a, R: Read + Seek> Read for CabEntry<'a, R> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize, std::io::Error> {
        self.reader.read(bytes)
    }
}

pub struct CabEntries<'a, R> {
    archive: &'a mut cab::Cabinet<R>,
    iter: std::slice::Iter<'a, FileEntry>,
}

impl<'c, R: Read + Seek> LendingIterator for CabEntries<'c, R> {
    type Item<'a> where 'c: 'a = Result<Box<dyn Entry + 'a>, Error>;

    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>> {
        let entry = match self.iter.next() {
            Some(entry) => entry,
            None => return None,
        };

        let reader = match self.archive.read_file(&entry.name) {
            Ok(reader) => reader,
            Err(e) => return Some(Err(Error::Io(e))),
        };

        Some(Ok(Box::new(CabEntry {
            entry,
            reader,
        })))
    }
}

pub struct CabArchive<R> {
    archive: cab::Cabinet<R>,
    entries: Vec<FileEntry>
}

impl<R: Read + Seek> CabArchive<R> {
    pub fn new(reader: R) -> Result<Self, Error> {
        let archive = cab::Cabinet::new(reader)?;
        let mut entries = vec![];

        for folder_entry in archive.folder_entries() {
            for file_entry in folder_entry.file_entries() {
                entries.push(FileEntry {
                    path: PathBuf::from(file_entry.name()),
                    name: file_entry.name().to_string(),
                });
            }
        }

        Ok(Self {
            archive,
            entries,
        })
    }
}

impl<R: Read + Seek> Archive<R> for CabArchive<R> {
    fn entries<'a>(&'a mut self) -> Result<Entries<'a, R>, Error> {
        Ok(Entries::Cab(CabEntries {
            archive: &mut self.archive,
            iter: self.entries.iter(),
        }))
    }
}
