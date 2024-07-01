use std::ops::{Index, IndexMut};

use crate::ffi::{connection::Connection, provided_types::Tm};

/// Represents 1C variant values for parameters in safe Rust code.
#[derive(Clone, Debug)]
pub enum ParamValue {
    /// Empty value
    Empty,
    /// Boolean value
    Bool(bool),
    /// Integer value
    I32(i32),
    /// Float value
    F64(f64),
    /// Date-time value
    Date(Tm),
    /// UTF-16 string value
    String(Vec<u16>),
    /// Blob value
    Blob(Vec<u8>),
}

impl ParamValue {
    pub fn set_bool(&mut self, val: bool) {
        *self = Self::Bool(val);
    }

    pub fn set_i32(&mut self, val: i32) {
        *self = Self::I32(val);
    }

    pub fn set_f64(&mut self, val: f64) {
        *self = Self::F64(val);
    }

    pub fn set_date(&mut self, val: Tm) {
        *self = Self::Date(val);
    }

    pub fn set_str(&mut self, val: Vec<u16>) {
        *self = Self::String(val);
    }

    pub fn set_blob(&mut self, val: Vec<u8>) {
        *self = Self::Blob(val);
    }
}

impl PartialEq for ParamValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::I32(a), Self::I32(b)) => a == b,
            (Self::F64(a), Self::F64(b)) => a == b,
            (Self::Date(a), Self::Date(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Blob(a), Self::Blob(b)) => a == b,
            _ => false,
        }
    }
}

/// Represents 1C variant values for return values in safe Rust code.
/// Only creator of the object can set the initial value, therefor has
/// control over count of values.
#[derive(Clone)]
pub struct ParamValues {
    values: Vec<ParamValue>,
}

