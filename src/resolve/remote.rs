use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use mime::Mime;
use sha2::{Digest, Sha256};
// TODO: The error representation is awkward with reqwest, evaluate cHTTP instead.
use reqwest::{Client, Error as RequestError, RedirectPolicy, Response, header};
use url::Url;

/// Provide access to remotely hosted resources.
pub struct Remote {
    client: Client,
    temp: PathBuf,
}

pub struct Downloaded {
    file: File,
    path: PathBuf,
    content_type: Option<ContentType>,
}

#[derive(Debug)]
pub enum Error {
    Request(RequestError),
    Io(io::Error),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentType {
    Markdown,
    Image,
    Pdf,
}

/// Unique information identifying the same resource.
///
/// Used to calculate a hash-based unique path in the download directory, as a preparation for
/// possible caching. TODO: when caching is implemented we need to store enough information to
/// serve this key information instead of the response.
struct FileKey {
    host: String,
    path: PathBuf,
    mime: Option<Mime>,
}

impl Remote {
    pub fn new(download_folder: PathBuf) -> Result<Self, Error> {
        fs::create_dir_all(&download_folder)?;

        let client = Client::builder()
            // TODO: how should redirects interact relative references etc. ?
            // Also consider that redirects could influence the injectivity of `target_path`
            .redirect(RedirectPolicy::none())
            .build()?;

        Ok(Remote {
            temp: download_folder,
            client,
        })
    }

    pub fn http(&self, url: Url) -> Result<Downloaded, Error> {
        let mut response = self.client.get(url.as_ref())
            // TODO set headers: user agent, accepted content type, ...
            .send()
            .and_then(|response| response.error_for_status())?;

        let key = FileKey::from(&response);
        let path = self.target_path(&key);

        fs::create_dir_all(path.parent().unwrap())?;

        // Replace whatever file already existed.
        //
        // Doing this in two steps instead of create+truncate keeps the file unmodified for
        // processes that already own the old file handle. The old file is merely unlinked.
        // TODO: proper caching
        let _ = fs::remove_file(&path);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;

        io::copy(&mut response, &mut file)?;

        let content_type = key.content_type();

        Ok(Downloaded {
            file,
            path,
            content_type,
        })
    }

    /// Injectively map urls to paths in the download directory.
    fn target_path(&self, key: &FileKey) -> PathBuf {
        let extension = key.path.extension();

        let file_hash = Sha256::new()
            .chain(key.host.as_str())
            .chain(key.path.to_str().unwrap())
            .chain(key.mime.as_ref().map(Mime::as_ref).unwrap_or(""))
            .result();

        let mut hash_name = format!("{:x}", file_hash);

        if let Some(extension) = extension {
            hash_name.push('.');
            hash_name.push_str(extension.to_str().unwrap());
        }

        let mut target = self.temp.clone();
        target.push(&key.host);
        target.push(hash_name);

        target
    }
}

impl FileKey {
    fn from(response: &Response) -> Self {
        let mime = response.headers().get(header::CONTENT_TYPE)
            .and_then(|raw| raw.to_str().ok())
            .and_then(|string| string.parse::<Mime>().ok());

        Self::from_parts(response.url(), mime)
    }

    fn from_parts(url: &Url, mime: Option<Mime>) -> Self {
        let host = url.host_str().unwrap().to_owned();
        let path = Path::new(url.path()).to_owned();

        FileKey {
            host,
            path,
            mime,
        }
    }

    fn content_type(&self) -> Option<ContentType> {
        let mime = self.mime.as_ref()?;

        match (mime.type_(), mime.subtype()) {
            (mime::TEXT, sub) if sub == "markdown" => Some(ContentType::Markdown),
            (mime::IMAGE, mime::PNG) | (mime::IMAGE, mime::JPEG) => Some(ContentType::Image),
            (mime::APPLICATION, sub) if sub == "pdf" => Some(ContentType::Pdf),
            // Let the file extension logic take over.
            _ => None,
        }
    }
}

impl Downloaded {
    /// The file path into which the downloaded data was written.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Indicate the content type if the response had a fitting header.
    pub fn content_type(&self) -> Option<ContentType> {
        self.content_type
    }
}

impl From<RequestError> for Error {
    fn from(inner: RequestError) -> Self {
        Error::Request(inner)
    }
}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Self {
        Error::Io(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::{FileKey, Remote};
    use tempdir::TempDir;

    #[test]
    fn download_paths() {
        let dir = TempDir::new("heradoc-remote-test")
            .expect("Can't create tempdir");
        let remote = Remote::new(dir.path().to_path_buf()).unwrap();
        let top_level_path = remote.target_path(
            &FileKey::from_parts(&"https://example.com/".parse().unwrap(), None));
        let some_file = remote.target_path(
            &FileKey::from_parts(&"https://example.com/index.html".parse().unwrap(), None));
        let path_with_dir = remote.target_path(
            &FileKey::from_parts(&"https://example.com/subsite/index.html".parse().unwrap(), None));
        
        // Ensure that the temp dir has a parent relationship with downloaded file.
        assert!(top_level_path.parent().unwrap().starts_with(dir.path()));
        
        // Make sure that even the top level was placed in the same directory as other files.
        assert_eq!(top_level_path.parent(), some_file.parent());

        // Test that folders in the path don't create an actual hierarchy.
        assert_eq!(top_level_path.parent(), path_with_dir.parent());
    }
}
