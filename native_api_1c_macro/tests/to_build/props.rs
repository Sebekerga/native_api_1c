use std::sync::Arc;

use native_api_1c::native_api_1c_core::ffi::connection::Connection;
use native_api_1c_macro::AddIn;

#[derive(AddIn)]
pub struct MyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_prop(name = "prp_RW_str", name_ru = "свств_RW_str", readable, writable)]
    pub str_prop_rw: String,
    #[add_in_prop(name = "prp_R_str", name_ru = "свств_R_str", readable)]
    pub str_prop_r: String,
    #[add_in_prop(name = "prp_W_str", name_ru = "свств_W_str", writable)]
    pub str_prop_w: String,

    #[add_in_prop(name = "prp_RW_int", name_ru = "свств_RW_int", readable, writable)]
    pub int_prop_rw: i32,
    #[add_in_prop(name = "prp_R_int", name_ru = "свств_R_int", readable)]
    pub int_prop_r: i32,
    #[add_in_prop(name = "prp_W_int", name_ru = "свств_W_int", writable)]
    pub int_prop_w: i32,

    #[add_in_prop(name = "prp_RW_float", name_ru = "свств_RW_float", readable, writable)]
    pub float_prop_rw: f64,
    #[add_in_prop(name = "prp_R_float", name_ru = "свств_R_float", readable)]
    pub float_prop_r: f64,
    #[add_in_prop(name = "prp_W_float", name_ru = "свств_W_float", writable)]
    pub float_prop_w: f64,

    #[add_in_prop(name = "prp_RW_bool", name_ru = "свств_RW_bool", readable, writable)]
    pub bool_prop_rw: bool,
    #[add_in_prop(name = "prp_R_bool", name_ru = "свств_R_bool", readable)]
    pub bool_prop_r: bool,
    #[add_in_prop(name = "prp_W_bool", name_ru = "свств_W_bool", writable)]
    pub bool_prop_w: bool,
}

impl MyAddIn {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(None),
            str_prop_rw: String::new(),
            str_prop_r: String::new(),
            str_prop_w: String::new(),
            int_prop_rw: 0,
            int_prop_r: 0,
            int_prop_w: 0,
            float_prop_rw: 0.0,
            float_prop_r: 0.0,
            float_prop_w: 0.0,
            bool_prop_rw: false,
            bool_prop_r: false,
            bool_prop_w: false,
        }
    }
}

fn main() {
    let _add_in = MyAddIn::new();
}
