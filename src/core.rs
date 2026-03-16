use std::{
    ffi::OsStr,
    fs::canonicalize,
    path::{
        Path as StdPath,
        PathBuf,
    },
};

use anyhow::Result;
use path_absolutize::Absolutize;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Path(pub(crate) PathBuf);

impl Path {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self(path.into())
    }

    pub fn absolutize(&self) -> Result<Self> {
        Ok(Self::new(self.0.absolutize()?))
    }

    pub fn absolutize_from(&self, cwd: impl AsRef<StdPath>) -> Result<Self> {
        Ok(Self::new(self.0.absolutize_from(cwd)?))
    }

    pub fn absolutize_virtually(&self, virtual_root: impl AsRef<StdPath>) -> Result<Self> {
        Ok(Self::new(self.0.absolutize_virtually(virtual_root)?))
    }

    pub fn as_path(&self) -> &StdPath {
        &self.0
    }

    pub fn canonicalize(&self) -> Result<Self, std::io::Error> {
        canonicalize(&self.0).map(Self::new)
    }

    pub fn join(&self, path: impl AsRef<StdPath>) -> Self {
        Self::new(self.0.join(path))
    }

    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(Self::new)
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }

    pub fn with_extension<S: AsRef<OsStr>>(&self, extension: S) -> Self {
        Self::new(self.0.with_extension(extension))
    }
}
