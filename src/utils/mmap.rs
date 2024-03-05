use nix::libc::{
    c_void, close, ftruncate, memcpy, mmap, mremap, off_t, shm_open, shm_unlink, size_t, unlink,
    MAP_SHARED, MREMAP_MAYMOVE, O_CREAT, O_RDWR, PROT_READ, PROT_WRITE,
};
use std::{
    ffi::CString,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

pub struct Mmap {
    name: CString,
    addr: AtomicPtr<c_void>,
    len_addr: AtomicPtr<c_void>,
}

impl Mmap {
    const INIT_LEN: u32 = 4;

    pub fn new(name: &str) -> Self {
        let (len, len_addr) = Self::init_len(name);

        let shm_name = CString::new(format!("/dk-rinha-2024-mmap-{name}")).unwrap();
        let shm_fd = Self::open_shared_memory(&shm_name, len);
        let addr = Self::map(shm_fd, len);

        let atomic_ptr = AtomicPtr::new(std::ptr::null_mut());

        atomic_ptr.store(addr, Ordering::SeqCst);

        Self {
            name: shm_name,
            len_addr,
            addr: atomic_ptr,
        }
    }

    fn init_len(name: &str) -> (u32, AtomicPtr<c_void>) {
        let shm_len_name = CString::new(format!("/dk-rinha-2024-mmap-len-{name}")).unwrap();
        let shm_len_fd = Self::open_shared_memory(&shm_len_name, 4);
        let len_addr = Self::map(shm_len_fd, 4);

        let bytes = unsafe { std::slice::from_raw_parts(len_addr as *const u8, 4) };

        let len: u32 = match bytes.iter().all(|&b| b == 0) {
            false => u32::from_le_bytes(bytes.try_into().unwrap()),
            true => {
                let bytes = u32::to_le_bytes(Self::INIT_LEN);

                unsafe {
                    memcpy(len_addr, bytes.as_ptr() as *const c_void, 4);
                };

                Self::INIT_LEN
            }
        };

        let len_atomic_ptr = AtomicPtr::new(std::ptr::null_mut());

        len_atomic_ptr.store(len_addr, Ordering::SeqCst);

        (len, len_atomic_ptr)
    }

    fn open_shared_memory(name: &CString, len: u32) -> i32 {
        let shm_fd = unsafe { shm_open(name.as_ptr(), O_RDWR | O_CREAT, 0o666) };

        if shm_fd < 0 {
            panic!(
                "failed to open shared memory with code: {}",
                std::io::Error::last_os_error().raw_os_error().unwrap()
            )
        }

        unsafe {
            ftruncate(shm_fd, len as off_t);
        }

        shm_fd
    }

    fn map(shm_fd: i32, len: u32) -> *mut c_void {
        unsafe {
            let addr = mmap(
                ptr::null_mut(),
                len as size_t,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                shm_fd,
                0,
            );

            close(shm_fd);

            addr
        }
    }

    pub fn write(&self, bytes: &[u8]) {
        self.set_len(bytes.len().try_into().unwrap());

        unsafe {
            memcpy(
                self.addr.load(Ordering::SeqCst),
                bytes.as_ptr() as *const c_void,
                self.get_len() as size_t,
            );
        };
    }

    pub fn read<'a>(&self) -> &'a [u8] {
        let content = unsafe {
            std::slice::from_raw_parts(
                self.addr.load(Ordering::SeqCst) as *const u8,
                self.get_len() as size_t,
            )
        };

        content
    }

    pub fn set_len<'a>(&self, len: u32) {
        let bytes = u32::to_le_bytes(len);

        unsafe {
            memcpy(
                self.len_addr.load(Ordering::SeqCst),
                bytes.as_ptr() as *const c_void,
                4,
            );

            let addr = mremap(
                self.addr.load(Ordering::SeqCst),
                self.get_len() as size_t,
                len as size_t,
                MREMAP_MAYMOVE,
            );

            if addr != self.addr.load(Ordering::SeqCst) {
                self.addr.store(addr, Ordering::SeqCst)
            }
        };
    }

    pub fn get_len(&self) -> u32 {
        let bytes = unsafe {
            std::slice::from_raw_parts(self.len_addr.load(Ordering::SeqCst) as *const u8, 4)
        };

        u32::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        unsafe { shm_unlink(self.name.as_ptr()) };
    }
}
