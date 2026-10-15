use std::sync::Arc;

use native_api_1c::native_api_1c_core::{
    ffi::{connection::Connection, provided_types::Tm, string_utils::os_string_nil},
    interface::{AddInWrapper, ParamValue, ParamValues},
};
use native_api_1c_macro::AddIn;
use rstest::{fixture, rstest};

const DEFAULT_VALUE: ParamValue = ParamValue::I32(42);

const PROP_NAME_EN: &str = "AnyProperty";
const PROP_NAME_RU: &str = "ЛюбоеСвойство";

const ECHO_NAME_EN: &str = "Echo";
const ECHO_NAME_RU: &str = "Эхо";

const SUM_NAME_EN: &str = "Sum";
const SUM_NAME_RU: &str = "Сумма";

const OUT_NAME_EN: &str = "OutProcedure";
const OUT_NAME_RU: &str = "ПроцедураВывода";

const INVALID_NAME: &str = "Invalid";

#[derive(AddIn)]
struct AnyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_prop(ty = Any, name = PROP_NAME_EN, name_ru = PROP_NAME_RU, readable, writable)]
    prop: ParamValue,

    #[add_in_func(name = ECHO_NAME_EN, name_ru = ECHO_NAME_RU)]
    #[arg(ty = Any)]
    #[returns(ty = Any)]
    echo: fn(&Self, ParamValue) -> ParamValue,

    #[add_in_func(name = SUM_NAME_EN, name_ru = SUM_NAME_RU)]
    #[arg(ty = Any)]
    #[arg(ty = Any, default = DEFAULT_VALUE)]
    #[returns(ty = Any, result)]
    sum: fn(&Self, ParamValue, ParamValue) -> Result<ParamValue, ()>,

    #[add_in_func(name = OUT_NAME_EN, name_ru = OUT_NAME_RU)]
    #[arg(ty = Any, as_out)]
    out_procedure: fn(&mut ParamValue),
}

#[fixture]
fn add_in() -> AnyAddIn {
    AnyAddIn {
        connection: Arc::new(None),
        prop: ParamValue::Empty,
        echo: |_addin, value| value,
        sum: |_addin, lhs, rhs| match (lhs, rhs) {
            (ParamValue::I32(lhs), ParamValue::I32(rhs)) => Ok(ParamValue::I32(lhs + rhs)),
            (ParamValue::F64(lhs), ParamValue::F64(rhs)) => Ok(ParamValue::F64(lhs + rhs)),
            _ => Err(()),
        },
        out_procedure: |value| *value = ParamValue::Bool(true),
    }
}

#[rstest]
fn test_get_n_props(add_in: AnyAddIn) {
    assert_eq!(add_in.get_n_props(), 1)
}

#[rstest]
#[case(PROP_NAME_EN, Some(0))]
#[case(PROP_NAME_RU, Some(0))]
#[case(INVALID_NAME, None)]
fn test_find_prop(add_in: AnyAddIn, #[case] name: &str, #[case] expected: Option<usize>) {
    assert_eq!(add_in.find_prop(&os_string_nil(name)), expected);
}

#[rstest]
#[case(0, 0, Some(PROP_NAME_EN))]
#[case(0, 1, Some(PROP_NAME_RU))]
#[case(0, 42, Some(PROP_NAME_RU))]
#[case(1, 0, None)]
fn test_get_prop_name(
    add_in: AnyAddIn,
    #[case] prop_index: usize,
    #[case] alias_index: usize,
    #[case] expected: Option<&str>,
) {
    assert_eq!(
        add_in.get_prop_name(prop_index, alias_index),
        expected.map(os_string_nil)
    );
}

#[rstest]
#[case(0, true)]
#[case(1, false)]
fn test_is_prop_readable(add_in: AnyAddIn, #[case] prop_index: usize, #[case] expected: bool) {
    assert_eq!(add_in.is_prop_readable(prop_index), expected);
}

#[rstest]
#[case(0, true)]
#[case(1, false)]
fn test_is_prop_writable(add_in: AnyAddIn, #[case] prop_index: usize, #[case] expected: bool) {
    assert_eq!(add_in.is_prop_writable(prop_index), expected);
}

/// An `Any` property stores whatever the platform sent, keeping the type intact
#[rstest]
#[case(ParamValue::Empty)]
#[case(ParamValue::Bool(true))]
#[case(ParamValue::I32(-7))]
#[case(ParamValue::F64(1.5))]
#[case(ParamValue::Date(Tm::default()))]
#[case(ParamValue::String(os_string_nil("строка")))]
#[case(ParamValue::Blob(vec![0xDE, 0xAD, 0xBE, 0xEF]))]
fn test_prop_keeps_type_on_roundtrip(mut add_in: AnyAddIn, #[case] value: ParamValue) {
    assert_eq!(add_in.set_prop_val(0, value.clone()), Ok(()));
    assert_eq!(add_in.get_prop_val(0), Ok(value));
}

