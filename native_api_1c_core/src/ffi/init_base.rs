use std::ffi::c_long;

use super::{connection::Connection, memory_manager::MemoryManager, This};
use crate::interface::AddInWrapper;

#[repr(C)]
pub struct InitDoneBaseVTable<T: AddInWrapper> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    init:
        unsafe extern "system" fn(&mut This<0, T>, &'static Connection) -> bool,
    set_mem_manager: unsafe extern "system" fn(
        &mut This<0, T>,
        &'static MemoryManager,
    ) -> bool,
    get_info: unsafe extern "system" fn(&mut This<0, T>) -> c_long,
    done: unsafe extern "system" fn(&mut This<0, T>),
}

unsafe extern "system" fn init<T: AddInWrapper>(
    this: &mut This<0, T>,
    interface: &'static Connection,
) -> bool {
    let component = this.get_component();
    component.addin.init(interface)
}

unsafe extern "system" fn set_mem_manager<T: AddInWrapper>(
    this: &mut This<0, T>,
    mem: &'static MemoryManager,
) -> bool {
    let component = this.get_component();
    component.memory = Some(mem);
    true
}

unsafe extern "system" fn get_info<T: AddInWrapper>(
    this: &mut This<0, T>,
) -> c_long {
    let component = this.get_component();
    component.addin.get_info() as c_long
}

unsafe extern "system" fn done<T: AddInWrapper>(this: &mut This<0, T>) {
    let component = this.get_component();
    component.addin.done()
}

impl<T: AddInWrapper> Default for InitDoneBaseVTable<T> {
    fn default() -> Self {
        Self {
            dtor: 0,
            #[cfg(target_family = "unix")]
            dtor2: 0,
            init,
            set_mem_manager,
            get_info,
            done,
        }
    }
}
