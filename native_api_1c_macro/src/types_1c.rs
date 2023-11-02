use crate::constants::{BLOB_TYPE, BOOL_TYPE, DATE_TYPE, F64_TYPE, I32_TYPE, STRING_TYPE};

#[derive(Clone)]
pub enum ParamType {
    Bool,
    I32,
    F64,
    String,
    Date,
    Blob,
    SelfType,
}

impl TryFrom<&String> for ParamType {
    type Error = ();

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        match value.as_str() {
            BOOL_TYPE => Ok(ParamType::Bool),
            I32_TYPE => Ok(ParamType::I32),
            F64_TYPE => Ok(ParamType::F64),
            STRING_TYPE => Ok(ParamType::String),
            DATE_TYPE => Ok(ParamType::Date),
            BLOB_TYPE => Ok(ParamType::Blob),
            _ => Err(()),
        }
    }
}
