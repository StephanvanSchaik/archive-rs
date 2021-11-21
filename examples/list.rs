use archive_rs::{Archive, LendingIterator};
use archive_rs::error::Error;
use clap::Parser;

#[derive(Parser)]
#[clap(version = "1.0", author = "Stephan van Schaik <stephan@synkhronix.com")]
struct Options {
    input: String,
}

fn main() -> Result<(), Error> {
    let options: Options = Options::parse();

    let mut archives = archive_rs::open(&options.input)?;
    let mut entries = archives.entries()?;

    while let Some(entry) = Iterator::next(&mut entries) {
        let entry = entry?;
        let path = entry.path()?;
        println!("{:?}", path);
    }

    while let Some(entry) = LendingIterator::next(&mut entries) {
        let entry = entry?;
        let path = entry.path()?;
        println!("{:?}", path);
    }

    Ok(())
}
