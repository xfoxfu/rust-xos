mod device;
mod directory;
mod file;

pub use device::{BlockError, Device, FatDevice};
pub use directory::Directory;
pub use file::File;

pub enum Entry<'a, T> {
    Dir(Directory<'a, T>),
    File(File<'a, T>),
}

impl<'a, T> From<Directory<'a, T>> for Entry<'a, T> {
    fn from(dir: Directory<'a, T>) -> Self {
        Self::Dir(dir)
    }
}

impl<'a, T> From<File<'a, T>> for Entry<'a, T> {
    fn from(file: File<'a, T>) -> Self {
        Self::File(file)
    }
}

impl<'a, T> core::fmt::Debug for Entry<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Dir(d) => d.fmt(f),
            Self::File(fl) => fl.fmt(f),
        }
    }
}
