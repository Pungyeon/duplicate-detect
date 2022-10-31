use std::collections::{hash_map, HashMap};

pub struct FileIndex {
    pub(crate) files: HashMap<String, DuplicateMeta>,
    pub(crate) count: i64,
    pub(crate) dupe_size: u64,
}

impl FileIndex {
    pub(crate) fn new() -> Self {
        FileIndex{
            files: HashMap::new(),
            count: 0,
            dupe_size: 0,
        }
    }

    pub(crate) fn increment(&mut self) {
        self.count += 1;
    }

    fn duplication_size_increment(&mut self, size: u64) {
        self.dupe_size += size;
    }

    pub(crate) fn insert(&mut self, hash: String, filepath: String, filesize: u64) {
        if let hash_map::Entry::Vacant(e) = self.files.entry(hash.clone()) {
            e.insert(DuplicateMeta::new(filepath));
        } else {
            self.files.get_mut(&hash).unwrap().duplicates.push(filepath);
            self.duplication_size_increment(filesize);
        }
    }
}

pub(crate) struct DuplicateMeta {
    pub(crate) filename: String,
    pub(crate) duplicates: Vec<String>,
}

impl DuplicateMeta {
    fn new(filename: String) -> Self {
        DuplicateMeta{
            filename,
            duplicates: Vec::new(),
        }
    }
}

