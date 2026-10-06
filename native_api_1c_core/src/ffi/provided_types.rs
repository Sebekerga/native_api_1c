use std::{
    ffi::{c_int, c_void},
    ptr,
    slice::from_raw_parts,
};

use chrono::{Datelike, Timelike};

use crate::interface::ParamValue;

use super::memory_manager::{AllocationError, MemoryManager};

/// Type representing 1C date and time values
/// # Fields
/// * `sec` - seconds after the minute - [0, 60] including leap second
/// * `min` - minutes after the hour - [0, 59]
/// * `hour` - hours since midnight - [0, 23]
/// * `mday` - day of the month - [1, 31]
/// * `mon` - month of the year - [0, 11]
/// * `year` - years since 1900
/// * `wday` - days since Sunday - [0, 6]
/// * `yday` - days since January 1 - [0, 365]
/// * `isdst` - daylight savings time flag
/// * `gmtoff` - seconds east of UTC (unix only)
/// * `zone` - timezone abbreviation (unix only)
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Tm {
    pub sec: c_int,
    pub min: c_int,
    pub hour: c_int,
    pub mday: c_int,
    pub mon: c_int,
    pub year: c_int,
    pub wday: c_int,
    pub yday: c_int,
    pub isdst: c_int,

    #[cfg(target_family = "unix")]
    pub gmtoff: std::ffi::c_long,
    #[cfg(target_family = "unix")]
    pub zone: std::ffi::c_char,
}

impl From<&Tm> for chrono::NaiveDateTime {
    fn from(value: &Tm) -> Self {
        let default_time = chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let from_result = chrono::NaiveDate::from_ymd_opt(
            1900 + value.year,
            (1 + value.mon) as u32,
            (value.mday) as u32,
        )
        .ok_or(());
        if from_result.is_err() {
            return default_time;
        }

        let from_result = from_result.unwrap().and_hms_opt(
            value.hour as u32,
            value.min as u32,
            value.sec as u32,
        );
        if from_result.is_none() {
            return default_time;
        }

        from_result.unwrap()
    }
}

impl From<Tm> for chrono::NaiveDateTime {
    fn from(value: Tm) -> Self {
        Self::from(&value)
    }
}

impl From<&chrono::NaiveDateTime> for Tm {
    fn from(value: &chrono::NaiveDateTime) -> Self {
        Self {
            sec: value.time().second() as c_int,
            min: value.time().minute() as c_int,
            hour: value.time().hour() as c_int,
            mday: value.date().day() as c_int,
            mon: value.date().month0() as c_int,
            year: value.date().year() as c_int - 1900,
            wday: value.date().weekday().num_days_from_sunday() as c_int,
            yday: value.ordinal0() as c_int,
            isdst: 0,
            #[cfg(target_family = "unix")]
            gmtoff: 0,
            #[cfg(target_family = "unix")]
            zone: 0,
        }
    }
}

impl From<chrono::NaiveDateTime> for Tm {
    fn from(value: chrono::NaiveDateTime) -> Self {
        Self::from(&value)
    }
}

#[cfg(target_family = "unix")]
impl PartialEq for Tm {
    fn eq(&self, other: &Self) -> bool {
        self.sec == other.sec
            && self.min == other.min
            && self.hour == other.hour
            && self.mday == other.mday
            && self.mon == other.mon
            && self.year == other.year
            && self.wday == other.wday
            && self.yday == other.yday
            && self.isdst == other.isdst
            && self.gmtoff == other.gmtoff
            && self.zone == other.zone
    }
}

#[cfg(target_family = "windows")]
impl PartialEq for Tm {
    fn eq(&self, other: &Self) -> bool {
        self.sec == other.sec
            && self.min == other.min
            && self.hour == other.hour
            && self.mday == other.mday
            && self.mon == other.mon
            && self.year == other.year
            && self.wday == other.wday
            && self.yday == other.yday
            && self.isdst == other.isdst
    }
}

