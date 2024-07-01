use std::sync::Arc;

use native_api_1c::native_api_1c_core::{
    ffi::connection::Connection,
    interface::{AddInWrapper, ParamValue},
};
use native_api_1c_macro::AddIn;
use rstest::{fixture, rstest};

const RW_PROP_NAME: &str = "Property";
const RW_PROP_NAME_RU: &str = "Свойство";

const R_PROP_NAME: &str = "ReadOnlyProperty";
const R_PROP_NAME_RU: &str = "СвойствоТолькоЧтение";

const W_PROP_NAME: &str = "WriteOnlyProperty";
const W_PROP_NAME_RU: &str = "СвойствоТолькоЗапись";

const INVALID_PROP_NAME: &str = "InvalidProperty";

const START_VALUE: i32 = 42;
const NEW_VALUE: i32 = 24;

#[derive(AddIn)]
struct TestAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_prop(ty = Int, name = RW_PROP_NAME, name_ru = RW_PROP_NAME_RU, readable, writable)]
    rw_property: i32,

    #[add_in_prop(ty = Int, name = R_PROP_NAME, name_ru = R_PROP_NAME_RU, readable)]
    r_property: i32,

    #[add_in_prop(ty = Int, name = W_PROP_NAME, name_ru = W_PROP_NAME_RU, writable)]
    w_property: i32,
}

#[fixture]
fn add_in() -> TestAddIn {
    TestAddIn {
        connection: Arc::new(None),
        rw_property: START_VALUE,
        r_property: START_VALUE,
        w_property: START_VALUE,
    }
}

#[rstest]
fn test_get_n_props(add_in: TestAddIn) {
    assert_eq!(add_in.get_n_props(), 3)
}

#[rstest]
#[case(RW_PROP_NAME, Some(0))]
#[case(RW_PROP_NAME_RU, Some(0))]
#[case(R_PROP_NAME, Some(1))]
#[case(R_PROP_NAME_RU, Some(1))]
#[case(W_PROP_NAME, Some(2))]
#[case(W_PROP_NAME_RU, Some(2))]
#[case(INVALID_PROP_NAME, None)]
fn test_find_prop(add_in: TestAddIn, #[case] prop_name: &str, #[case] prop_index: Option<usize>) {
    use native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil;

    assert_eq!(add_in.find_prop(&os_string_nil(prop_name)), prop_index);
}

#[rstest]
#[case(0, 0, Some(RW_PROP_NAME))]
#[case(0, 1, Some(RW_PROP_NAME_RU))]
#[case(0, 43, Some(RW_PROP_NAME_RU))]
#[case(1, 0, Some(R_PROP_NAME))]
#[case(1, 1, Some(R_PROP_NAME_RU))]
#[case(1, 43, Some(R_PROP_NAME_RU))]
#[case(2, 0, Some(W_PROP_NAME))]
#[case(2, 1, Some(W_PROP_NAME_RU))]
#[case(2, 43, Some(W_PROP_NAME_RU))]
fn test_get_prop_name(
    add_in: TestAddIn,
    #[case] prop_index: usize,
    #[case] name_index: usize,
    #[case] name: Option<&str>,
) {
    use native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil;

    let prop_name = add_in.get_prop_name(prop_index, name_index);
    assert_eq!(prop_name, name.map(os_string_nil));
}

#[rstest]
#[case(0, true)]
#[case(1, true)]
#[case(2, false)]
fn test_is_prop_readable(add_in: TestAddIn, #[case] prop_index: usize, #[case] readable: bool) {
    assert_eq!(add_in.is_prop_readable(prop_index), readable);
}

#[rstest]
#[case(0, true)]
#[case(1, false)]
#[case(2, true)]
fn test_is_prop_writable(add_in: TestAddIn, #[case] prop_index: usize, #[case] writable: bool) {
    assert_eq!(add_in.is_prop_writable(prop_index), writable);
}

#[rstest]
#[case(0, Some(START_VALUE))]
#[case(1, Some(START_VALUE))]
#[case(2, None)]
fn test_get_prop_val(
    add_in: TestAddIn,
    #[case] prop_i: usize,
    #[case] expected_value: Option<i32>,
) {
    let prop_value = add_in.get_prop_val(prop_i);
    match expected_value {
        Some(value) => {
            assert!(prop_value.is_ok());
            assert_eq!(prop_value.unwrap(), ParamValue::I32(value));
        }
        None => assert!(prop_value.is_err()),
    }
}

#[rstest]
#[case(0, Ok(()), |add_in: &TestAddIn| add_in.rw_property, NEW_VALUE)]
#[case(1, Err(()), |add_in: &TestAddIn| add_in.r_property, START_VALUE)]
#[case(2, Ok(()), |add_in: &TestAddIn| add_in.w_property, NEW_VALUE)]
fn test_set_prop_val(
    mut add_in: TestAddIn,
    #[case] prop_i: usize,
    #[case] expected_result: Result<(), ()>,
    #[case] value_getter: fn(&TestAddIn) -> i32,
    #[case] new_value: i32,
) {
    assert_eq!(
        add_in.set_prop_val(prop_i, ParamValue::I32(NEW_VALUE)),
        expected_result
    );
    assert_eq!(value_getter(&add_in), new_value);
}
