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
    init_base::InitDoneBaseVTable, lang_extender::LanguageExtenderBaseVTable,
    memory_manager::MemoryManager, string_utils::get_str,
};

/// Implementation of `Connection` - replacement for `IAddInDefBase`
pub mod connection;
/// Implementation of `InitDone` - replacement for `IInitDoneBase`
pub mod init_base;
/// Implementation of `LanguageExtender` - replacement for `ILanguageExtenderBase`
pub mod lang_extender;
/// Implementation of `MemoryManager` - replacement for `IMemoryManager`
pub mod memory_manager;
/// Implementations of types, provided by Native API for easy of use in Rust
pub mod provided_types;
/// Functions to convert between Rust and 1C strings
pub mod string_utils;

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

#[repr(C)]
struct This<const OFFSET: usize, T: AddInWrapper> {
    ptr: *mut Component<T>,
}

impl<'a, const OFFSET: usize, T: AddInWrapper> This<OFFSET, T> {
    unsafe fn get_component(&mut self) -> &'a mut Component<T> {
        let new_ptr = (self as *mut This<OFFSET, T> as *mut c_void)
            .sub(OFFSET * std::mem::size_of::<usize>());
        &mut *(new_ptr as *mut Component<T>)
    }
}

#[repr(C)]
struct LocaleBaseVTable<T: AddInWrapper> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    set_locale: unsafe extern "system" fn(&mut This<2, T>, *const u16),
}

unsafe extern "system" fn set_locale<T: AddInWrapper>(
    this: &mut This<2, T>,
    loc: *const u16,
) {
    let component = this.get_component();
    let loc = get_str(loc);
    component.addin.set_locale(loc)
}

#[repr(C)]
struct UserLanguageBaseVTable<T: AddInWrapper> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    set_user_interface_language_code:
        unsafe extern "system" fn(&mut This<3, T>, *const u16),
}

unsafe extern "system" fn set_user_interface_language_code<T: AddInWrapper>(
    this: &mut This<3, T>,
    lang: *const u16,
) {
    let component = this.get_component();
    let lang = get_str(lang);
    component.addin.set_user_interface_language_code(lang)
}

#[repr(C)]
struct Component<T: AddInWrapper> {
    vptr1: Box<InitDoneBaseVTable<T>>,
    vptr2: Box<LanguageExtenderBaseVTable<T>>,
    vptr3: Box<LocaleBaseVTable<T>>,
    vptr4: Box<UserLanguageBaseVTable<T>>,
    destroy: unsafe extern "system" fn(*mut *mut Component<T>),
    memory: Option<&'static MemoryManager>,
    addin: T,
}

unsafe extern "system" fn destroy<T: AddInWrapper>(
    component: *mut *mut Component<T>,
) {
    let comp = Box::from_raw(*component);
    drop(comp);
}

pub unsafe fn create_component<T: AddInWrapper>(
    component: *mut *mut c_void,
    addin: T,
) -> c_long {
    let vptr1 = Box::<InitDoneBaseVTable<T>>::default();
    let vptr2 = Box::<LanguageExtenderBaseVTable<T>>::default();
    let vptr3 = Box::new(LocaleBaseVTable {
        dtor: 0,
        #[cfg(target_family = "unix")]
        dtor2: 0,
        set_locale,
    });

    let vptr4 = Box::new(UserLanguageBaseVTable {
        dtor: 0,
        #[cfg(target_family = "unix")]
        dtor2: 0,
        set_user_interface_language_code,
    });

    let c = Box::new(Component {
        vptr1,
        vptr2,
        vptr3,
        vptr4,
        destroy: destroy::<T>,
        memory: None,
        addin,
    });

    *component = Box::into_raw(c) as *mut c_void;
    1
}

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
