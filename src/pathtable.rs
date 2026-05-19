//! A global table for the pathnames used in `FileEntry` and `Loc`.
//!
//! Using this will make the often-cloned Loc faster to copy, since it will just contain an index into the global table.
//! It also makes it faster to compare pathnames, because the table will be created in lexical order by the caller
//! ([`Fileset`](crate::fileset::Fileset)), with the exception of some stray files (such as the config file)
//! where the order doesn't matter.
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, RwLock};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathTableIndex(u32);

static PATHTABLE: LazyLock<RwLock<PathTable>> = LazyLock::new(|| RwLock::new(PathTable::default()));

/// A global table for the pathnames used in `FileEntry` and `Loc`.
///
/// See the [`self`](module-level documentation) for details.
pub struct PathTable {
    /// Heap-allocated paths stored as raw pointers so they can be explicitly freed during
    /// LSP resets. Each pointer was created via `Box::into_raw(PathBuf::into_boxed_path())`.
    /// Invariant: each raw pointer remains valid until `clear_for_lsp_run` is called.
    paths: Vec<(*const Path, *const Path)>,
}

impl Default for PathTable {
    fn default() -> Self {
        PathTable { paths: Vec::new() }
    }
}

impl std::fmt::Debug for PathTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathTable").field("len", &self.paths.len()).finish()
    }
}

// SAFETY: PathTable uniquely owns the heap-allocated Box<Path> values behind its raw pointers.
unsafe impl Send for PathTable {}
unsafe impl Sync for PathTable {}

impl PathTable {
    /// Stores a path in the path table and returns the index for the entry.
    /// It's assumed that the caller has a master list of paths and won't store duplicates.
    ///
    /// The indexes are guaranteed to be in ascending order, so that if the caller stores a sorted
    /// list of paths then the indexes will also be sorted.
    pub fn store(local: PathBuf, fullpath: PathBuf) -> PathTableIndex {
        PATHTABLE.write().unwrap().store_internal(local, fullpath)
    }

    fn store_internal(&mut self, local: PathBuf, fullpath: PathBuf) -> PathTableIndex {
        let idx = PathTableIndex(u32::try_from(self.paths.len()).expect("internal error"));
        let local = Box::into_raw(local.into_boxed_path()) as *const Path;
        let fullpath = Box::into_raw(fullpath.into_boxed_path()) as *const Path;
        self.paths.push((local, fullpath));
        idx
    }

    /// Return the local path based on its index.
    /// This can panic if the index is not one provided by `PathTable::store`.
    pub fn lookup_path(idx: PathTableIndex) -> &'static Path {
        let guard = PATHTABLE.read().unwrap();
        let PathTableIndex(i) = idx;
        // SAFETY: pointer is valid until clear_for_lsp_run() is called; callers must not
        // hold &'static Path references across a reset.
        unsafe { &*guard.paths[i as usize].0 }
    }

    /// Return the full path based on its index.
    /// This can panic if the index is not one provided by `PathTable::store`.
    pub fn lookup_fullpath(idx: PathTableIndex) -> &'static Path {
        let guard = PATHTABLE.read().unwrap();
        let PathTableIndex(i) = idx;
        unsafe { &*guard.paths[i as usize].1 }
    }

    /// Free all path allocations and clear the table.
    ///
    /// # Safety
    /// Must only be called when no `&'static Path` references or `PathTableIndex` values
    /// from this table are live. In LSP mode, call after clearing `ERRORS` and `MACRO_MAP`
    /// and before starting a new validation run.
    pub unsafe fn clear_for_lsp_run() {
        let mut guard = PATHTABLE.write().unwrap();
        for (local, fullpath) in guard.paths.drain(..) {
            // SAFETY: Pointers were created by `Box::into_raw` in `store_internal`.
            // Caller guarantees no live `&'static Path` refs exist at this point.
            unsafe {
                drop(Box::from_raw(local as *mut Path));
                drop(Box::from_raw(fullpath as *mut Path));
            }
        }
    }
}
