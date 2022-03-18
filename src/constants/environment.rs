pub struct Environment;

impl Environment {
    pub const IS_DEBUG: bool = cfg!(debug_assertions);
    pub const IS_RELEASE: bool = cfg!(not(debug_assertions));
}
