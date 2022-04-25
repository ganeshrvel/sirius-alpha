use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiClientError {
    #[error("[0:?] an API client error occured: {1:?}")]
    Response(String, String),

    #[error("[0:?] an Json Parsing error occured: {1:?}")]
    JsonParsing(String, String),
}

#[derive(Error, Debug)]
pub enum ApiResponseError {
    #[error("[0:?] a site not found error occured. {1:?}")]
    SiteNotFound(String, String),

    #[error("[0:?] an Internal Server error occured: Error: {1:?}, Message: {2:?}")]
    InternalServerError(String, String, String),

    #[error("[0:?] a Bad Request error occured. Error: {1:?}, Message: {2:?}")]
    BadRequest(String, String, String),

    #[error("[0:?] a 404 error occured. Error: {1:?}, Message: {2:?}")]
    NotFound(String, String, String),
}
