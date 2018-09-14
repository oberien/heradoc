use std::io;
use std::path::{PathBuf, Path, Component};

use url::{Url, Host};

use resolve::{Resource, ResourceBuilder, Source, InnerSource, Include, Command};

// We Java now
pub struct ResourceProviderProvider {
    locals: Locals,
    remotes: Remotes,
    files: Files,
    toc: Toc,
}

impl ResourceProviderProvider {
    pub fn new(base: PathBuf) -> ResourceProviderProvider {
        ResourceProviderProvider {
            locals: Locals,
            remotes: Remotes,
            files: Files::new(base),
            toc: Toc,
        }
    }

    pub fn get_provider(&self, target: &Source) -> Option<&dyn ResourceProvider> {
        match &target.inner {
            InnerSource::Local(_) => Some(&self.locals),
            InnerSource::Remote(_) => Some(&self.remotes),
            InnerSource::Implementation(url) => match url.host() {
                Some(Host::Domain("document")) => Some(&self.files),
                Some(Host::Domain("toc")) => Some(&self.toc),
                _ => None
            }
        }
    }
}


pub trait ResourceProvider {
    fn build(&self, target: Url, builder: ResourceBuilder) -> io::Result<Resource>;
}

pub struct Toc;

impl ResourceProvider for Toc {
    fn build(&self, _target: Url, builder: ResourceBuilder) -> io::Result<Resource> {
        Ok(builder.build(Include::Command(Command::Toc)))
    }
}

pub struct Files {
    base: PathBuf,
}

impl Files {
    pub fn new(base: PathBuf) -> Files {
        Files {
            base
        }
    }
}

impl ResourceProvider for Files {
    fn build(&self, mut target: Url, builder: ResourceBuilder) -> io::Result<Resource> {
        // Normalize the url to interpret the serialization as a relative path.

        // Host can not be cleared from some special schemes, so normalize the scheme first.
        target.set_scheme("pundoc").unwrap();
        // Clear the host
        target.set_host(None).unwrap();
        target.set_query(None);
        target.set_fragment(None);

        // Url is now: `pundoc:<absolute path>`, e.g. `pundoc:/main.md`
        let path = target.into_string();
        assert_eq!(&path[..8], "pundoc:/");
        let path = Path::new(&path[8..]);

        let downwards = path.components()
            .filter_map(|component| match component {
                Component::Normal(os) => Some(os),
                _ => None,
            });
        let mut output_path = self.base.clone();
        output_path.extend(downwards);

        Ok(builder.with_path(output_path))
    }
}

pub struct Locals;

impl ResourceProvider for Locals {
    fn build(&self, target: Url, builder: ResourceBuilder) -> io::Result<Resource> {
        let path = target.to_file_path().unwrap();
        Ok(builder.with_path(path))
    }
}


pub struct Remotes;

impl ResourceProvider for Remotes {
    fn build(&self, _target: Url, _builder: ResourceBuilder) -> io::Result<Resource> {
        unimplemented!("Remote urls can not yet be used inside documents")
    }
}

