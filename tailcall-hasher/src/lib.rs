use std::collections::HashMap;
use std::hash::Hasher;

use fnv::FnvHasher;

/// A hasher that uses the FxHash algorithm. Currently it's a dumb wrapper
/// around `fxhash::FxHasher`. We could potentially add some custom logic here
/// in the future.
#[derive(Default)]
pub struct TailcallHasher {
    hasher: FnvHasher,
}

impl Hasher for TailcallHasher {
    fn finish(&self) -> u64 {
        self.hasher.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes)
    }
}

#[derive(Clone, Default)]
pub struct TailcallBuildHasher;

impl std::hash::BuildHasher for TailcallBuildHasher {
    type Hasher = TailcallHasher;

    fn build_hasher(&self) -> Self::Hasher {
        TailcallHasher::default()
    }
}

pub type TailcallHashMap<K, V> = HashMap<K, V, TailcallBuildHasher>;