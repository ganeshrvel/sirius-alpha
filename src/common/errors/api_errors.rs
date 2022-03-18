use std::fmt::{Display, Formatter};

use attohttpc::StatusCode;

// #[derive(Debug)]
// pub enum ApiErrors {
//     _Http(StatusCode, String),
//
//     InternalServerError(String),
//
//     BadRequest(String),
// }
//
// impl ApiErrors {
//     pub fn message(&self) -> String {
//         match self {
//             Self::InternalServerError(e) => format!(r"An Internal Server Error occured: {:?}", e),
//             Self::BadRequest(e) => format!(r"A Bad Request Error occured: {:?}", e),
//             Self::_Http(status_code, e) => {
//                 format!(r"A HTTP error occured: {:?} | {:?}", status_code, e)
//             }
//         }
//     }
// }
//
// impl Display for ApiErrors {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }
