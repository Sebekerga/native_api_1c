use std::ffi::{c_long, c_ushort};

use super::{provided_types::TVariant, string_utils::os_string_nil};

/// Message codes that can be used in `Connection::add_error` method
/// to specify message type.
/// See [1C documentation](https://its.1c.ru/db/content/metod8dev/src/developers/platform/i8103221.htm#_com_infomessage)
pub enum MessageCode {
    /// Error without icon
    None = 1000,
    /// Error with ">" icon
    Ordinary = 1001,
    /// Error with "!" icon
    Attention = 1002,
    /// Error with "!!" icon
    Important = 1003,
    /// Error with "!!!" icon
    VeryImportant = 1004,
    /// Error with "i" icon
    Info = 1005,
    /// Error with "err" icon
    Fail = 1006,
    /// Shows a dialog with "MB_ICONEXCLAMATION" icon
    DialogAttention = 1007,
    /// Shows a dialog with "MB_ICONINFORMATION" icon
    DialogInfo = 1008,
    /// Shows a dialog with "MB_ICONERROR" icon
    DialogFail = 1009,
}

/// VTable for Connection object, derived from Native API interface. See original
/// C++ implementation in [example project](https://its.1c.ru/db/files/1CITS/EXE/VNCOMPS/VNCOMPS.zip)
/// from 1C documentation
#[repr(C)]
struct ConnectionVTable {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    add_error: unsafe extern "system" fn(
        &Connection,
        c_ushort,
        *const u16,
        *const u16,
        c_long,
    ) -> bool,
    read: unsafe extern "system" fn(
        &Connection,
        *mut u16,
        &mut TVariant,
        c_long,
        *mut *mut u16,
    ) -> bool,
    write:
        unsafe extern "system" fn(&Connection, *mut u16, &mut TVariant) -> bool,
    register_profile_as:
        unsafe extern "system" fn(&Connection, *mut u16) -> bool,
    set_event_buffer_depth:
        unsafe extern "system" fn(&Connection, c_long) -> bool,
    get_event_buffer_depth: unsafe extern "system" fn(&Connection) -> c_long,
    external_event: unsafe extern "system" fn(
        &Connection,
        *mut u16,
        *mut u16,
        *mut u16,
    ) -> bool,
    clean_event_buffer: unsafe extern "system" fn(&Connection),
    set_status_line: unsafe extern "system" fn(&Connection, *mut u16) -> bool,
    reset_status_line: unsafe extern "system" fn(&Connection),
}

/// Connection object, used to communicate with 1C platform after the AddIn is loaded
#[repr(C)]
pub struct Connection {
    vptr1: &'static ConnectionVTable,
}

impl Connection {
    /// Equivalent to `AddError` from Native API interface and is used to add an error to the 1C platform
    /// # Arguments
    /// * `code` - message code, see [MessageCode](enum.MessageCode)
    /// * `source` - source of the error
    /// * `description` - description of the error
    /// # Returns
    /// `bool` - operation success status
    pub fn add_error(
        &self,
        code: MessageCode,
        source: &str,
        description: &str,
    ) -> bool {
        unsafe {
            let source_wstr = os_string_nil(source);
            let description_wstr = os_string_nil(description);
            (self.vptr1.add_error)(
                self,
                code as u16,
                source_wstr.as_ptr(),
                description_wstr.as_ptr(),
                0,
            )
        }
    }

    /// Equivalent to `ExternalEvent` from Native API interface and is used to send an external event to the 1C platform
    /// # Arguments
    /// * `caller` - name of the event caller
    /// * `name` - name of the event
    /// * `data` - data of the event
    /// # Returns
    /// `bool` - operation success status
    pub fn external_event(&self, caller: &str, name: &str, data: &str) -> bool {
        unsafe {
            let mut caller_wstr = os_string_nil(caller);
            let mut name_wstr = os_string_nil(name);
            let mut data_wstr = os_string_nil(data);
            (self.vptr1.external_event)(
                self,
                caller_wstr.as_mut_ptr(),
                name_wstr.as_mut_ptr(),
                data_wstr.as_mut_ptr(),
            )
        }
    }

    /// Equivalent to `SetEventBufferDepth` from Native API interface
    /// # Arguments
    /// * `depth` - new event buffer depth
    pub fn set_event_buffer_depth(&self, depth: c_long) -> bool {
        unsafe { (self.vptr1.set_event_buffer_depth)(self, depth) }
    }

    /// Equivalent to `GetEventBufferDepth` from Native API interface
    /// # Returns
    /// `c_long` - current event buffer depth
    pub fn get_event_buffer_depth(&self) -> c_long {
        unsafe { (self.vptr1.get_event_buffer_depth)(self) }
    }
}
