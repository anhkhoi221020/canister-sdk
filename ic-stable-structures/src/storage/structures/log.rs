use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;

use ic_exports::ic_kit::ic;
use ic_exports::stable_structures::memory_manager::MemoryId;
use ic_exports::stable_structures::{log, Storable};
use ic_exports::Principal;

use crate::{Error, Memory, Result};

/// Stores list of immutable values in stable memory.
/// Provides only `append()` and `get()` operations.
pub struct StableLog<T: Storable> {
    data: HashMap<Principal, log::Log<Memory, Memory>>,
    index_memory_id: MemoryId,
    data_memory_id: MemoryId,
    _data: PhantomData<T>,
}

impl<T: Storable> StableLog<T> {
    /// Create new storage for values with `T` type.
    pub fn new(index_memory_id: MemoryId, data_memory_id: MemoryId) -> Result<Self> {
        // Method returns Result to be compatible with wasm implementation.

        // Index and data should be stored in different memories.
        assert_ne!(index_memory_id, data_memory_id);

        Ok(Self {
            data: HashMap::default(),
            index_memory_id,
            data_memory_id,
            _data: PhantomData,
        })
    }

    /// Returns reference to value stored in stable memory.
    pub fn get(&self, index: usize) -> Option<T> {
        self.get_inner()?.get(index).map(T::from_bytes)
    }

    /// Updates value in stable memory.
    pub fn append(&mut self, value: T) -> Result<usize> {
        let canister_id = ic::id();
        let index = match self.data.entry(canister_id) {
            Entry::Occupied(mut entry) => entry
                .get_mut()
                .append(value.to_bytes().deref())
                .map_err(|_| Error::OutOfStableMemory)?,
            Entry::Vacant(entry) => {
                let index_memory = crate::get_memory_by_id(self.index_memory_id);
                let data_memory = crate::get_memory_by_id(self.data_memory_id);
                let inserted = entry.insert(log::Log::init(index_memory, data_memory)?);
                inserted
                    .append(value.to_bytes().deref())
                    .map_err(|_| Error::OutOfStableMemory)?
            }
        };
        Ok(index)
    }

    /// Number of values in the log.
    pub fn len(&self) -> usize {
        self.get_inner()
            .map(|inner| inner.len())
            .unwrap_or_default()
    }

    /// Return true, if the Log doesn't contain any value.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get_inner(&self) -> Option<&log::Log<Memory, Memory>> {
        let canister_id = ic::id();
        self.data.get(&canister_id)
    }
}

#[cfg(test)]
mod tests {
    use ic_exports::{
        ic_kit::{inject::get_context, mock_principals, MockContext},
        stable_structures::memory_manager::MemoryId,
    };

    use super::StableLog;

    #[test]
    fn log_works() {
        MockContext::new().inject();
        let mut log = StableLog::new(MemoryId::new(0), MemoryId::new(1)).unwrap();
        assert!(log.is_empty());

        log.append(10u32).unwrap();
        log.append(20u32).unwrap();
        assert_eq!(log.len(), 2);

        assert_eq!(log.get(0).unwrap(), 10);
        assert_eq!(log.get(1).unwrap(), 20);
    }

    #[test]
    fn two_canisters() {
        MockContext::new()
            .with_id(mock_principals::alice())
            .inject();

        let mut log = StableLog::new(MemoryId::new(0), MemoryId::new(1)).unwrap();
        log.append(10u32).unwrap();

        get_context().update_id(mock_principals::bob());
        log.append(20u32).unwrap();
        log.append(30u32).unwrap();

        get_context().update_id(mock_principals::alice());
        assert_eq!(log.len(), 1);

        get_context().update_id(mock_principals::bob());
        assert_eq!(log.len(), 2);
    }
}