/// Type representing 1C variant values
/// # Fields
/// `mem` - pointer to the MemoryManager object
/// `variant` - pointer to the TVariant object
/// `result` - pointer to the result of the operation
pub struct ReturnValue<'a> {
    pub mem: &'a MemoryManager,
    pub variant: &'a mut TVariant,
    pub result: &'a mut bool,
}

#[allow(dead_code)]
impl<'a> ReturnValue<'a> {
    /// Creates a new ReturnValue object
    pub fn new(
        mem: &'a MemoryManager,
        variant: &'a mut TVariant,
        result: &'a mut bool,
    ) -> Self {
        Self {
            mem,
            variant,
            result,
        }
    }

    /// Sets the value of the ReturnValue object to empty
    pub fn set_empty(self) {
        self.variant.vt = VariantType::Empty;
    }

    /// Sets the value of the ReturnValue object to integer `i32`
    pub fn set_i32(self, val: i32) {
        self.variant.vt = VariantType::Int32;
        self.variant.value.i32 = val;
    }

    /// Sets the value of the ReturnValue object to bool `bool`
    pub fn set_bool(self, val: bool) {
        self.variant.vt = VariantType::Bool;
        self.variant.value.bool = val;
    }

    /// Sets the value of the ReturnValue object to float `f64`
    pub fn set_f64(self, val: f64) {
        self.variant.vt = VariantType::Double;
        self.variant.value.f64 = val;
    }

    /// Sets the value of the ReturnValue object to date-time `Tm`
    pub fn set_date(self, val: Tm) {
        self.variant.vt = VariantType::Time;
        self.variant.value.tm = val;
    }

    /// Sets the value of the ReturnValue object to UTF-16 `&[u16]`
    pub fn set_str(self, val: &[u16]) {
        let Ok(ptr) = self.mem.alloc_str(val.len()) else {
            *self.result = false;
            return;
        };

        unsafe {
            ptr::copy_nonoverlapping(val.as_ptr(), ptr.as_ptr(), val.len())
        };

        self.variant.vt = VariantType::WStr;
        self.variant.value.data_str.ptr = ptr.as_ptr();
        self.variant.value.data_str.len = val.len() as u32;
    }

    /// Sets the value of the ReturnValue object to blob `&[u8]`
    pub fn set_blob(self, val: &[u8]) {
        let Ok(ptr) = self.mem.alloc_blob(val.len()) else {
            *self.result = false;
            return;
        };

        unsafe {
            ptr::copy_nonoverlapping(val.as_ptr(), ptr.as_ptr(), val.len())
        };

        self.variant.vt = VariantType::Blob;
        self.variant.value.data_blob.ptr = ptr.as_ptr();
        self.variant.value.data_blob.len = val.len() as u32;
    }
}

impl<'a> From<&'a TVariant> for ParamValue {
    fn from(param: &'a TVariant) -> ParamValue {
        unsafe {
            match param.vt {
                VariantType::Empty
                | VariantType::Null
                | VariantType::Undefined => Self::Empty,
                VariantType::Bool => Self::Bool(param.value.bool),
                // The platform keeps integer values in `lVal` and fractional ones in
                // `dblVal` no matter which concrete VTYPE tag accompanies them
                VariantType::Int8
                | VariantType::Int16
                | VariantType::Int32
                | VariantType::UInt8
                | VariantType::UInt16
                | VariantType::Error => Self::I32(param.value.i32),
                VariantType::Float | VariantType::Double => {
                    Self::F64(param.value.f64)
                }
                VariantType::Time => Self::Date(param.value.tm),
                VariantType::WStr => Self::String(
                    from_raw_parts(
                        param.value.data_str.ptr,
                        param.value.data_str.len as usize,
                    )
                    .into(),
                ),
                VariantType::Blob => Self::Blob(
                    from_raw_parts(
                        param.value.data_blob.ptr,
                        param.value.data_blob.len as usize,
                    )
                    .into(),
                ),
                // `PStr` would need a codepage to decode and `Date` is an OLE automation
                // date; `Interface`, `Variant` and `ClsID` are not supported by Native API
                _ => Self::Empty,
            }
        }
    }
}