#[rstest]
fn test_get_n_methods(add_in: AnyAddIn) {
    assert_eq!(add_in.get_n_methods(), 3)
}

#[rstest]
#[case(ECHO_NAME_EN, Some(0))]
#[case(ECHO_NAME_RU, Some(0))]
#[case(SUM_NAME_EN, Some(1))]
#[case(SUM_NAME_RU, Some(1))]
#[case(OUT_NAME_EN, Some(2))]
#[case(OUT_NAME_RU, Some(2))]
#[case(INVALID_NAME, None)]
fn test_find_method(add_in: AnyAddIn, #[case] name: &str, #[case] expected: Option<usize>) {
    assert_eq!(add_in.find_method(&os_string_nil(name)), expected);
}

#[rstest]
#[case(0, 0, Some(ECHO_NAME_EN))]
#[case(0, 1, Some(ECHO_NAME_RU))]
#[case(0, 42, Some(ECHO_NAME_RU))]
#[case(1, 0, Some(SUM_NAME_EN))]
#[case(1, 1, Some(SUM_NAME_RU))]
#[case(2, 0, Some(OUT_NAME_EN))]
#[case(2, 1, Some(OUT_NAME_RU))]
#[case(3, 0, None)]
fn test_get_method_name(
    add_in: AnyAddIn,
    #[case] method_index: usize,
    #[case] alias_index: usize,
    #[case] expected: Option<&str>,
) {
    assert_eq!(
        add_in.get_method_name(method_index, alias_index),
        expected.map(os_string_nil)
    );
}

#[rstest]
#[case(0, 1)]
#[case(1, 2)]
#[case(2, 1)]
#[case(3, 0)]
fn test_get_n_params(add_in: AnyAddIn, #[case] method_index: usize, #[case] expected: usize) {
    assert_eq!(add_in.get_n_params(method_index), expected);
}

#[rstest]
#[case(0, 0, None)]
#[case(1, 0, None)]
#[case(1, 1, Some(DEFAULT_VALUE))]
#[case(1, 2, None)]
#[case(2, 0, None)]
#[case(3, 0, None)]
fn test_get_param_def_value(
    add_in: AnyAddIn,
    #[case] method_index: usize,
    #[case] param_index: usize,
    #[case] expected: Option<ParamValue>,
) {
    assert_eq!(
        add_in.get_param_def_value(method_index, param_index),
        expected
    );
}

#[rstest]
#[case(0, true)]
#[case(1, true)]
#[case(2, false)]
#[case(3, false)]
fn test_has_ret_val(add_in: AnyAddIn, #[case] method_index: usize, #[case] expected: bool) {
    assert_eq!(add_in.has_ret_val(method_index), expected);
}

/// An `Any` argument reaches the function with its original type and comes back unchanged
#[rstest]
#[case(ParamValue::Empty)]
#[case(ParamValue::Bool(false))]
#[case(ParamValue::I32(42))]
#[case(ParamValue::F64(3.25))]
#[case(ParamValue::Date(Tm::default()))]
#[case(ParamValue::String(os_string_nil("строка")))]
#[case(ParamValue::Blob(vec![1, 2, 3]))]
fn test_echo_returns_value_unchanged(mut add_in: AnyAddIn, #[case] value: ParamValue) {
    let mut params = ParamValues::new(vec![value.clone()]);

    assert_eq!(add_in.call_as_func(0, &mut params), Ok(value));
}

#[rstest]
fn test_sum_of_integers(mut add_in: AnyAddIn) {
    let mut params = ParamValues::new(vec![ParamValue::I32(2), ParamValue::I32(3)]);

    assert_eq!(add_in.call_as_func(1, &mut params), Ok(ParamValue::I32(5)));
}

#[rstest]
fn test_sum_of_floats(mut add_in: AnyAddIn) {
    let mut params = ParamValues::new(vec![ParamValue::F64(0.5), ParamValue::F64(0.25)]);

    assert_eq!(
        add_in.call_as_func(1, &mut params),
        Ok(ParamValue::F64(0.75))
    );
}

#[rstest]
fn test_sum_of_mismatched_types_fails(mut add_in: AnyAddIn) {
    let mut params = ParamValues::new(vec![
        ParamValue::I32(2),
        ParamValue::String(os_string_nil("строка")),
    ]);

    assert_eq!(add_in.call_as_func(1, &mut params), Err(()));
}

#[rstest]
fn test_out_procedure_rewrites_param(mut add_in: AnyAddIn) {
    let mut params = ParamValues::new(vec![ParamValue::I32(1)]);

    assert_eq!(add_in.call_as_proc(2, &mut params), Ok(()));
    assert_eq!(params[0], ParamValue::Bool(true));
}

#[rstest]
fn test_out_procedure_returns_no_value(mut add_in: AnyAddIn) {
    let mut params = ParamValues::new(vec![ParamValue::Empty]);

    assert!(add_in.call_as_func(2, &mut params).is_err());
}
