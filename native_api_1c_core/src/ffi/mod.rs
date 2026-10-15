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

/// Version of the platform capabilities a component is able to work with,
/// negotiated through the `SetPlatformCapabilities` export.
///
/// If the export is missing, the platform withholds message output and platform
/// info, and `Connection::add_error` silently does nothing.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppCapabilities {
    /// Component cannot work with what the platform offered
    Invalid = -1,
    /// The only capability level the platform defines so far
    Capabilities1 = 1,
}

impl AppCapabilities {
    /// Picks the highest level supported by both sides
    pub fn negotiate(offered: Self) -> Self {
        match offered {
            Self::Capabilities1 => Self::Capabilities1,
            Self::Invalid => Self::Invalid,
        }
    }
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

impl<const OFFSET: usize, T: AddInWrapper> This<OFFSET, T> {
    unsafe fn get_component(&mut self) -> &mut Component<T> {
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

    // rust part
    // Must stay at exactly this offset: `destroy_component` reads it through
    // `ComponentWrapper`, which knows nothing about the fields below
    destroy: unsafe extern "system" fn(*mut *mut Component<T>),
    /// Set once a panic escaped the add-in code; see `guard`
    poisoned: bool,

    // storage for additional interfaces
    memory_manager_ptr: Option<&'static MemoryManager>,
    connection_ptr: Option<&'static Connection>,
    locale: Option<String>,
    user_interface_language_code: Option<String>,

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
        poisoned: false,
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

/// Source reported to 1C for every panic caught at the FFI boundary
const PANIC_SOURCE: &str = "native_api_1c";

/// Runs a platform callback so that a Rust panic cannot unwind into the C++
/// frames of 1C, which aborts the whole process.
///
/// The add-in state is unknown after unwinding, so the component is poisoned on
/// the first panic and every later call returns `fallback` without touching it.
/// `context` names the callback in the message reported to 1C.
fn guard<T: AddInWrapper, R>(
    component: &mut Component<T>,
    context: &'static str,
    fallback: R,
    body: impl FnOnce(&mut Component<T>) -> R,
) -> R {
    if component.poisoned {
        return fallback;
    }

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        body(&mut *component)
    })) {
        Ok(value) => value,
        Err(payload) => {
            component.poisoned = true;
            report_panic(component, context, &payload);
            fallback
        }
    }
}

/// Sends the caught panic to 1C. Needs `SetPlatformCapabilities` to be exported,
/// otherwise the platform drops the message.
fn report_panic<T: AddInWrapper>(
    component: &Component<T>,
    context: &str,
    payload: &Box<dyn std::any::Any + Send>,
) {
    let Some(connection) = component.connection_ptr else {
        return;
    };

    let description = format!("{context}: {}", panic_message(payload));
    connection.add_error(
        connection::MessageCode::Fail,
        PANIC_SOURCE,
        &description,
    );
}

