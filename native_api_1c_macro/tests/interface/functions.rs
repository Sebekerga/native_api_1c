use std::sync::Arc;

use native_api_1c::native_api_1c_core::{
    ffi::{
        connection::Connection,
        string_utils::{os_string, os_string_nil},
    },
    interface::{AddInWrapper, ParamValue, ParamValues},
};
use native_api_1c_macro::AddIn;
use rstest::{fixture, rstest};

const DEFAULT_VALUE: i32 = 12;
const OUT_STR: &str = "world";

const FUNCTION_NAME_EN: &str = "Function";
const FUNCTION_NAME_RU: &str = "Функция";

const PROCEDURE_NAME_EN: &str = "Procedure";
const PROCEDURE_NAME_RU: &str = "Процедура";

const OUT_FUNCTION_NAME_EN: &str = "OutFunction";
const OUT_FUNCTION_NAME_RU: &str = "ВыводФункция";

const INVALID_NAME: &str = "Invalid";

#[derive(AddIn)]
struct TestAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    pub storage: i32,

    #[add_in_func(name = FUNCTION_NAME_EN, name_ru = FUNCTION_NAME_RU)]
    #[arg(ty = Int)]
    #[arg(ty = Int, default = DEFAULT_VALUE)]
    #[returns(ty = Int, result)]
    pub function: fn(&Self, i32, i32) -> Result<i32, ()>,

    #[add_in_func(name = PROCEDURE_NAME_EN, name_ru = PROCEDURE_NAME_RU)]
    #[arg(ty = Int)]
    #[arg(ty = Int, default = DEFAULT_VALUE)]
    pub procedure: fn(&mut Self, i32, i32),

    #[add_in_func(name = OUT_FUNCTION_NAME_EN, name_ru = OUT_FUNCTION_NAME_RU)]
    #[arg(ty = Str, as_out, default = OUT_STR)]
    pub out_function: fn(&mut String),
}

#[fixture]
fn add_in() -> TestAddIn {
    TestAddIn {
        connection: Arc::new(None),
        storage: 0,
        function: |addin, a, b| Ok(a + b + addin.storage),
        procedure: |addin, a, b| {
            addin.storage = a + b;
        },
        out_function: |out_str| {
            *out_str = format!("Hello, {out_str}!");
        },
    }
}

#[rstest]
fn test_get_n_methods(add_in: TestAddIn) {
    assert_eq!(add_in.get_n_methods(), 3)
}

#[rstest]
#[case(FUNCTION_NAME_EN, Some(0))]
#[case(FUNCTION_NAME_RU, Some(0))]
#[case(PROCEDURE_NAME_EN, Some(1))]
#[case(PROCEDURE_NAME_RU, Some(1))]
#[case(OUT_FUNCTION_NAME_EN, Some(2))]
#[case(OUT_FUNCTION_NAME_RU, Some(2))]
#[case(INVALID_NAME, None)]
fn test_find_method(add_in: TestAddIn, #[case] name: &str, #[case] expected: Option<usize>) {
    use native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil;

    assert_eq!(add_in.find_method(&os_string_nil(name)), expected);
}

#[rstest]
#[case(0, 0, Some(FUNCTION_NAME_EN))]
#[case(0, 1, Some(FUNCTION_NAME_RU))]
#[case(0, 42, Some(FUNCTION_NAME_RU))]
#[case(1, 0, Some(PROCEDURE_NAME_EN))]
#[case(1, 1, Some(PROCEDURE_NAME_RU))]
#[case(1, 42, Some(PROCEDURE_NAME_RU))]
#[case(2, 0, Some(OUT_FUNCTION_NAME_EN))]
#[case(2, 1, Some(OUT_FUNCTION_NAME_RU))]
#[case(2, 42, Some(OUT_FUNCTION_NAME_RU))]
#[case(3, 0, None)]
fn test_get_method_name(
    add_in: TestAddIn,
    #[case] method_i: usize,
    #[case] alias_i: usize,
    #[case] expected: Option<&str>,
) {
    use native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil;

    assert_eq!(
        add_in.get_method_name(method_i, alias_i),
        expected.map(os_string_nil)
    );
}

#[rstest]
#[case(0, 2)]
#[case(1, 2)]
#[case(2, 1)]
#[case(3, 0)]
fn test_get_n_params(add_in: TestAddIn, #[case] method_i: usize, #[case] n_params: usize) {
    assert_eq!(add_in.get_n_params(method_i), n_params);
}

#[rstest]
#[case(0, 0, None)]
#[case(0, 1, Some(ParamValue::I32(DEFAULT_VALUE)))]
#[case(0, 42, None)]
#[case(1, 0, None)]
#[case(1, 1, Some(ParamValue::I32(DEFAULT_VALUE)))]
#[case(1, 42, None)]
#[case(2, 0, Some(ParamValue::String(os_string_nil(OUT_STR))))]
#[case(2, 42, None)]
#[case(3, 0, None)]
fn test_get_param_def_value(
    add_in: TestAddIn,
    #[case] method_i: usize,
    #[case] param_i: usize,
    #[case] expected: Option<ParamValue>,
) {
    assert_eq!(add_in.get_param_def_value(method_i, param_i), expected);
}

#[rstest]
#[case(0, true)]
#[case(1, false)]
#[case(2, false)]
#[case(3, false)]
fn test_has_ret_val(add_in: TestAddIn, #[case] method_i: usize, #[case] has_ret_val: bool) {
    assert_eq!(add_in.has_ret_val(method_i), has_ret_val);
}

#[rstest]
fn test_call_function(mut add_in: TestAddIn) {
    let a = ParamValue::I32(1);
    let b = ParamValue::I32(2);
    let mut params = ParamValues::new(vec![a, b]);

    let result = add_in.call_as_func(0, &mut params);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ParamValue::I32(1 + 2 + add_in.storage));

    let result = add_in.call_as_proc(0, &mut params);
    assert!(result.is_ok());
}

#[rstest]
fn test_call_procedure(mut add_in: TestAddIn) {
    let a = ParamValue::I32(1);
    let b = ParamValue::I32(2);
    let mut params = ParamValues::new(vec![a, b]);

    let result = add_in.call_as_func(1, &mut params);
    assert!(result.is_err());

    let result = add_in.call_as_proc(1, &mut params);
    assert!(result.is_ok());
    assert_eq!(add_in.storage, 1 + 2);
}

#[rstest]
fn test_call_out_function(mut add_in: TestAddIn) {
    let out_str = os_string("1C");
    let mut params = ParamValues::new(vec![ParamValue::String(out_str)]);

    let result = add_in.call_as_func(2, &mut params);
    assert!(result.is_err());

    let result = add_in.call_as_proc(2, &mut params);
    assert!(result.is_ok());
    assert_eq!(params[0], ParamValue::String(os_string("Hello, 1C!")));
}
