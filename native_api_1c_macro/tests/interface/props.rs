use std::{collections::HashSet, ptr::NonNull, sync::Arc};

use native_api_1c::native_api_1c_core::{
    ffi::{
        connection::Connection,
        memory_manager::{AllocationError, MemoryManagerImpl},
        provided_types::{ParamValue, ReturnValue, TVariant, VariantType},
        string_utils::{from_os_string, os_string_nil},
    },
    interface::AddInWrapper,
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

const NAME_PAIRS: &[(&str, &str)] = &[
    (RW_PROP_NAME, RW_PROP_NAME_RU),
    (R_PROP_NAME, R_PROP_NAME_RU),
    (W_PROP_NAME, W_PROP_NAME_RU),
];

const START_VALUE: i32 = 42;
const NEW_VALUE: i32 = 24;

const SETTABLE_VALUES: &[i32] = &[START_VALUE, NEW_VALUE, i32::max_value(), i32::min_value()];

static NON_SETTABLE_VALUES: &[ParamValue] = &[ParamValue::Bool(false), ParamValue::Empty];

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

struct DummyMemoryManager;
impl MemoryManagerImpl for DummyMemoryManager {
    fn alloc_blob(&self, _size: usize) -> Result<NonNull<u8>, AllocationError> {
        Ok(NonNull::dangling())
    }

    fn alloc_str(&self, _size: usize) -> Result<NonNull<u16>, AllocationError> {
        Ok(NonNull::dangling())
    }

    fn free_memory(&self, _ptr: &mut *mut std::ffi::c_void) {}
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
fn get_n_props(add_in: TestAddIn) {
    assert_eq!(add_in.get_n_props(), 3)
}

#[rstest]
fn find_prop(add_in: TestAddIn) {
    for (name, name_ru) in NAME_PAIRS {
        let name = os_string_nil(name);
        let name_ru = os_string_nil(name_ru);

        assert!(add_in.find_prop(&name).is_some());
        assert!(add_in.find_prop(&name_ru).is_some());

        assert!(add_in.find_prop(&name).is_some());
    }

    let invalid_name = os_string_nil(INVALID_PROP_NAME);
    assert!(add_in.find_prop(&invalid_name).is_none());
}

#[rstest]
fn get_prop_name(add_in: TestAddIn) {
    let mut prop_names = HashSet::new();
    for prop_i in 0..=2 {
        for alias in 0..=2 {
            assert!(add_in.get_prop_name(prop_i, alias).is_some());
        }

        let returned_name = from_os_string(&add_in.get_prop_name(prop_i, 0).unwrap());
        let returned_name_ru = from_os_string(&add_in.get_prop_name(prop_i, 1).unwrap());
        let returned_name_rand = from_os_string(&add_in.get_prop_name(prop_i, 42).unwrap());

        assert_eq!(returned_name_rand, returned_name_ru);

        prop_names.insert(returned_name);
        prop_names.insert(returned_name_ru);
        prop_names.insert(returned_name_rand);
    }
    assert_eq!(prop_names.len(), 6);

    assert!(add_in.get_prop_name(3, 0).is_none());
}

#[rstest]
fn is_prop_readable(add_in: TestAddIn) {
    let rw_prop_index = add_in.find_prop(&os_string_nil(RW_PROP_NAME)).unwrap();
    assert!(add_in.is_prop_readable(rw_prop_index));

    let r_prop_index = add_in.find_prop(&os_string_nil(R_PROP_NAME)).unwrap();
    assert!(add_in.is_prop_readable(r_prop_index));

    let w_prop_index = add_in.find_prop(&os_string_nil(W_PROP_NAME)).unwrap();
    assert!(!add_in.is_prop_readable(w_prop_index));
}

#[rstest]
fn is_prop_writable(add_in: TestAddIn) {
    let rw_prop_index = add_in.find_prop(&os_string_nil(RW_PROP_NAME)).unwrap();
    assert!(add_in.is_prop_writable(rw_prop_index));

    let r_prop_index = add_in.find_prop(&os_string_nil(R_PROP_NAME)).unwrap();
    assert!(!add_in.is_prop_writable(r_prop_index));

    let w_prop_index = add_in.find_prop(&os_string_nil(W_PROP_NAME)).unwrap();
    assert!(add_in.is_prop_writable(w_prop_index));
}

#[rstest]
fn get_prop_val(add_in: TestAddIn) {
    let mem = &mut DummyMemoryManager;
    let result = &mut false;

    let variant = &mut TVariant::default();
    let rw_prop_index = add_in.find_prop(&os_string_nil(RW_PROP_NAME)).unwrap();
    assert!(add_in.get_prop_val(rw_prop_index, ReturnValue::new(mem, variant, result)));
    assert_eq!(variant.vt, VariantType::Int32);
    unsafe {
        assert_eq!(variant.value.i32, START_VALUE);
    }

    let variant = &mut TVariant::default();
    let r_prop_index = add_in.find_prop(&os_string_nil(R_PROP_NAME)).unwrap();
    assert!(add_in.get_prop_val(r_prop_index, ReturnValue::new(mem, variant, result)));
    assert_eq!(variant.vt, VariantType::Int32);
    unsafe {
        assert_eq!(variant.value.i32, START_VALUE);
    }

    let variant = &mut TVariant::default();
    variant.value.i32 = NEW_VALUE;
    let w_prop_index = add_in.find_prop(&os_string_nil(W_PROP_NAME)).unwrap();
    assert!(!add_in.get_prop_val(w_prop_index, ReturnValue::new(mem, variant, result)));
    unsafe {
        assert_ne!(variant.value.i32, START_VALUE);
    }
}

#[rstest]
fn set_prop_val(mut add_in: TestAddIn) {
    let rw_prop_index = add_in.find_prop(&os_string_nil(RW_PROP_NAME)).unwrap();
    for value in SETTABLE_VALUES {
        assert!(add_in.set_prop_val(rw_prop_index, &ParamValue::I32(*value)));
        assert_eq!(add_in.rw_property, *value);
    }
    for value in NON_SETTABLE_VALUES {
        let value_before = add_in.rw_property;
        assert!(!add_in.set_prop_val(rw_prop_index, value));
        assert_eq!(add_in.rw_property, value_before);
    }

    let r_prop_index = add_in.find_prop(&os_string_nil(R_PROP_NAME)).unwrap();
    for value in SETTABLE_VALUES {
        let value_before = add_in.r_property;
        assert!(!add_in.set_prop_val(r_prop_index, &ParamValue::I32(*value)));
        assert_eq!(add_in.r_property, value_before);
    }
    for value in NON_SETTABLE_VALUES {
        let value_before = add_in.r_property;
        assert!(!add_in.set_prop_val(r_prop_index, value));
        assert_eq!(add_in.r_property, value_before);
    }

    let w_prop_index = add_in.find_prop(&os_string_nil(W_PROP_NAME)).unwrap();
    for value in SETTABLE_VALUES {
        assert!(add_in.set_prop_val(w_prop_index, &ParamValue::I32(*value)));
        assert_eq!(add_in.w_property, *value);
    }
    for value in NON_SETTABLE_VALUES {
        let value_before = add_in.w_property;
        assert!(!add_in.set_prop_val(w_prop_index, value));
        assert_eq!(add_in.w_property, value_before);
    }
}