#[repr(u16)]
#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum VariantType {
    Empty = 0,
    Null,
    Int16,     //int16_t
    Int32,     //int32_t
    Float,     //float
    Double,    //double
    Date,      //DATE (double)
    Time,      //struct tm
    PStr,      //struct str    string
    Interface, //struct iface
    Error,     //int32_t errCode
    Bool,      //bool
    Variant,   //struct _tVariant *
    Int8,      //int8_t
    UInt8,     //uint8_t
    UInt16,    //uint16_t
    UInt32,    //uint32_t
    Int64,     //int64_t
    UInt64,    //uint64_t
    Int,       //int   Depends on architecture
    UInt,      //unsigned int  Depends on architecture
    HResult,   //long hRes
    WStr,      //struct wstr
    Blob,      //means in struct str binary data contain
    ClsID,     //UUID

    Undefined = 0xFFFF,
}

/// Type representing stored String data
/// # Fields
/// * `ptr` - pointer to the data
/// * `len` - length of the data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DataStr {
    pub ptr: *mut u16,
    pub len: u32,
}

/// Type representing stored Blob data
/// # Fields
/// * `ptr` - pointer to the data
/// * `len` - length of the data
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DataBlob {
    pub ptr: *mut u8,
    pub len: u32,
}

/// Type encapsulating 1C variant values
/// # Fields
/// * `bool` - boolean value
/// * `i32` - integer value
/// * `f64` - float value
/// * `tm` - date-time value
/// * `data_str` - UTF-16 string value
/// * `data_blob` - blob value
#[repr(C)]
pub union VariantValue {
    pub bool: bool,
    pub i32: i32,
    pub f64: f64,
    pub tm: Tm,
    pub data_str: DataStr,
    pub data_blob: DataBlob,
}

/// Type encapsulating 1C variant values for internal use
#[repr(C)]
pub struct TVariant {
    pub value: VariantValue,
    pub elements: u32, //Dimension for an one-dimensional array in pvarVal
    pub vt: VariantType,
}

impl Default for TVariant {
    fn default() -> Self {
        Self {
            value: VariantValue { bool: false },
            elements: 0,
            vt: VariantType::Empty,
        }
    }
}

impl TVariant {
    /// # Safety
    /// This function is unsafe because it manipulates pointers, provided by the 1C platform.
    /// Function is safe as long as 1C platform provides valid pointers.
    pub unsafe fn update_to_str(
        &mut self,
        mem_mngr: &MemoryManager,
        v: &[u16],
    ) -> Result<u32, AllocationError> {
        let old_pointer = self.value.data_str.ptr;

        let ptr = mem_mngr.alloc_str(v.len())?;
        ptr::copy_nonoverlapping(v.as_ptr(), ptr.as_ptr(), v.len());

        self.value.data_str.ptr = ptr.as_ptr();
        self.value.data_str.len = v.len() as u32;

        mem_mngr.free_memory(&mut old_pointer.cast::<c_void>());

        self.vt = VariantType::WStr;

        Ok(self.value.data_str.len)
    }

    /// # Safety
    /// This function is unsafe because it manipulates pointers, provided by the 1C platform.
    /// Function is safe as long as 1C platform provides valid pointers.
    pub unsafe fn update_to_blob(
        &mut self,
        mem_mngr: &MemoryManager,
        v: &[u8],
    ) -> Result<u32, AllocationError> {
        let old_pointer = self.value.data_blob.ptr;

        let ptr = mem_mngr.alloc_blob(v.len())?;
        ptr::copy_nonoverlapping(v.as_ptr(), ptr.as_ptr(), v.len());

        self.value.data_blob.ptr = ptr.as_ptr();
        self.value.data_blob.len = v.len() as u32;

        mem_mngr.free_memory(&mut old_pointer.cast::<c_void>());

        self.vt = VariantType::Blob;

        Ok(self.value.data_blob.len)
    }

    pub fn update_to_bool(&mut self, v: bool) {
        self.value.bool = v;
        self.vt = VariantType::Bool;
    }

    pub fn update_to_i32(&mut self, v: i32) {
        self.value.i32 = v;
        self.vt = VariantType::Int32;
    }

