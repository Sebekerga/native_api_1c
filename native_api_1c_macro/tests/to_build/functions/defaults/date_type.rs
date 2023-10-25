use std::sync::Arc;

use native_api_1c::native_api_1c_core::ffi::connection::Connection;
use native_api_1c_macro::AddIn;

#[derive(AddIn)]
pub struct MyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_func(name = "MyFunction", name_ru = "МояФункция")]
    #[arg(Date, default = "2021-01-01T00:00:00+00:00")]
    #[returns(Date)]
    pub my_function:
        fn(&Self, chrono::DateTime<chrono::FixedOffset>) -> chrono::DateTime<chrono::FixedOffset>,
}

impl MyAddIn {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(None),
            my_function: Self::my_function_inner,
        }
    }

    fn my_function_inner(
        &self,
        arg: chrono::DateTime<chrono::FixedOffset>,
    ) -> chrono::DateTime<chrono::FixedOffset> {
        arg
    }
}

fn main() {
    let _add_in = MyAddIn::new();
}
