use std::borrow::Cow;
use std::io::{Read};
use std::path::Path;

use crate::{Archive, Entry, Entries};
use crate::error::Error;

pub struct TarEntry<'a, R: Read> {
    entry: tar::Entry<'a, R>,
}

impl<'a, R: Read> Entry for TarEntry<'a, R> {
    fn path(&self) -> Result<Cow<Path>, Error> {
        Ok(self.entry.path()?)
    }
}

impl<'a, R: Read> Read for TarEntry<'a, R> {
    fn read(&mut self, bytes: &mut [u8]) -> Result<usize, std::io::Error> {
        self.entry.read(bytes)
    }
}

pub struct TarEntries<'a, R: Read> {
    entries: tar::Entries<'a, R>,
}

impl<'a, R: Read> Iterator for TarEntries<'a, R> {
    type Item = Result<Box<dyn Entry + 'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = match self.entries.next() {
            Some(Ok(entry)) => entry,
            Some(Err(e)) => return Some(Err(Error::Io(e))),
            None => return None,
        };

        Some(Ok(Box::new(TarEntry {
            entry,
        })))
    }
}

/* FIXME: this does not work due to lifetime issues.
impl<'c, R: Read + Seek> LendingIterator for TarEntries<'c, R> {
    type Item<'a> where R: 'a, 'c: 'a = Result<Box<TarEntry<'a, R>>, Error>;

    fn next<'b>(&'b mut self) -> Option<Self::Item<'b>> {
        let entry = match self.entries.next() {
            Some(Ok(entry)) => entry,
            Some(Err(e)) => return Some(Err(Error::Io(e))),
            None => return None,
        };

        Some(Ok(Box::new(TarEntry {
            entry,
        })))
    }
}
*/

pub struct TarArchive<R: Read> {
    archive: tar::Archive<R>,
}

impl<R: Read> TarArchive<R> {
    pub fn new(reader: R) -> Result<Self, Error> {
        Ok(Self {
            archive: tar::Archive::new(reader),
        })
    }
}

impl<R: Read> Archive<R> for TarArchive<R> {
    fn entries<'a>(&'a mut self) -> Result<Entries<'a, R>, Error> {
        Ok(Entries::Tar(TarEntries {
            entries: self.archive.entries()?,
        }))
    }
}

#[cfg(feature = "bzip2")]
pub struct Bzip2TarArchive<R: Read> {
    archive: tar::Archive<bzip2::read::BzDecoder<R>>,
}

#[cfg(feature = "bzip2")]
impl<R: Read> Bzip2TarArchive<R> {
    pub fn new(reader: R) -> Result<Self, Error> {
        let decoder = bzip2::read::BzDecoder::new(reader);
        let archive = tar::Archive::new(decoder);

        Ok(Self {
            archive,
        })
    }
}

#[cfg(feature = "bzip2")]
impl<R: Read> Archive<R> for Bzip2TarArchive<R> {
    fn entries<'a>(&'a mut self) -> Result<Entries<'a, R>, Error> {
        Ok(Entries::Bzip2Tar(TarEntries {
            entries: self.archive.entries()?,
        }))
    }
}

#[cfg(feature = "gzip")]
pub struct GzipTarArchive<R: Read> {
    archive: tar::Archive<flate2::read::GzDecoder<R>>,
}

#[cfg(feature = "gzip")]
impl<R: Read> GzipTarArchive<R> {
    pub fn new(reader: R) -> Result<Self, Error> {
        let decoder = flate2::read::GzDecoder::new(reader);
        let archive = tar::Archive::new(decoder);

        Ok(Self {
            archive,
        })
    }
}

#[cfg(feature = "gzip")]
impl<R: Read> Archive<R> for GzipTarArchive<R> {
    fn entries<'a>(&'a mut self) -> Result<Entries<'a, R>, Error> {
        Ok(Entries::GzipTar(TarEntries {
            entries: self.archive.entries()?,
        }))
    }
}

#[cfg(feature = "lzma")]
pub struct LzmaTarArchive<R: Read> {
    archive: tar::Archive<lzma::reader::LzmaReader<R>>,
}

#[cfg(feature = "lzma")]
impl<R: Read> LzmaTarArchive<R> {
    pub fn new(reader: R) -> Result<Self, Error> {
        let decoder = lzma::LzmaReader::new_decompressor(reader)?;
        let archive = tar::Archive::new(decoder);

        Ok(Self {
            archive,
        })
    }
}

#[cfg(feature = "lzma")]
impl<R: Read> Archive<R> for LzmaTarArchive<R> {
    fn entries<'a>(&'a mut self) -> Result<Entries<'a, R>, Error> {
        Ok(Entries::LzmaTar(TarEntries {
            entries: self.archive.entries()?,
        }))
    }
}