    pub fn update_to_f64(&mut self, v: f64) {
        self.value.f64 = v;
        self.vt = VariantType::Double;
    }

    pub fn update_to_date(&mut self, v: Tm) {
        self.value.tm = v;
        self.vt = VariantType::Time;
    }

    pub fn update_from_return(
        &mut self,
        mem_mngr: &MemoryManager,
        value: &ParamValue,
    ) {
        match value {
            ParamValue::Empty => self.vt = VariantType::Empty,
            ParamValue::Bool(v) => self.update_to_bool(*v),
            ParamValue::I32(v) => self.update_to_i32(*v),
            ParamValue::F64(v) => self.update_to_f64(*v),
            ParamValue::Date(v) => self.update_to_date(*v),
            ParamValue::String(v) => {
                let _ = unsafe { self.update_to_str(mem_mngr, v.as_slice()) };
            }
            ParamValue::Blob(v) => {
                let _ = unsafe { self.update_to_blob(mem_mngr, v.as_slice()) };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variant(vt: VariantType, value: VariantValue) -> TVariant {
        TVariant {
            value,
            elements: 0,
            vt,
        }
    }

    #[test]
    fn empty_tags_map_to_empty() {
        for vt in [
            VariantType::Empty,
            VariantType::Null,
            VariantType::Undefined,
        ] {
            let param = variant(vt, VariantValue { bool: true });
            assert_eq!(ParamValue::from(&param), ParamValue::Empty);
        }
    }

    #[test]
    fn bool_tag_maps_to_bool() {
        let param = variant(VariantType::Bool, VariantValue { bool: true });
        assert_eq!(ParamValue::from(&param), ParamValue::Bool(true));
    }

    #[test]
    fn integer_tags_read_lval() {
        for vt in [
            VariantType::Int8,
            VariantType::Int16,
            VariantType::Int32,
            VariantType::UInt8,
            VariantType::UInt16,
            VariantType::Error,
        ] {
            let param = variant(vt, VariantValue { i32: -7 });
            assert_eq!(ParamValue::from(&param), ParamValue::I32(-7));
        }
    }

    #[test]
    fn fractional_tags_read_dblval() {
        for vt in [VariantType::Float, VariantType::Double] {
            let param = variant(vt, VariantValue { f64: 1.5 });
            assert_eq!(ParamValue::from(&param), ParamValue::F64(1.5));
        }
    }

    #[test]
    fn time_tag_maps_to_date() {
        let tm = Tm {
            year: 126,
            mon: 9,
            mday: 6,
            hour: 12,
            min: 34,
            sec: 56,
            ..Tm::default()
        };
        let param = variant(VariantType::Time, VariantValue { tm });
        assert_eq!(ParamValue::from(&param), ParamValue::Date(tm));
    }

    #[test]
    fn wstr_tag_maps_to_string() {
        let data: Vec<u16> = vec![0x41f, 0x440, 0x438, 0x432, 0x435, 0x442];
        let param = variant(
            VariantType::WStr,
            VariantValue {
                data_str: DataStr {
                    ptr: data.as_ptr() as *mut u16,
                    len: data.len() as u32,
                },
            },
        );
        assert_eq!(ParamValue::from(&param), ParamValue::String(data));
    }

    #[test]
    fn blob_tag_maps_to_blob() {
        let data: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let param = variant(
            VariantType::Blob,
            VariantValue {
                data_blob: DataBlob {
                    ptr: data.as_ptr() as *mut u8,
                    len: data.len() as u32,
                },
            },
        );
        assert_eq!(ParamValue::from(&param), ParamValue::Blob(data));
    }

    #[test]
    fn unsupported_tags_map_to_empty() {
        for vt in [
            VariantType::PStr,
            VariantType::Interface,
            VariantType::Variant,
            VariantType::ClsID,
            VariantType::Date,
            VariantType::Int64,
            VariantType::UInt64,
            VariantType::UInt32,
            VariantType::Int,
            VariantType::UInt,
            VariantType::HResult,
        ] {
            let param = variant(vt, VariantValue { i32: 1 });
            assert_eq!(ParamValue::from(&param), ParamValue::Empty);
        }
    }
}
