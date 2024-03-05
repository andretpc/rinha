use super::{semaphore::Semaphore, Mmap};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct Cache {
    inner: RwLock<HashMap<i32, Mmap>>,
    pub semaphore: Semaphore,
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

        Self {
            inner,
            semaphore: Semaphore::new(),
        }
    }

    pub async fn init<'a>(&self, key: i32, bytes: &[u8]) -> &'a [u8] {
        let mmap = Mmap::new(&format!("cache-{key}"));

        if mmap.read().iter().all(|&b| b == 0) {
            mmap.write(bytes);
        }

        let mut guard = self.inner.write().await;

        guard.insert(key, mmap);

        guard.get(&key).unwrap().read()
    }

    pub async fn get<'a>(&self, key: i32) -> Option<&'a [u8]> {
        self.inner
            .read()
            .await
            .get(&key)
            .and_then(|mmap| Some(mmap.read()))
    }

    pub async fn lock(&self, key: i32) {
        self.semaphore.wait(key).await;
    }

    pub async fn unlock(&self, key: i32) {
        self.semaphore.release(key).await;
    }

    pub async fn write<'a>(&self, key: i32, bytes: &'a [u8]) {
        if let Some(mmap) = self.inner.write().await.get(&key) {
            mmap.write(bytes);
        };
    }
}
