// APK file is just Zip file
extern crate zip;
use std::fs;
use std::path::Path;
use std::io::Error;
use file::axml::{Axml, AxmlError};

// Error for APK package
#[derive(Debug)]
pub enum ApkError {
    // wrap basic io error
    IoError(Error),
    ZipError(zip::result::ZipError),
    AxmlError(AxmlError)
}

// record io error in apk error
impl From<Error> for ApkError {
    fn from(e: Error) -> ApkError {
        ApkError::IoError(e)
    }
}

impl From<zip::result::ZipError> for ApkError {
    fn from(e: zip::result::ZipError) -> ApkError {
        ApkError::ZipError(e)
    }
}

impl From<AxmlError> for ApkError {
    fn from(e: AxmlError) -> ApkError {
        ApkError::AxmlError(e)
    }
}

// define my result type
pub type ApkResult<T> = Result<T, ApkError>;

pub struct Apk {
}

// open my apk file
pub fn open(file_path: &String) -> ApkResult<Apk> {
    // open file
    let path = Path::new(file_path);
    let file = fs::File::open(path)?;

    let mut archive = zip::ZipArchive::new(file)?;

    // try to find AndroidManifest.xml
    let manifest = Axml::read(&mut archive.by_name("AndroidManifest.xml")?)?;

    Ok(Apk {

    })
}