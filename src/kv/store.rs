use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use super::log as kv_log;

#[derive(Debug)]
pub struct KV {
    index: HashMap<Vec<u8>, Vec<u8>>,
    log: File,
}

impl KV {
    pub fn open(path: &Path) -> io::Result<Self> {
        let log = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;

        let mut index = HashMap::new();
        let good_pos = kv_log::replay(&log, &mut index)?;
        log.set_len(good_pos)?;

        Ok(KV { index, log })
    }

    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        self.index.get(key.as_ref()).cloned()
    }

    pub fn set(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> io::Result<()> {
        let (key, value) = (key.as_ref(), value.as_ref());
        self.log.write_all(&kv_log::encode(key, Some(value)))?;
        self.log.sync_all()?;
        self.index.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    pub fn delete(&mut self, key: impl AsRef<[u8]>) -> io::Result<()> {
        let key = key.as_ref();
        self.log.write_all(&kv_log::encode(key, None))?;
        self.log.sync_all()?;
        self.index.remove(key);
        Ok(())
    }
}
