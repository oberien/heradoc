use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use mime;
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

        let path = self.target_path(&url);

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

        let content_type = self.content_type(&response);

        Ok(Downloaded {
            file,
            path,
            content_type,
        })
    }

    fn content_type(&self, response: &Response) -> Option<ContentType> {
        // Get a value content type header if any.
        let header = response.headers().get(header::CONTENT_TYPE)
            .and_then(|raw| raw.to_str().ok())
            .and_then(|string| string.parse::<mime::Mime>().ok())?;

        match (header.type_(), header.subtype()) {
            (mime::TEXT, sub) if sub == "markdown" => Some(ContentType::Markdown),
            (mime::IMAGE, mime::PNG) | (mime::IMAGE, mime::JPEG) => Some(ContentType::Image),
            (mime::APPLICATION, sub) if sub == "pdf" => Some(ContentType::Pdf),
            // Let the file extension logic take over.
            _ => None,
        }
    }

    /// Injectively map urls to paths in the download directory.
    fn target_path(&self, url: &Url) -> PathBuf {
        let mut target = self.temp.clone();

        // http(s) domains must contain a hostname
        target.push(url.host_str().unwrap());

        // http(s) must not be cannot-be-base.  Also, '+' is a reserved character that can not
        // appear unescaped and "+0" is a unique sequence to this replacement.
        let path = url.path().replace('/', "+0");
        let path = Path::new(&path);

        // Since pdflatex is picky with file extensions, replace all dots.
        let mut stem = path.file_stem()
            // `path` already was valid utf-8
            .map(|osstr| osstr.to_str().unwrap())
            // file_stem is a part of the file_name, which exists if the last component is not `..`
            // This would not make sense to handle right now.
            .expect("url path should not be empty")
            // Replace all preceding dots, since some consumers (`pdflatex`) do not expect that.
            // The sequence "+1" is unique to this kind of replacement.
            .replace('.', "+1");

        // `PathBuf::set_extension` does not do anything when the extension is empty. This would
        // resolve files ending in `.` to the same path as without. Instead, we correct the
        // extension manually.
        if let Some(extension) = path.extension() {
            stem.push('.');
            // The extension is always valid utf-8
            stem.push_str(extension.to_str().unwrap());
        }

        target.push(stem);

        target
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
    use super::Remote;
    use tempdir::TempDir;

    #[test]
    fn download_paths() {
        let dir = TempDir::new("pundoc-remote-test")
            .expect("Can't create tempdir");
        let remote = Remote::new(dir.path().to_path_buf()).unwrap();
        let top_level_path = remote.target_path(&"https://example.com/".parse().unwrap());
        let some_file = remote.target_path(&"https://example.com/index.html".parse().unwrap());
        let path_with_dir = remote.target_path(&"https://example.com/subsite/index.html".parse().unwrap());
        
        // Ensure that the temp dir has a parent relationship with downloaded file.
        assert!(top_level_path.parent().unwrap().starts_with(dir.path()));
        
        // Make sure that even the top level was placed in the same directory as other files.
        assert_eq!(top_level_path.parent(), some_file.parent());

        // Test that folders in the path don't create an actual hierarchy.
        assert_eq!(top_level_path.parent(), path_with_dir.parent());
    }

    #[test]
    fn path_injectivity() {
        let dir = TempDir::new("pundoc-remote-test")
            .expect("Can't create tempdir");
        let remote = Remote::new(dir.path().to_path_buf()).unwrap();

        // Test no extension vs. empty extension.
        let a = remote.target_path(&"https://example.com/a".parse().unwrap());
        let b = remote.target_path(&"https://example.com/a.".parse().unwrap());
        assert!(a != b, "Two urls with the same underlying file path: {:?} {:?}", a, b);

        // Test character replacement '/' vs '.' within the path.
        let a = remote.target_path(&"https://example.com/a/b.jpg".parse().unwrap());
        let b = remote.target_path(&"https://example.com/a.b.jpg".parse().unwrap());
        assert!(a != b, "Two urls with the same underlying file path");

        // Test character replacement '/' vs an existing '+' within the path.
        let a = remote.target_path(&"https://example.com/a/b.jpg".parse().unwrap());
        let b = remote.target_path(&"https://example.com/a+b.jpg".parse().unwrap());
        assert!(a != b, "Two urls with the same underlying file path");
    }
}
