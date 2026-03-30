use alloc::string::String as HeapString;
use alloc::vec::Vec as AllocVec;

use compact_str::CompactString;
use hashbrown::{HashMap, HashSet};
use rustc_hash::FxBuildHasher;
use smallvec::SmallVec;

pub(crate) type AstString = CompactString;
pub(crate) type AstVec<T> = SmallVec<[T; 4]>;
pub(crate) type HeapVec<T> = AllocVec<T>;
pub(crate) type Map<K, V> = HashMap<K, V, FxBuildHasher>;
pub(crate) type Set<T> = HashSet<T, FxBuildHasher>;
pub(crate) type OutputString = HeapString;
