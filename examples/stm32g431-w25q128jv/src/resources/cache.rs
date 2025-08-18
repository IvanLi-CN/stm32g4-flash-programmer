use heapless::Vec;

/// Simple LRU cache for Flash data
pub struct FlashCache<const N: usize> {
    entries: Vec<CacheEntry, N>,
}

/// Cache entry
#[derive(Clone)]
struct CacheEntry {
    address: u32,
    data: Vec<u8, 1024>, // Max 1KB per entry
    access_count: u32,
}

impl<const N: usize> FlashCache<N> {
    /// Create new cache
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Get data from cache
    pub fn get(&mut self, address: u32, length: usize) -> Option<&[u8]> {
        for entry in &mut self.entries {
            if entry.address == address && entry.data.len() >= length {
                entry.access_count += 1;
                return Some(&entry.data[..length]);
            }
        }
        None
    }

    /// Put data into cache
    pub fn put(&mut self, address: u32, data: &[u8]) -> Result<(), &'static str> {
        // Check if entry already exists
        for entry in &mut self.entries {
            if entry.address == address {
                entry.data.clear();
                for &byte in data {
                    entry.data.push(byte).map_err(|_| "Data too large for cache entry")?;
                }
                entry.access_count += 1;
                return Ok(());
            }
        }

        // Add new entry
        if self.entries.len() < N {
            let mut new_data = Vec::new();
            for &byte in data {
                new_data.push(byte).map_err(|_| "Data too large for cache entry")?;
            }

            let entry = CacheEntry {
                address,
                data: new_data,
                access_count: 1,
            };

            self.entries.push(entry).map_err(|_| "Cache full")?;
        } else {
            // Replace least recently used entry
            let lru_index = self.find_lru_index();
            let entry = &mut self.entries[lru_index];

            entry.address = address;
            entry.data.clear();
            for &byte in data {
                entry.data.push(byte).map_err(|_| "Data too large for cache entry")?;
            }
            entry.access_count = 1;
        }

        Ok(())
    }

    /// Find least recently used entry index
    fn find_lru_index(&self) -> usize {
        let mut lru_index = 0;
        let mut min_access = u32::MAX;

        for (i, entry) in self.entries.iter().enumerate() {
            if entry.access_count < min_access {
                min_access = entry.access_count;
                lru_index = i;
            }
        }

        lru_index
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut total_access = 0;
        let mut total_size = 0;

        for entry in &self.entries {
            total_access += entry.access_count;
            total_size += entry.data.len();
        }

        CacheStats {
            entries: self.entries.len(),
            max_entries: N,
            total_access_count: total_access,
            total_size_bytes: total_size,
        }
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub entries: usize,
    pub max_entries: usize,
    pub total_access_count: u32,
    pub total_size_bytes: usize,
}