/// `panic!` accepts anything, but only the string forms carry a usable message
fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "panic with a payload that is neither &str nor String".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::c_void,
        mem::{offset_of, size_of},
    };

    use crate::interface::{
        AddInWrapper, AddInWrapperResult, ParamValue, ParamValues,
    };

    use super::{
        connection::Connection, create_component, destroy_component, guard,
        AppCapabilities, Component,
    };

    /// Minimal `AddInWrapper`, only needed to instantiate `Component` for the
    /// layout assertions below
    struct Dummy;

    impl AddInWrapper for Dummy {
        fn init(&mut self, _interface: &'static Connection) -> bool {
            true
        }
        fn done(&mut self) {}
        fn register_extension_as(&mut self) -> &[u16] {
            &[]
        }
        fn get_n_props(&self) -> usize {
            0
        }
        fn find_prop(&self, _name: &[u16]) -> Option<usize> {
            None
        }
        fn get_prop_name(
            &self,
            _num: usize,
            _alias: usize,
        ) -> Option<Vec<u16>> {
            None
        }
        fn get_prop_val(&self, _num: usize) -> AddInWrapperResult<ParamValue> {
            Err(())
        }
        fn set_prop_val(
            &mut self,
            _num: usize,
            _val: ParamValue,
        ) -> AddInWrapperResult<()> {
            Err(())
        }
        fn is_prop_readable(&self, _num: usize) -> bool {
            false
        }
        fn is_prop_writable(&self, _num: usize) -> bool {
            false
        }
        fn get_n_methods(&self) -> usize {
            0
        }
        fn find_method(&self, _name: &[u16]) -> Option<usize> {
            None
        }
        fn get_method_name(
            &self,
            _num: usize,
            _alias: usize,
        ) -> Option<Vec<u16>> {
            None
        }
        fn get_n_params(&self, _num: usize) -> usize {
            0
        }
        fn get_param_def_value(
            &self,
            _method_num: usize,
            _param_num: usize,
        ) -> Option<ParamValue> {
            None
        }
        fn has_ret_val(&self, _num: usize) -> bool {
            false
        }
        fn call_as_proc(
            &mut self,
            _num: usize,
            _params: &mut ParamValues,
        ) -> AddInWrapperResult<()> {
            Err(())
        }
        fn call_as_func(
            &mut self,
            _num: usize,
            _params: &mut ParamValues,
        ) -> AddInWrapperResult<ParamValue> {
            Err(())
        }
        fn set_locale(&mut self, _loc: &[u16]) {}
        fn set_user_interface_language_code(&mut self, _lang: &[u16]) {}
    }

    /// The four interface pointers must come first: `This::get_component` walks
    /// back from them by `OFFSET * size_of::<usize>()`
    #[test]
    fn interface_pointers_come_first() {
        assert_eq!(offset_of!(Component<Dummy>, init_done_ptr), 0);
        assert_eq!(
            offset_of!(Component<Dummy>, lang_extender_ptr),
            size_of::<usize>()
        );
        assert_eq!(
            offset_of!(Component<Dummy>, locale_ptr),
            2 * size_of::<usize>()
        );
        assert_eq!(
            offset_of!(Component<Dummy>, usr_lang_ptr),
            3 * size_of::<usize>()
        );
    }

    /// `destroy_component` reaches the destructor through `ComponentWrapper`,
    /// which assumes `destroy` sits right after the four interface pointers. If
    /// the real offset differs, `DestroyObject` calls whatever pointer happens
    /// to be there - normally the platform memory manager - and 1C dies with a
    /// DEP violation
    #[test]
    fn destroy_sits_where_component_wrapper_expects_it() {
        assert_eq!(
            offset_of!(Component<Dummy>, destroy),
            4 * size_of::<usize>()
        );
    }

    /// What the platform actually does: `GetClassObject` then `DestroyObject`.
    /// With a misplaced `destroy` field this jumps to a data pointer and takes
    /// the whole process down, which is how the bug reaches 1C users
    #[test]
    fn create_and_destroy_component_roundtrip() {
        unsafe {
            let mut component: *mut c_void = std::ptr::null_mut();

            assert_eq!(create_component(&mut component, Dummy), 1);
            assert!(!component.is_null());

            assert_eq!(destroy_component(&mut component), 0);
            assert!(component.is_null());
        }
    }

    /// Builds a live `Component` and frees it again whatever the body does
    fn with_component<R>(body: impl FnOnce(&mut Component<Dummy>) -> R) -> R {
        unsafe {
            let mut raw: *mut c_void = std::ptr::null_mut();
            assert_eq!(create_component(&mut raw, Dummy), 1);

            let result = body(&mut *(raw as *mut Component<Dummy>));

            assert_eq!(destroy_component(&mut raw), 0);
            result
        }
    }

    #[test]
    fn guard_returns_body_value_on_success() {
        with_component(|component| {
            assert!(guard(component, "test", false, |_| true));
            assert!(!component.poisoned);
        });
    }

    /// Without the guard this panic would unwind through the C++ frames of 1C
    /// and abort the process
    #[test]
    fn guard_poisons_component_and_returns_fallback_on_panic() {
        with_component(|component| {
            let value = guard(component, "test", -1, |_| panic!("boom"));

            assert_eq!(value, -1);
            assert!(component.poisoned);
        });
    }

    #[test]
    fn guard_skips_body_once_poisoned() {
        with_component(|component| {
            guard(component, "test", (), |_| panic!("boom"));

            let called = std::cell::Cell::new(false);
            guard(component, "test", (), |_| called.set(true));

            assert!(!called.get(), "body ran on a poisoned component");
        });
    }

    #[test]
    fn app_capabilities_negotiates_to_the_offered_level() {
        assert_eq!(
            AppCapabilities::negotiate(AppCapabilities::Capabilities1),
            AppCapabilities::Capabilities1
        );
        assert_eq!(
            AppCapabilities::negotiate(AppCapabilities::Invalid),
            AppCapabilities::Invalid
        );
    }
}
