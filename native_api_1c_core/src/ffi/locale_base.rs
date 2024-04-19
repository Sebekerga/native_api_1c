use crate::interface::AddInWrapper;

use super::string_utils::get_str;

type This<T> = super::This<{ super::offset::LOCALE }, T>;

#[repr(C)]
pub struct LocaleBaseVTable<T: AddInWrapper> {
    dtor: usize,
    #[cfg(target_family = "unix")]
    dtor2: usize,
    set_locale: unsafe extern "system" fn(&mut This<T>, *const u16),
}

impl<T: AddInWrapper> Default for LocaleBaseVTable<T> {
    fn default() -> Self {
        Self {
            dtor: 0,
            #[cfg(target_family = "unix")]
            dtor2: 0,
            set_locale,
        }
    }
}

unsafe extern "system" fn set_locale<T: AddInWrapper>(
    this: &mut This<T>,
    loc: *const u16,
) {
    let component = this.get_component();
    let loc = get_str(loc);
    component.addin.set_locale(loc)
}
