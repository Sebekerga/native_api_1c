//!
//! FFI bindings for 1C:Enterprise Native API are divided between several
//! submodules according to what C++ class they originate from
//!
use std::{
    ffi::{c_long, c_void},
    ptr,
};

use crate::interface::AddInWrapper;

use self::{
    connection::Connection, init_done::InitDoneBaseVTable,
    lang_extender::LanguageExtenderBaseVTable, locale_base::LocaleBaseVTable,
    memory_manager::MemoryManager, string_utils::get_str,
    user_lang_base::UserLanguageBaseVTable,
};

/// Implementation of `Connection` - replacement for `IAddInDefBase`
pub mod connection;
/// Implementation of `InitDone` - replacement for `IInitDoneBase`
pub mod init_done;
/// Implementation of `LanguageExtender` - replacement for `ILanguageExtenderBase`
pub mod lang_extender;
/// Implementation of `LocaleBase`
pub mod locale_base;
/// Implementation of `MemoryManager` - replacement for `IMemoryManager`
pub mod memory_manager;
/// Implementations of types, provided by Native API for easy of use in Rust
pub mod provided_types;
/// Functions to convert between Rust and 1C strings
pub mod string_utils;
/// Implementation of `UserLanguageBase`
pub mod user_lang_base;

/// Scheme of attaching to 1C platform process
#[repr(C)]
#[derive(Debug)]
#[allow(dead_code)]
pub enum AttachType {
    /// Attach to 1C platform process
    NotIsolated = 1,
    /// Attach to separate process
    Isolated,
    /// Any of the above
    Any,
}

/// Struct to extract pointer to `Component` from it's interface components
/// In some places we need to get pointer to `Component` from it's interface
/// components, so we need to calculate offset of `Component` in memory
#[repr(C)]
struct This<const OFFSET: usize, T: AddInWrapper> {
    ptr: *mut Component<T>,
}

mod offset {
    pub const INIT_DONE: usize = 0;
    pub const LANG_EXTENDER: usize = 1;
    pub const LOCALE: usize = 2;
    pub const USER_LANG: usize = 3;
}

impl<'a, const OFFSET: usize, T: AddInWrapper> This<OFFSET, T> {
    unsafe fn get_component(&mut self) -> &'a mut Component<T> {
        let new_ptr = (self as *mut This<OFFSET, T> as *mut c_void)
            .sub(OFFSET * std::mem::size_of::<usize>());
        &mut *(new_ptr as *mut Component<T>)
    }
}

#[repr(C)]
struct Component<T: AddInWrapper> {
    // 1C Interface
    init_done_ptr: Box<InitDoneBaseVTable<T>>,
    lang_extender_ptr: Box<LanguageExtenderBaseVTable<T>>,
    locale_ptr: Box<LocaleBaseVTable<T>>,
    usr_lang_ptr: Box<UserLanguageBaseVTable<T>>,

    // storage for additional interfaces
    memory_manager_ptr: Option<&'static MemoryManager>,
    connection_ptr: Option<&'static Connection>,
    locale: Option<String>,
    user_interface_language_code: Option<String>,

    // rust part
    destroy: unsafe extern "system" fn(*mut *mut Component<T>),
    addin: T,
}

unsafe extern "system" fn destroy<T: AddInWrapper>(
    component: *mut *mut Component<T>,
) {
    let comp = Box::from_raw(*component);
    drop(comp);
}

/// # Safety
/// `component` must be a valid pointer to a `Component` from GetClassObject call
/// `addin` must be a valid `AddInWrapper` instance
pub unsafe fn create_component<T: AddInWrapper>(
    component: *mut *mut c_void,
    addin: T,
) -> c_long {
    let c = Box::new(Component {
        init_done_ptr: Default::default(),
        lang_extender_ptr: Default::default(),
        locale_ptr: Default::default(),
        usr_lang_ptr: Default::default(),

        destroy: destroy::<T>,
        memory_manager_ptr: Default::default(),
        connection_ptr: Default::default(),
        locale: Default::default(),
        user_interface_language_code: Default::default(),
        addin,
    });

    *component = Box::into_raw(c) as *mut c_void;
    1
}

/// # Safety
/// `component` must be a valid pointer to a `Component` from DestroyObject call
pub unsafe fn destroy_component(component: *mut *mut c_void) -> c_long {
    #[repr(C)]
    struct ComponentWrapper {
        vptr1: usize,
        vptr2: usize,
        vptr3: usize,
        vptr4: usize,
        destroy: unsafe extern "system" fn(*mut *mut c_void),
    }

    let wrapper = *component as *mut ComponentWrapper;
    let wrapper = &mut *wrapper;
    (wrapper.destroy)(component);
    *component = ptr::null_mut();

    0
}
