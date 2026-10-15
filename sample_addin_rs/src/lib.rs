use std::sync::Arc;

use native_api_1c::{
    native_api_1c_core::ffi::{
        connection::{Connection, MessageCode},
        string_utils::{from_os_string, os_string},
    },
    native_api_1c_core::interface::ParamValue,
    native_api_1c_macro::{extern_functions, AddIn},
};

#[derive(AddIn)]
pub struct SampleAddIn {
    /// connection with 1C, used for calling events
    /// Arc is used to allow multiple threads to access the connection
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    /// Property, readable and writable from 1C
    #[add_in_prop(ty = Int, name = "MyProp", name_ru = "МоеСвойство", readable, writable)]
    pub some_prop: i32,

    /// Property, readable from 1C but not writable
    #[add_in_prop(ty = Int, name = "ProtectedProp", name_ru = "ЗащищенноеСвойство", readable)]
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
    #[arg(ty = Int, default = 12)] // default value for the second argument
    #[returns(ty = Int, result)]
    pub my_function: fn(&Self, i32, i64) -> Result<i32, ()>,

    /// Function, taking no arguments and returning nothing
    #[add_in_func(name = "MyProcedure", name_ru = "МояПроцедура")]
    pub my_procedure: fn(&mut Self),

    /// Property, holding a value of any 1C type, its type included
    #[add_in_prop(ty = Any, name = "LastValue", name_ru = "ПоследнееЗначение", readable, writable)]
    pub last_value: ParamValue,

    /// Function, taking and returning a value of any 1C type
    /// In 1C it can be called as:
    /// ```bsl
    ///  CompObj.DescribeValue(10);        // "Number: 10"
    ///  CompObj.DescribeValue("text");    // "String: text"
    ///  CompObj.DescribeValue(Undefined); // "Undefined"
    /// ```
    /// Every call also stores the received value into the LastValue property
    #[add_in_func(name = "DescribeValue", name_ru = "ОписатьЗначение")]
    #[arg(ty = Any)]
    #[returns(ty = Any)]
    pub describe_value: fn(&mut Self, ParamValue) -> ParamValue,

    /// Puts a message into the 1C message window through `Connection::add_error`.
    /// This only reaches the user because `extern_functions!` exports
    /// `SetPlatformCapabilities`; without it the platform drops the message
    #[add_in_func(name = "ReportMessage", name_ru = "СообщитьВ1С")]
    #[arg(ty = Str)]
    #[returns(ty = Bool)]
    pub report_message: fn(&Self, String) -> bool,

    /// Panics on purpose, to check that the panic is caught at the FFI boundary
    /// and reported to 1C instead of aborting the process.
    /// Destructive: the component stays poisoned and refuses every later call,
    /// so this has to be the last thing you run
    #[add_in_func(name = "Panic", name_ru = "Паника")]
    pub panic_on_purpose: fn(&Self),

    /// Private field, not visible from 1C
    private_field: i32,
}

impl Default for SampleAddIn {
    fn default() -> Self {
        Self {
            connection: Arc::new(None),
            some_prop: 0,
            protected_prop: 50,
            my_function: Self::my_function_inner,
            my_procedure: Self::my_procedure_inner,
            last_value: ParamValue::Empty,
            describe_value: Self::describe_value_inner,
            report_message: Self::report_message_inner,
            panic_on_purpose: Self::panic_on_purpose_inner,
            private_field: 100,
        }
    }
}

impl SampleAddIn {
    fn my_function_inner(&self, arg: i32, arg_maybe_default: i64) -> Result<i32, ()> {
        Ok(self.protected_prop
            + self.some_prop
            + arg
            + self.private_field
            + arg_maybe_default as i32)
    }

    fn my_procedure_inner(&mut self) {
        self.protected_prop += 10;
    }

    fn describe_value_inner(&mut self, value: ParamValue) -> ParamValue {
        let description = match &value {
            ParamValue::Empty => "Undefined".to_owned(),
            ParamValue::Bool(v) => format!("Boolean: {v}"),
            ParamValue::I32(v) => format!("Number: {v}"),
            ParamValue::F64(v) => format!("Number: {v}"),
            ParamValue::Date(v) => format!("Date: {}", chrono::NaiveDateTime::from(v)),
            ParamValue::String(v) => format!("String: {}", from_os_string(v)),
            ParamValue::Blob(v) => format!("BinaryData: {} bytes", v.len()),
        };

        self.last_value = value;

        // `os_string`, not `os_string_nil`: `update_to_str` reports the slice length
        // to 1C as `wstrLen`, and a trailing terminator would become part of the string
        ParamValue::String(os_string(&description))
    }

    fn report_message_inner(&self, text: String) -> bool {
        let Some(connection) = *self.connection else {
            return false;
        };

        connection.add_error(MessageCode::Info, "SampleAddIn", &text)
    }

    fn panic_on_purpose_inner(&self) {
        panic!("deliberate panic from the sample add-in");
    }
}

extern_functions! {
    SampleAddIn::default(),
}
