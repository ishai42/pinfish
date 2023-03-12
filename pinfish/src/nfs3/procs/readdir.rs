use crate::{
    nfs3::{Count3, NfsFh3, Cookie3, PostOpAttributes, Verifier3, FileId3, Filename3},
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct Readdir3Args {
    pub dir: NfsFh3,
    pub cookie: Cookie3,
    pub verifier: Verifier3,
    pub count: Count3,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct Entry3 {
    pub fileid: FileId3,
    pub name: Filename3,
    pub cookie: Cookie3,
    pub next_entry: Option<Box<Entry3>>,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct DirList3 {
    pub entries: Option<Entry3>,
    pub eof: bool,
}


#[derive(PackTo, UnpackFrom, Debug)]
pub struct Readdir3ResOk {
    pub dir_attributes: PostOpAttributes,
    pub verifier: Verifier3,
    pub reply: DirList3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct Readdir3ResFail {
    pub dir_attributes: PostOpAttributes,
}

pub type ReaddirResult = Result<Readdir3ResOk, (u32, Readdir3ResFail)>;

/// Iterator for directory entries
pub struct Entry3Iter<'a> {
    next: Option<&'a Entry3>,
}

impl<'a> Iterator for Entry3Iter<'a> {
    type Item = &'a Entry3;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.next {
            self.next = entry.next_entry.as_ref().map(|e| &**e);
            Some(entry)
        } else {
            None
        }
    }
}

impl DirList3 {
    /// Iterate over directory entries
    pub fn iter(&self) -> Entry3Iter<'_> {
        Entry3Iter {
            next: self.entries.as_ref(),
        }
    }
}
