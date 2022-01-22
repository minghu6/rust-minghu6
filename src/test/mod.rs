pub mod spm;
pub mod sort;
pub mod dict;
pub mod heap;
pub mod persistent;
pub mod utils;


use rand::random;


////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait Provider<T> {
    fn get_one(&self) -> T;

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a> {
        box std::iter::from_fn(move || Some(self.get_one()))
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Structures


#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Inode {
    pub mode: u16,
    pub num_links: u16,
    pub uid: u16,
    pub gid: u16,
    pub size: u32,
    pub atime: u32, // access time
    pub mtime: u32, // modified time
    pub ctime: u32, // create time
    pub zones: [u32; 10],
}


pub struct InodeProvider {}


pub struct UZProvider {}


////////////////////////////////////////////////////////////////////////////////
//// Implements


impl Provider<Inode> for InodeProvider {
    fn get_one(&self) -> Inode {
        Inode {
            mode: now_secs() as u16,
            num_links: now_secs() as u16,
            uid: now_secs() as u16,
            gid: now_secs() as u16,
            size: now_secs() as u32,
            atime: now_secs() as u32,
            mtime: now_secs() as u32,
            ctime: now_secs() as u32,
            zones: [now_secs() as u32; 10],
        }
    }
}


impl Provider<usize> for UZProvider {
    fn get_one(&self) -> usize {
        random::<usize>() % 1000
    }
}


#[inline]
fn now_secs() -> u64 {
    random::<u32>() as u64
}
