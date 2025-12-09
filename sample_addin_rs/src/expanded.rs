#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2021::*;
use std::sync::Arc;
use native_api_1c::{
    native_api_1c_core::{ffi::connection::Connection, interface::ParamValue},
    native_api_1c_macro::{extern_functions, AddIn},
};
pub struct SampleAddIn {
    /// connection with 1C, used for calling events
    /// Arc is used to allow multiple threads to access the connection
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,
    /// Property, readable and writable from 1C
    #[add_in_prop(
        ty = Int,
        name = "MyProp",
        name_ru = "МоеСвойство",
        readable,
        writable
    )]
    pub some_prop: i32,
    /// Property, readable from 1C but not writable
    #[add_in_prop(
        ty = Int,
        name = "ProtectedProp",
        name_ru = "ЗащищенноеСвойство",
        readable
    )]
    pub protected_prop: i32,
    /// Function, taking one or two arguments and returning a result
    /// In 1C it can be called as:
    /// ```bsl
    ///  CompObj.MyFunction(10, 15); // 2nd arg = 15
    ///  CompObj.MyFunction(10);     // 2nd arg = 12 (default value)
    /// ```
    /// If function returns an error, but does not panic, then 1C will throw an exception
    #[add_in_func(name = "MyFunction", name_ru = "МояФункция")]
    #[arg(ty = Int)]
    #[arg(ty = Int, default = 12)]
    #[returns(ty = Int, result)]
    pub my_function: fn(&Self, i32, i64) -> Result<i32, ()>,
    /// Function, taking no arguments and returning nothing
    #[add_in_func(name = "MyProcedure", name_ru = "МояПроцедура")]
    pub my_procedure: fn(&mut Self),
    /// Function that accepts Any types
    #[add_in_func(name = "CompareAny", name_ru = "СравнитьЛюбое")]
    #[arg(ty = Any)]
    #[arg(ty = Any)]
    #[returns(ty = Bool)]
    pub compare_any: fn(ParamValue, ParamValue) -> bool,
    /// Private field, not visible from 1C
    private_field: i32,
}
impl native_api_1c::native_api_1c_core::interface::AddInWrapper for SampleAddIn {
    fn init(
        &mut self,
        interface: &'static native_api_1c::native_api_1c_core::ffi::connection::Connection,
    ) -> bool {
        self.connection = std::sync::Arc::new(Some(interface));
        true
    }
    fn get_info(&self) -> u16 {
        2000
    }
    fn done(&mut self) {}
    fn register_extension_as(&mut self) -> &[u16] {
        &{
            const ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_UTF8: &str = "SampleAddIn";
            const ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_LEN: usize = ::utf16_lit::internals::length_as_utf16(
                ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_UTF8,
            ) + 1;
            const ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_UTF16: [u16; ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_LEN] = {
                let mut buffer = [0u16; ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_LEN];
                let mut bytes = ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_UTF8
                    .as_bytes();
                let mut i = 0;
                while let Some((ch, rest)) = ::utf16_lit::internals::next_code_point(
                    bytes,
                ) {
                    bytes = rest;
                    if ch & 0xFFFF == ch {
                        buffer[i] = ch as u16;
                        i += 1;
                    } else {
                        let code = ch - 0x1_0000;
                        buffer[i] = 0xD800 | ((code >> 10) as u16);
                        buffer[i + 1] = 0xDC00 | ((code as u16) & 0x3FF);
                        i += 2;
                    }
                }
                buffer
            };
            ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_UTF16
        }
    }
    fn find_prop(&self, name: &[u16]) -> Option<usize> {
        if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil("MyProp")
            == name
        {
            return Some(0usize);
        }
        if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
            "МоеСвойство",
        ) == name
        {
            return Some(0usize);
        }
        if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
            "ProtectedProp",
        ) == name
        {
            return Some(1usize);
        }
        if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
            "ЗащищенноеСвойство",
        ) == name
        {
            return Some(1usize);
        }
        None
    }
    fn get_n_props(&self) -> usize {
        2usize
    }
    fn get_prop_name(&self, num: usize, alias: usize) -> Option<Vec<u16>> {
        if num == 0usize && alias == 0 {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "MyProp",
                    )
                    .into(),
            );
        }
        if num == 0usize {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "МоеСвойство",
                    )
                    .into(),
            );
        }
        if num == 1usize && alias == 0 {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "ProtectedProp",
                    )
                    .into(),
            );
        }
        if num == 1usize {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "ЗащищенноеСвойство",
                    )
                    .into(),
            );
        }
        None
    }
    fn is_prop_readable(&self, num: usize) -> bool {
        if num == 0usize {
            return true;
        }
        if num == 1usize {
            return true;
        }
        false
    }
    fn is_prop_writable(&self, num: usize) -> bool {
        if num == 0usize {
            return true;
        }
        if num == 1usize {
            return false;
        }
        false
    }
    fn get_prop_val(
        &self,
        num: usize,
    ) -> native_api_1c::native_api_1c_core::interface::AddInWrapperResult<
        native_api_1c::native_api_1c_core::interface::ParamValue,
    > {
        if num == 0usize {
            return Ok({
                native_api_1c::native_api_1c_core::interface::ParamValue::I32(
                    self.some_prop.clone().into(),
                )
            });
        }
        if num == 1usize {
            return Ok({
                native_api_1c::native_api_1c_core::interface::ParamValue::I32(
                    self.protected_prop.clone().into(),
                )
            });
        }
        return Err(());
    }
    fn set_prop_val(
        &mut self,
        num: usize,
        val: native_api_1c::native_api_1c_core::interface::ParamValue,
    ) -> native_api_1c::native_api_1c_core::interface::AddInWrapperResult<()> {
        if num == 0usize {
            self.some_prop = {
                match val {
                    native_api_1c::native_api_1c_core::interface::ParamValue::I32(
                        val,
                    ) => Ok(val),
                    _ => Err(()),
                }?
                    .clone()
            }
                .into();
            return Ok(());
        }
        return Err(());
    }
    fn find_method(&self, name: &[u16]) -> Option<usize> {
        if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
            "MyFunction",
        ) == name
        {
            return Some(0usize);
        }
        if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
            "МояФункция",
        ) == name
        {
            return Some(0usize);
        }
        if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
            "MyProcedure",
        ) == name
        {
            return Some(1usize);
        }
        if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
            "МояПроцедура",
        ) == name
        {
            return Some(1usize);
        }
        if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
            "CompareAny",
        ) == name
        {
            return Some(2usize);
        }
        if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
            "СравнитьЛюбое",
        ) == name
        {
            return Some(2usize);
        }
        None
    }
    fn get_method_name(&self, num: usize, alias: usize) -> Option<Vec<u16>> {
        if num == 0usize && alias == 0 {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "MyFunction",
                    )
                    .into(),
            );
        }
        if num == 0usize {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "МояФункция",
                    )
                    .into(),
            );
        }
        if num == 0usize && alias == 0 {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "MyFunction",
                    )
                    .into(),
            );
        }
        if num == 0usize {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "МояФункция",
                    )
                    .into(),
            );
        }
        if num == 1usize && alias == 0 {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "MyProcedure",
                    )
                    .into(),
            );
        }
        if num == 1usize {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "МояПроцедура",
                    )
                    .into(),
            );
        }
        if num == 0usize && alias == 0 {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "MyFunction",
                    )
                    .into(),
            );
        }
        if num == 0usize {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "МояФункция",
                    )
                    .into(),
            );
        }
        if num == 0usize && alias == 0 {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "MyFunction",
                    )
                    .into(),
            );
        }
        if num == 0usize {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "МояФункция",
                    )
                    .into(),
            );
        }
        if num == 1usize && alias == 0 {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "MyProcedure",
                    )
                    .into(),
            );
        }
        if num == 1usize {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "МояПроцедура",
                    )
                    .into(),
            );
        }
        if num == 2usize && alias == 0 {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "CompareAny",
                    )
                    .into(),
            );
        }
        if num == 2usize {
            return Some(
                native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        "СравнитьЛюбое",
                    )
                    .into(),
            );
        }
        None
    }
    fn get_n_methods(&self) -> usize {
        3usize
    }
    fn get_n_params(&self, num: usize) -> usize {
        if num == 0usize {
            return 2usize;
        }
        if num == 1usize {
            return 0usize;
        }
        if num == 2usize {
            return 2usize;
        }
        0
    }
    fn has_ret_val(&self, method_num: usize) -> bool {
        if method_num == 0usize {
            return true;
        }
        if method_num == 1usize {
            return false;
        }
        if method_num == 2usize {
            return true;
        }
        false
    }
    fn call_as_proc(
        &mut self,
        method_num: usize,
        params: &mut native_api_1c::native_api_1c_core::interface::ParamValues,
    ) -> native_api_1c::native_api_1c_core::interface::AddInWrapperResult<()> {
        if method_num == 0usize {
            let param_1 = {
                match params[0usize] {
                    native_api_1c::native_api_1c_core::interface::ParamValue::I32(
                        val,
                    ) => Ok(val),
                    _ => Err(()),
                }?
                    .clone()
            };
            let param_1 = param_1.clone().into();
            let param_2 = {
                match params[1usize] {
                    native_api_1c::native_api_1c_core::interface::ParamValue::I32(
                        val,
                    ) => Ok(val),
                    _ => Err(()),
                }?
                    .clone()
            };
            let param_2 = param_2.clone().into();
            let call_result = (self.my_function)(self, param_1, param_2);
            if call_result.is_err() {
                return Err(());
            }
            let call_result = call_result.unwrap();
        }
        if method_num == 1usize {
            let call_result = (self.my_procedure)(self);
        }
        if method_num == 2usize {
            let param_1 = &params[0usize];
            let param_1 = param_1.clone().into();
            let param_2 = &params[1usize];
            let param_2 = param_2.clone().into();
            let call_result = (self.compare_any)(param_1, param_2);
        }
    }
    fn call_as_func(
        &mut self,
        method_num: usize,
        params: &mut native_api_1c::native_api_1c_core::interface::ParamValues,
    ) -> native_api_1c::native_api_1c_core::interface::AddInWrapperResult<
        native_api_1c::native_api_1c_core::interface::ParamValue,
    > {
        if method_num == 0usize {
            let param_1 = {
                match params[0usize] {
                    native_api_1c::native_api_1c_core::interface::ParamValue::I32(
                        val,
                    ) => Ok(val),
                    _ => Err(()),
                }?
                    .clone()
            };
            let param_1 = param_1.clone().into();
            let param_2 = {
                match params[1usize] {
                    native_api_1c::native_api_1c_core::interface::ParamValue::I32(
                        val,
                    ) => Ok(val),
                    _ => Err(()),
                }?
                    .clone()
            };
            let param_2 = param_2.clone().into();
            let call_result = (self.my_function)(self, param_1, param_2);
            if call_result.is_err() {
                return Err(());
            }
            let call_result = call_result.unwrap();
            let val = {
                native_api_1c::native_api_1c_core::interface::ParamValue::I32(
                    call_result.clone().into(),
                )
            };
            return Ok(val);
        }
        if method_num == 2usize {
            let param_1 = &params[0usize];
            let param_1 = param_1.clone().into();
            let param_2 = &params[1usize];
            let param_2 = param_2.clone().into();
            let call_result = (self.compare_any)(param_1, param_2);
            let val = {
                native_api_1c::native_api_1c_core::interface::ParamValue::Bool(
                    call_result.clone().into(),
                )
            };
            return Ok(val);
        }
        Err(())
    }
    fn get_param_def_value(
        &self,
        method_num: usize,
        param_num: usize,
    ) -> Option<native_api_1c::native_api_1c_core::interface::ParamValue> {
        if method_num == 0usize {
            if param_num == 1usize {
                return Some({
                    native_api_1c::native_api_1c_core::interface::ParamValue::I32(
                        12.clone().into(),
                    )
                });
            }
            return None;
        }
        if method_num == 1usize {
            return None;
        }
        if method_num == 2usize {
            return None;
        }
        None
    }
    fn set_locale(&mut self, loc: &[u16]) {}
    fn set_user_interface_language_code(&mut self, lang: &[u16]) {}
}
impl Default for SampleAddIn {
    fn default() -> Self {
        Self {
            connection: Arc::new(None),
            some_prop: 0,
            protected_prop: 50,
            my_function: Self::my_function_inner,
            my_procedure: Self::my_procedure_inner,
            private_field: 100,
            compare_any: Self::compare_any,
        }
    }
}
impl SampleAddIn {
    fn my_function_inner(&self, arg: i32, arg_maybe_default: i64) -> Result<i32, ()> {
        Ok(
            self.protected_prop + self.some_prop + arg + self.private_field
                + arg_maybe_default as i32,
        )
    }
    fn my_procedure_inner(&mut self) {
        self.protected_prop += 10;
    }
    fn compare_any(arg1: ParamValue, arg2: ParamValue) -> bool {
        arg1 == arg2
    }
}
pub static mut PLATFORM_CAPABILITIES: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(
    -1,
);
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn GetAttachType() -> native_api_1c::native_api_1c_core::ffi::AttachType {
    native_api_1c::native_api_1c_core::ffi::AttachType::Any
}
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn DestroyObject(
    component: *mut *mut std::ffi::c_void,
) -> std::ffi::c_long {
    native_api_1c::native_api_1c_core::ffi::destroy_component(component)
}
#[allow(non_snake_case)]
#[no_mangle]
pub unsafe extern "C" fn GetClassObject(
    name: *const u16,
    component: *mut *mut std::ffi::c_void,
) -> std::ffi::c_long {
    match *name as u8 {
        48u8 => {
            let add_in = SampleAddIn::default();
            native_api_1c::native_api_1c_core::ffi::create_component(component, add_in)
        }
        _ => 0,
    }
}
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn GetClassNames() -> *const u16 {
    {
        const ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_UTF8: &str = "0";
        const ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_LEN: usize = ::utf16_lit::internals::length_as_utf16(
            ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_UTF8,
        ) + 1;
        const ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_UTF16: [u16; ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_LEN] = {
            let mut buffer = [0u16; ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_LEN];
            let mut bytes = ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_UTF8
                .as_bytes();
            let mut i = 0;
            while let Some((ch, rest)) = ::utf16_lit::internals::next_code_point(bytes) {
                bytes = rest;
                if ch & 0xFFFF == ch {
                    buffer[i] = ch as u16;
                    i += 1;
                } else {
                    let code = ch - 0x1_0000;
                    buffer[i] = 0xD800 | ((code >> 10) as u16);
                    buffer[i + 1] = 0xDC00 | ((code as u16) & 0x3FF);
                    i += 2;
                }
            }
            buffer
        };
        ABC678_PREFIX_THAT_SHOULD_NEVER_CLASH_WITH_OUTER_SCOPE_UTF16
    }
        .as_ptr()
}
