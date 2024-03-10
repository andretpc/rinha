use nix::libc::{
    c_void, close, ftruncate, memcpy, mmap, mremap, off_t, shm_open, shm_unlink, size_t,
    MAP_SHARED, MREMAP_MAYMOVE, O_CREAT, O_RDWR, PROT_READ, PROT_WRITE,
};
use std::{
    ffi::CString,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

pub struct Mmap {
    name: CString,
    address: AtomicPtr<c_void>,
    length_address: AtomicPtr<c_void>,
}

impl Mmap {
    const INIT_LENGTH: u32 = 4;

    pub fn new(name: &str) -> Self {
        let (length, length_address) = Self::init_length(name);

        let name = Self::mmap_name(name);
        let fd = Self::open_shared_memory(&name, length);
        let address = Self::map_to_memory(fd, length);

        let atomic_ptr = AtomicPtr::new(std::ptr::null_mut());

        atomic_ptr.store(address, Ordering::SeqCst);

        Self {
            name,
            address: atomic_ptr,
            length_address,
        }
    }

    fn mmap_name(name: &str) -> CString {
        CString::new(format!("/dk-rinha-2024-mmap-{name}")).unwrap()
    }

    fn mmap_length_name(name: &str) -> CString {
        CString::new(format!("/dk-rinha-2024-mmap-len-{name}")).unwrap()
    }

    fn init_length(name: &str) -> (u32, AtomicPtr<c_void>) {
        let mmap_length_name = Self::mmap_length_name(name);
        let fd = Self::open_shared_memory(&mmap_length_name, 4);
        let length_address = Self::map_to_memory(fd, 4);

        let bytes = unsafe { std::slice::from_raw_parts(length_address as *const u8, 4) };

        let length: u32 = match bytes.iter().all(|&b| b == 0) {
            false => u32::from_le_bytes(bytes.try_into().unwrap()),
            true => {
                let bytes = u32::to_le_bytes(Self::INIT_LENGTH);

                unsafe {
                    memcpy(length_address, bytes.as_ptr() as *const c_void, 4);
                };

                Self::INIT_LENGTH
            }
        };

        let length_atomic_ptr = AtomicPtr::new(std::ptr::null_mut());

        length_atomic_ptr.store(length_address, Ordering::SeqCst);

        (length, length_atomic_ptr)
    }

    fn open_shared_memory(name: &CString, length: u32) -> i32 {
        let shm_fd = unsafe { shm_open(name.as_ptr(), O_RDWR | O_CREAT, 0o666) };

        if shm_fd < 0 {
            panic!(
                "failed to open shared memory with code: {}",
                std::io::Error::last_os_error().raw_os_error().unwrap()
            )
        }

        unsafe {
            ftruncate(shm_fd, length as off_t);
        }

        shm_fd
    }

    fn map_to_memory(shm_fd: i32, len: u32) -> *mut c_void {
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
        self.set_length(bytes.len().try_into().unwrap());

        unsafe {
            memcpy(
                self.address.load(Ordering::SeqCst),
                bytes.as_ptr() as *const c_void,
                self.get_length() as size_t,
            );
        };
    }

    pub fn read<'a>(&self) -> &'a [u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.address.load(Ordering::SeqCst) as *const u8,
                self.get_length() as size_t,
            )
        }
    }

    pub fn set_length<'a>(&self, len: u32) {
        let bytes = u32::to_le_bytes(len);

        unsafe {
            memcpy(
                self.length_address.load(Ordering::SeqCst),
                bytes.as_ptr() as *const c_void,
                4,
            );

            let addr = mremap(
                self.address.load(Ordering::SeqCst),
                self.get_length() as size_t,
                len as size_t,
                MREMAP_MAYMOVE,
            );

            if addr != self.address.load(Ordering::SeqCst) {
                self.address.store(addr, Ordering::SeqCst)
            }
        };
    }

    pub fn get_length(&self) -> u32 {
        let bytes = unsafe {
            std::slice::from_raw_parts(self.length_address.load(Ordering::SeqCst) as *const u8, 4)
        };

        u32::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        unsafe { shm_unlink(self.name.as_ptr()) };
    }
}
