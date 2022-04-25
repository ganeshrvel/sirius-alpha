use std::marker::PhantomData;

#[derive(Debug)]
pub struct SegmentDisplayText<'a> {
    /// https://stackoverflow.com/questions/40484154/parameter-a-is-never-used-error-when-a-is-used-in-type-parameter-bound
    // Causes the type to function *as though* it has a `&'a ()` field,
    // despite not *actually* having one.
    _marker: PhantomData<&'a ()>,
}

impl SegmentDisplayText<'static> {
    pub const ERR_404: &'static str = "Err 404";
    pub const ERR_400: &'static str = "Err 400";
    pub const ERR_503: &'static str = "Err 503";
    pub const ERR_API: &'static str = "Err API";
    pub const ERR_JSON: &'static str = "Err JSON";
    pub const ERR_NO_WIFI: &'static str = "Err no yify";
}
