use crate::ffi::{
    connection::Connection,
    provided_types::{ParamValue, ReturnValue},
};

/// `AddInWrapper` trait is used to implement the 1C AddIn interface,
/// and is used in FFI to get necessary information about the AddIn
/// and call its methods.
///
/// All trait methods, that return bool, should return true if the operation was successful
/// and false otherwise.
///
/// Many of the are equivalents of methods in the 1C AddIn interface, and their
/// descriptions can be found in the [1C documentation](https://its.1c.ru/db/metod8dev/content/3221/hdoc).
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
    /// `&[u16]` - name of the AddIn in UTF-16
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
    /// * `val` - pointer to the ReturnValue object that will be used to return the value
    /// # Returns
    /// `bool` - operation success status
    fn get_prop_val(&self, num: usize, val: ReturnValue) -> bool;

    /// Equivalent to `SetPropVal` from Native API interface and is used to set the value of the property
    /// with the given index
    /// # Arguments
    /// * `num` - index of the property
    /// * `val` - pointer to the ParamValue object that contains the value
    /// # Returns
    /// `bool` - operation success status
    fn set_prop_val(&mut self, num: usize, val: &ParamValue) -> bool;

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
    /// * `param_num` - index of the parameter
    /// * `value` - pointer to the ReturnValue object that will be used to return the value
    /// # Returns
    /// `bool` - operation success status
    fn get_param_def_value(
        &self,
        method_num: usize,
        param_num: usize,
        value: ReturnValue,
    ) -> bool;

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
    /// `bool` - operation success status
    fn call_as_proc(
        &mut self,
        method_num: usize,
        params: &mut [ParamValue],
    ) -> bool;

    /// Equivalent to `CallAsFunc` from Native API interface and is used to call method
    /// with the given index as a function, meaning that it returns a value
    /// # Arguments
    /// * `method_num` - index of method
    /// * `params` - slice of ParamValue objects that contain the parameters
    /// * `val` - pointer to the ReturnValue object that will be used to return the value
    /// # Returns
    /// `bool` - operation success status
    fn call_as_func(
        &mut self,
        method_num: usize,
        params: &mut [ParamValue],
        val: ReturnValue,
    ) -> bool;

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
