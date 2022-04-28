use std::marker::PhantomData;

#[non_exhaustive]
#[derive(Debug)]
pub struct HeaderKeys<'a> {
    /// https://stackoverflow.com/questions/40484154/parameter-a-is-never-used-error-when-a-is-used-in-type-parameter-bound
    // Causes the type to function *as though* it has a `&'a ()` field,
    // despite not *actually* having one.
    _marker: PhantomData<&'a ()>,
}

impl HeaderKeys<'static> {
    pub const DEVICE_ID: &'static str = "x-device-id";
    pub const CONTENT_TYPE: &'static str = "Content-Type";
    pub const PERMISSIONS_POLICY: &'static str = "Permissions-Policy";
    pub const AUTHORIZATION: &'static str = "Authorization";
    pub const ACCEPT_ENCODING: &'static str = "Accept-Encoding";
}

#[non_exhaustive]
#[derive(Debug)]
pub struct HeaderValues<'a> {
    /// https://stackoverflow.com/questions/40484154/parameter-a-is-never-used-error-when-a-is-used-in-type-parameter-bound
    // Causes the type to function *as though* it has a `&'a ()` field,
    // despite not *actually* having one.
    _marker: PhantomData<&'a ()>,
}

impl HeaderValues<'static> {
    pub const APPLICATION_JSON: &'static str = "application/json";
}
