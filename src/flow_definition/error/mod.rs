use std::io;
use std::path::PathBuf;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ReaderError {
    JsonError {
        path: PathBuf,
        error: serde_json::Error,
    },
    ReadFeatureError {
        path: String,
        source: Box<ReaderError>,
    },
    ReadDirectoryError {
        path: PathBuf,
        error: io::Error,
    },
    ReadFileError {
        path: PathBuf,
        error: io::Error,
    },
    DirectoryEntryError(io::Error),
}
