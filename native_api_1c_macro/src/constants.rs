pub const BOOL_TYPE: &str = "Bool";
pub const I32_TYPE: &str = "Int";
pub const F64_TYPE: &str = "Float";
pub const STRING_TYPE: &str = "Str";
pub const DATE_TYPE: &str = "Date";
pub const BLOB_TYPE: &str = "Blob";
pub const UNTYPED_TYPE: &str = "None";

pub const IN_PARAMETER_FLAG: &str = "as_in";
pub const OUT_PARAMETER_FLAG: &str = "as_out";

pub const ALL_RETURN_TYPES: &[&str] = &[
    BOOL_TYPE,
    I32_TYPE,
    F64_TYPE,
    STRING_TYPE,
    DATE_TYPE,
    BLOB_TYPE,
    UNTYPED_TYPE,
];
pub const ALL_ARG_TYPES: &[&str] = &[
    BOOL_TYPE,
    I32_TYPE,
    F64_TYPE,
    STRING_TYPE,
    DATE_TYPE,
    BLOB_TYPE,
];

pub const NAME_ATTR: &str = "name";
pub const NAME_RU_ATTR: &str = "name_ru";

pub const READABLE_ATTR: &str = "readable";
pub const WRITABLE_ATTR: &str = "writable";

pub const ARG_ATTR: &str = "arg";
pub const DEFAULT_ATTR: &str = "default";

pub const RETURNS_ATTR: &str = "returns";
pub const RESULT_ATTR: &str = "result";
