use nix::libc::{sem_open, sem_post, sem_t, sem_wait, O_CREAT, O_RDWR, SEM_FAILED};
use std::{
    collections::HashMap,
    ffi::CString,
    sync::atomic::{AtomicPtr, Ordering},
};
use tokio::sync::RwLock;

pub struct Semaphore {
    sems: RwLock<HashMap<i32, AtomicPtr<sem_t>>>,
}

impl Semaphore {
    pub fn new() -> Self {
        let sems = RwLock::new(HashMap::new());

        Self { sems }
    }

    pub async fn release(&self, key: i32) {
        if let Some(sem) = self.sems.read().await.get(&key) {
            unsafe { sem_post(sem.load(Ordering::SeqCst)) };
        }
    }

    pub async fn wait(&self, key: i32) {
        if let Some(sem) = self.sems.read().await.get(&key) {
            unsafe { sem_wait(sem.load(Ordering::SeqCst)) };

            return;
        }

        unsafe {
            let sem = Self::init(&key);

            sem_wait(sem.load(Ordering::SeqCst));

            self.sems.write().await.insert(key, sem);
        };
    }

    fn init(key: &i32) -> AtomicPtr<sem_t> {
        let name = CString::new(format!("/dk-rinha-2024-sem-{key}")).unwrap();
        let semaphore = Self::open_semaphore(&name);
        let atomic_ptr = AtomicPtr::new(std::ptr::null_mut());

        atomic_ptr.store(semaphore, Ordering::SeqCst);

        atomic_ptr
    }

    pub fn open_semaphore(name: &CString) -> *mut sem_t {
        unsafe {
            let sem_fd = sem_open(name.as_ptr(), O_RDWR | O_CREAT, 0o666, 1);

            if sem_fd == SEM_FAILED {
                panic!("failed to open sem")
            }

            sem_fd
        }
    }
}
