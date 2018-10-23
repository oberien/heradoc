use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

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
        let header = response.headers().get(header::CONTENT_TYPE);
        let header = header.and_then(|value| value.to_str().ok());

        match header {
            Some("text/markdown") => Some(ContentType::Markdown),
            Some("image/png") | Some("image/jpeg") => Some(ContentType::Image),
            Some("application/pdf") => Some(ContentType::Pdf),
            _ => None,
        }
    }

    fn target_path(&self, url: &Url) -> PathBuf {
        let mut target = self.temp.clone();

        // http(s) domains must contain a hostname
        target.push(url.host_str().unwrap());

        // http(s) must not be cannot-be-base
        //
        // Also, '+' is a reserved character that can not appear unescaped.
        let path = url.path().replace('/', "+");
        let path = Path::new(&path);

        // Since pdflatex is picky with file extensions, replace all dots.
        let stem = path.file_stem()
            // `path` already was valid utf-8
            .map(|osstr| osstr.to_str().unwrap())
            // file_stem is a part of the file_name, which exists if the last component is not `..`
            // This would not make sense to handle right now.
            .expect("url path should not be empty")
            // Replace all preceding dots, since some consumers (`pdflatex`) do not expect that.
            .replace('.', "+");

        target.push(stem);

        if let Some(extension) = path.extension() {
            target.set_extension(extension);
        }

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
