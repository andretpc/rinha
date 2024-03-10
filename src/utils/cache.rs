use super::Mmap;
use speedy::{LittleEndian, Readable, Writable};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct Cache {
    inner: RwLock<HashMap<String, Mmap>>,
}

impl Cache {
    pub fn new() -> Self {
        let inner = RwLock::new(HashMap::new());

        let paths = std::fs::read_dir("/dev/shm").unwrap();

        for path in paths {
            let entry = path.unwrap();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.starts_with("dk-rinha") || file_name_str.starts_with("sem.dk-rinha") {
                let file_path = entry.path();

                let _ = std::fs::remove_file(&file_path);
            }
        }

        Self { inner }
    }

    pub async fn get<'a, T>(&self, key: &str) -> Option<T>
    where
        T: Readable<'a, LittleEndian>,
    {
        let value = self.read(key).await;

        return T::read_from_buffer(value).ok();
    }

    pub async fn insert<'a, T>(&self, key: &str, value: &T) -> Option<T>
    where
        T: Writable<LittleEndian> + Readable<'a, LittleEndian>,
    {
        let value_bytes = value.write_to_vec().unwrap();
        let previous = self.write(key, &value_bytes).await;

        T::read_from_buffer(previous).ok()
    }

    async fn init<'a>(&self, key: &str, bytes: Option<&[u8]>) -> &'a [u8] {
        let mmap = Mmap::new(&format!("cache-{key}"));

        if let Some(bytes) = bytes {
            mmap.write(bytes);
        }

        let mut guard = self.inner.write().await;

        guard.insert(key.to_string(), mmap);

        guard.get(key).unwrap().read()
    }

    async fn read<'a>(&self, key: &str) -> &'a [u8] {
        if let Some(mmap) = self.inner.read().await.get(key) {
            return mmap.read();
        }

        return self.init(key, None).await;
    }

    async fn write<'a>(&self, key: &str, bytes: &[u8]) -> &'a [u8] {
        if let Some(mmap) = self.inner.read().await.get(key) {
            let prev = mmap.read();

            mmap.write(bytes);

            return prev;
        }

        return self.init(key, Some(bytes)).await;
    }
}
