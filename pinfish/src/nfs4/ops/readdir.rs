use super::{Cookie4, Verifier4, Count4, Bitmap4, Component4, FileAttributes};
use pinfish_macros::{PackTo, UnpackFrom};
use crate::xdr;


#[derive(PackTo, Debug)]
pub struct ReadDir4Args {
    pub cookie: Cookie4,
    pub verifier: Verifier4,
    pub dir_count: Count4,
    pub max_count: Count4,
    pub attr_request: Bitmap4,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct Entry4 {
    pub cookie: Cookie4,
    pub name: Component4,
    pub attrs: FileAttributes,
    pub next_entry: Option<Box<Entry4>>,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct DirList4 {
    pub entries: Option<Entry4>,
    pub eof: bool,
}

/// Iterator for directory entries
pub struct Entry4Iter<'a> {
    next: Option<&'a Entry4>,
}

impl<'a> Iterator for Entry4Iter<'a> {
    type Item = &'a Entry4;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.next {
            self.next = entry.next_entry.as_ref().map(|e| &**e);
            Some(entry)
        } else {
            None
        }
    }
}

impl DirList4 {
    /// Iterate over directory entries
    pub fn iter(&self) -> Entry4Iter<'_> {
        Entry4Iter {
            next: self.entries.as_ref(),
        }
    }
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct ReadDir4ResOk {
    pub cookie_verf: Verifier4,
    pub reply: DirList4,
}
