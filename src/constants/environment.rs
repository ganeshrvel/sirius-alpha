use lazy_static::lazy_static;

lazy_static! {
    pub static ref APP_ENV: Environment = Environment::new();
}

pub struct Environment {
    pub is_debug: bool,
    pub is_release: bool,
    pub config: EnvConfig,
}

impl Environment {
    pub const fn new() -> Self {
        const IS_DEBUG: bool = cfg!(debug_assertions);
        const IS_RELEASE: bool = cfg!(not(debug_assertions));

        let config: EnvConfig = EnvConfig::new(IS_DEBUG);

        Self {
            is_debug: IS_DEBUG,
            is_release: IS_RELEASE,
            config,
        }
    }
}

pub struct EnvConfig {
    pub show_network_requests: bool,
    pub show_network_response: bool,
}

impl EnvConfig {
    pub const fn new(is_debug: bool) -> Self {
        if is_debug {
            return Self {
                show_network_requests: true,
                show_network_response: true,
            };
        }

        Self {
            show_network_requests: false,
            show_network_response: false,
        }
    }
}
