use std::marker::PhantomData;

#[derive(Debug)]
pub struct DefaultValues<'a> {
    /// https://stackoverflow.com/questions/40484154/parameter-a-is-never-used-error-when-a-is-used-in-type-parameter-bound
    // Causes the type to function *as though* it has a `&'a ()` field,
    // despite not *actually* having one.
    _marker: PhantomData<&'a ()>,
}

impl DefaultValues<'static> {
    pub const API_TIMEOUT_MS: u64 = 10000;
    pub const WIFI_RECONNECTION_DELAY_MS: u64 = 4000;
    pub const NET_CONNECTION_MANAGER_THREAD_DELAY_MS: u64 = 4000;
    pub const TM1637_THREAD: u64 = 1000;
    pub const APIS_THREAD_DELAY_MS: u64 = 3000; //todo change to 30000
    pub const BUZZER_THREAD_DELAY_MS: u64 = 500;
}
