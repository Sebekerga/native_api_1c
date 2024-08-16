use std::{
    ffi::{c_ulong, c_void},
    ptr::{self, NonNull},
};

/// VTable for MemoryManager object, derived from Native API interface. See original
/// C++ implementation in [example project](https://its.1c.ru/db/files/1CITS/EXE/VNCOMPS/VNCOMPS.zip)
/// from 1C documentation
#[repr(C)]
struct MemoryManagerVTable {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    alloc_memory: unsafe extern "system" fn(
        &MemoryManager,
        *mut *mut c_void,
        c_ulong,
    ) -> bool,
    free_memory: unsafe extern "system" fn(&MemoryManager, *mut *mut c_void),
}

/// MemoryManager object, used to allocate memory for the AddIn
#[repr(C)]
pub struct MemoryManager {
    vptr: &'static MemoryManagerVTable,
}

pub struct AllocationError;

impl MemoryManager {
    /// Safe wrapper around `alloc_memory` method of the MemoryManager object
    /// to allocate memory for byte array
    /// # Arguments
    /// * `size` - size of the memory block to allocate
    /// # Returns
    /// `Result<NonNull<u8>, AllocationError>` - pointer to the allocated memory block
    pub fn alloc_blob(
        &self,
        size: usize,
    ) -> Result<NonNull<u8>, AllocationError> {
        let mut ptr = ptr::null_mut::<c_void>();
        unsafe {
            if (self.vptr.alloc_memory)(self, &mut ptr, size as c_ulong * 2) {
                match NonNull::new(ptr as *mut u8) {
                    Some(ptr) => Ok(ptr),
                    None => Err(AllocationError),
                }
            } else {
                Err(AllocationError)
            }
        }
    }

    /// Safe wrapper around `alloc_memory` method of the MemoryManager object
    /// to allocate memory for UTF-16 string
    /// # Arguments
    /// * `size` - size of the memory block to allocate
    /// # Returns
    /// `Result<NonNull<u16>, AllocationError>` - pointer to the allocated memory block
    pub fn alloc_str(
        &self,
        size: usize,
    ) -> Result<NonNull<u16>, AllocationError> {
        let mut ptr = ptr::null_mut::<c_void>();
        unsafe {
            if (self.vptr.alloc_memory)(self, &mut ptr, size as c_ulong * 2) {
                match NonNull::new(ptr as *mut u16) {
                    Some(ptr) => Ok(ptr),
                    None => Err(AllocationError),
                }
            } else {
                Err(AllocationError)
            }
        }
    }

    pub fn free_memory(&self, ptr: &mut *mut c_void) {
        unsafe {
            (self.vptr.free_memory)(self, ptr);
        }
    }
}
