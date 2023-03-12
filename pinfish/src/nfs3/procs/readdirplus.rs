use crate::{
    nfs3::{Count3, NfsFh3, Cookie3, PostOpAttributes, Verifier3, FileId3, Filename3, PostOpFh3},
    xdr::{self},
};
use pinfish_macros::{PackTo, UnpackFrom};

#[derive(PackTo, Debug)]
pub struct ReaddirPlus3Args {
    pub dir: NfsFh3,
    pub cookie: Cookie3,
    pub verifier: Verifier3,
    /// Number of READDIR bytes the client really wants
    pub dircount: Count3,
    /// Maximum size of response, including attributes
    pub maxcount: Count3,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct EntryPlus3 {
    pub fileid: FileId3,
    pub name: Filename3,
    pub cookie: Cookie3,
    name_attibutes: PostOpAttributes,
    name_handle: PostOpFh3,
    pub next_entry: Option<Box<EntryPlus3>>,
}

#[derive(UnpackFrom, PackTo, Debug)]
pub struct DirListPlus3 {
    pub entries: Option<EntryPlus3>,
    pub eof: bool,
}


#[derive(PackTo, UnpackFrom, Debug)]
pub struct ReaddirPlus3ResOk {
    pub dir_attributes: PostOpAttributes,
    pub verifier: Verifier3,
    pub reply: DirListPlus3,
}

#[derive(PackTo, UnpackFrom, Debug)]
pub struct ReaddirPlus3ResFail {
    pub dir_attributes: PostOpAttributes,
}

pub type ReaddirPlusResult = Result<ReaddirPlus3ResOk, (u32, ReaddirPlus3ResFail)>;

/// Iterator for directory entries
pub struct EntryPlus3Iter<'a> {
    next: Option<&'a EntryPlus3>,
}

impl<'a> Iterator for EntryPlus3Iter<'a> {
    type Item = &'a EntryPlus3;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.next {
            self.next = entry.next_entry.as_ref().map(|e| &**e);
            Some(entry)
        } else {
            None
        }
    }
}

impl DirListPlus3 {
    /// Iterate over directory entries
    pub fn iter(&self) -> EntryPlus3Iter<'_> {
        EntryPlus3Iter {
            next: self.entries.as_ref(),
        }
    }
}
