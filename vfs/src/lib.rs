use std::{
    collections::HashMap,
    default::Default,
    path::{Path, PathBuf},
};

use indexmap::IndexSet;
use rayon::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct FileId(pub u32);

#[derive(Debug, Default)]
pub struct Interner {
    map: IndexSet<PathBuf>,
}

impl Interner {
    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<FileId> {
        self.map
            .get_index_of(path.as_ref())
            .map(|i| FileId(i as u32))
    }
    pub fn intern(&mut self, path: PathBuf) -> FileId {
        let (id, _) = self.map.insert_full(path);
        FileId(id as u32)
    }
    pub fn lookup(&self, file: FileId) -> Option<&Path> {
        self.map.get_index(file.0 as usize).map(|p| p.as_path())
    }
}

#[derive(Default)]
pub struct ReadOnlyVfs {
    interner: Interner,
    data: HashMap<FileId, Vec<u8>>,
}

impl ReadOnlyVfs {
    pub fn singleton<P: AsRef<Path>>(path: P, contents: &[u8]) -> Self {
        let mut vfs = ReadOnlyVfs::default();
        vfs.set_file_contents(path, contents);
        vfs
    }
    pub fn alloc_file_id<P: AsRef<Path>>(&mut self, path: P) -> FileId {
        self.interner.intern(path.as_ref().to_owned())
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn file_path(&self, file_id: FileId) -> &Path {
        self.interner.lookup(file_id).unwrap()
    }
    pub fn get(&self, file_id: FileId) -> &Vec<u8> {
        self.data.get(&file_id).unwrap()
    }
    pub fn get_str(&self, file_id: FileId) -> &str {
        std::str::from_utf8(self.get(file_id)).unwrap()
    }
    pub fn get_mut(&mut self, file_id: FileId) -> &mut Vec<u8> {
        self.data.get_mut(&file_id).unwrap()
    }
    pub fn set_file_contents<P: AsRef<Path>>(&mut self, path: P, contents: &[u8]) {
        let file_id = self.alloc_file_id(path);
        self.data.insert(file_id, contents.to_owned());
    }
    pub fn iter(&self) -> impl Iterator<Item = VfsEntry> {
        self.data.keys().map(move |file_id| VfsEntry {
            file_id: *file_id,
            file_path: self.file_path(*file_id),
            contents: self.get_str(*file_id),
        })
    }
    pub fn par_iter(&self) -> impl ParallelIterator<Item = VfsEntry> {
        self.data.par_iter().map(move |(file_id, _)| VfsEntry {
            file_id: *file_id,
            file_path: self.file_path(*file_id),
            contents: self.get_str(*file_id),
        })
    }
}

pub struct VfsEntry<'ρ> {
    pub file_id: FileId,
    pub file_path: &'ρ Path,
    pub contents: &'ρ str,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn trivial() {
        let mut vfs = ReadOnlyVfs::default();
        let f1 = "a/b/c";
        let id1 = vfs.alloc_file_id(f1);
        let data = "hello".as_bytes().to_vec();
        vfs.set_file_contents(f1, &data);
        assert_eq!(vfs.get(id1), &data);
    }
}