impl ParamValues {
    pub fn new(values: Vec<ParamValue>) -> Self {
        Self { values }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<ParamValue> {
        self.values.iter()
    }
}

impl Index<usize> for ParamValues {
    type Output = ParamValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl IndexMut<usize> for ParamValues {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

pub type AddInWrapperResult<T> = Result<T, ()>;

/// `AddInWrapper` trait is used to implement the 1C AddIn interface,
/// and is used in FFI to get necessary information about the AddIn
/// and call its methods.
///
/// All trait methods, that return bool, should return true if the operation was successful
/// and false otherwise.
///
/// Many of them are equivalents of methods in the 1C AddIn interface, and their
/// descriptions can be found in the [1C documentation](https://its.1c.ru/db/metod8dev/content/3221/hdoc).
#[allow(clippy::result_unit_err)]
pub trait AddInWrapper {
    /// Equivalent to `Init` from Native API interface and is called when the AddIn is loaded by 1C platform
    /// and is used to pass the pointer to the 1C Connection object
    /// # Arguments
    /// * `interface` - pointer to the 1C Connection object
    /// # Returns
    /// `bool` - operation success status
    fn init(&mut self, interface: &'static Connection) -> bool;

    /// Equivalent to `GetInfo` from Native API interface and is used to get Native API version used by AddIn, either
    /// `1000` meaning 1.0 or `2000` meaning 2.0. It will be later removed to only
    /// support 2.0 version.
    /// # Returns
    /// `u16` - Native API version
    fn get_info(&self) -> u16 {
        2000
    }

    /// Equivalent to `Done` from Native API interface and is called when the AddIn is unloaded by 1C platform
    fn done(&mut self);

    /// Equivalent to `RegisterExtensionAs` from Native API interface and is used to get the name of the AddIn
    /// as it will be shown in 1C platform
    /// # Returns
    /// `&[u16]` - name of the AddIn in UTF-16 with null-terminator
    fn register_extension_as(&mut self) -> &[u16];

    /// Equivalent to `GetNProps` from Native API interface and is used to get the number of properties
    /// that the AddIn has that can be accessed by 1C platform
    /// # Returns
    /// `usize` - number of properties
    fn get_n_props(&self) -> usize;

    /// Equivalent to `FindProp` from Native API interface and is used to get the index of the property
    /// with the given name
    /// # Arguments
    /// * `name` - name of the property in UTF-16
    /// # Returns
    /// `Option<usize>` - index of the property or None if the property was not found
    fn find_prop(&self, name: &[u16]) -> Option<usize>;

    /// Equivalent to `GetPropName` from Native API interface and is used to get the name of the property
    /// with the given index
    /// # Arguments
    /// * `num` - index of the property
    /// * `alias` - alias of the property, usually 0 for Russian and 1 for English
    /// # Returns
    /// `Option<Vec<u16>>` - name of the property in UTF-16 or None if the property was not found
    fn get_prop_name(&self, num: usize, alias: usize) -> Option<Vec<u16>>;

    /// Equivalent to `GetPropVal` from Native API interface and is used to get the value of the property
    /// with the given index
    /// # Arguments
    /// * `num` - index of the property
    /// # Returns
    /// `AddInWrapperResult<ParamValue>` - value of the property, or error if the property was not found
    fn get_prop_val(&self, num: usize) -> AddInWrapperResult<ParamValue>;

    /// Equivalent to `SetPropVal` from Native API interface and is used to set the value of the property
    /// with the given index
    /// # Arguments
    /// * `num` - index of the property
    /// * `val` - value of the property
    /// # Returns
    /// `AddInWrapperResult<()>` - operation result
    fn set_prop_val(
        &mut self,
        num: usize,
        val: ParamValue,
    ) -> AddInWrapperResult<()>;

    /// Equivalent to `IsPropReadable` from Native API interface and is used to check if the property
    /// with the given index is readable
    /// # Arguments
    /// * `num` - index of the property
    /// # Returns
    /// `bool` - if the property is readable
    fn is_prop_readable(&self, num: usize) -> bool;

    /// Equivalent to `IsPropWritable` from Native API interface and is used to check if the property
    /// with the given index is writable
    /// # Arguments
    /// * `num` - index of the property
    /// # Returns
    /// `bool` - if the property is writable
    fn is_prop_writable(&self, num: usize) -> bool;

    /// Equivalent to `GetNMethods` from Native API interface and is used to get the number of methods
    /// that the AddIn has that can be called by 1C platform
    /// # Returns
    /// `usize` - number of methods
    fn get_n_methods(&self) -> usize;

    /// Equivalent to `FindMethod` from Native API interface and is used to get the index of method
    /// with the given name
    /// # Arguments
    /// * `name` - name of method in UTF-16
    /// # Returns
    /// `Option<usize>` - index of method or None if method was not found
    fn find_method(&self, name: &[u16]) -> Option<usize>;

    /// Equivalent to `GetMethodName` from Native API interface and is used to get the name of method
    /// with the given index
    /// # Arguments
    /// * `num` - index of method
    /// * `alias` - alias of method, usually 0 for Russian and 1 for English
    /// # Returns
    /// `Option<Vec<u16>>` - name of method in UTF-16 or None if method was not found
    fn get_method_name(&self, num: usize, alias: usize) -> Option<Vec<u16>>;

    /// Equivalent to `GetNParams` from Native API interface and is used to get the number of parameters
    /// that method with the given index has
    /// # Arguments
    /// * `num` - index of method
    /// # Returns
    /// `usize` - number of parameters
    fn get_n_params(&self, num: usize) -> usize;

    /// Equivalent to `GetParamDefValue` from Native API interface and is used to get the default value
    /// of the parameter
    /// # Arguments
    /// * `method_num` - index of method
    /// * `param_num` - index of parameter
    /// # Returns
    /// `Option<ParamValue>` - default value of the parameter or None if the parameter was not found
    fn get_param_def_value(
        &self,
        method_num: usize,
        param_num: usize,
    ) -> Option<ParamValue>;

    /// Equivalent to `HasRetVal` from Native API interface and is used to check if method
    /// with the given index returns a value
    /// # Arguments
    /// * `method_num` - index of method
    /// # Returns
    /// `bool` - if method returns a value
    fn has_ret_val(&self, method_num: usize) -> bool;

    /// Equivalent to `CallAsProc` from Native API interface and is used to call method
    /// with the given index as a procedure, meaning that it does not return a value
    /// # Arguments
    /// * `method_num` - index of method
    /// * `params` - slice of ParamValue objects that contain the parameters
    /// # Returns
    /// `AddInWrapperResult<()>` - operation result
    fn call_as_proc(
        &mut self,
        method_num: usize,
        params: &mut ParamValues,
    ) -> AddInWrapperResult<()>;

    /// Equivalent to `CallAsFunc` from Native API interface and is used to call method
    /// with the given index as a function, meaning that it returns a value
    /// # Arguments
    /// * `method_num` - index of method
    /// * `params` - slice of ParamValue objects that contain the parameters
    /// # Returns
    /// `AddInWrapperResult<ParamValue>` - result of the method
    fn call_as_func(
        &mut self,
        method_num: usize,
        params: &mut ParamValues,
    ) -> AddInWrapperResult<ParamValue>;

    /// Equivalent to `SetLocale` from Native API interface and is used to set the locale
    /// of the AddIn. It's marked as deprecated in 1C documentation, but is still available
    /// for use with platform versions prior to 8.3.21
    fn set_locale(&mut self, loc: &[u16]);

    /// Equivalent to `SetUserInterfaceLanguageCode` from Native API interface and is used to
    /// pass the language code of the 1C platform interface to the AddIn
    /// # Arguments
    /// * `lang` - language code in UTF-16, two letters
    fn set_user_interface_language_code(&mut self, lang: &[u16]);
}
