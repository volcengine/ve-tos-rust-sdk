/*
 * Copyright (c) 2025 Beijing Volcano Engine Technology Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use serde::Deserialize;

use crate::common::RequestInfo;

pub type CommonError = std::io::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum GenericError {
    UrlParseError(url::ParseError),
    DateTimeParseError(chrono::ParseError),
    HttpRequestError(String),
    IoError(String),
    JsonError(String),
    DefaultError(String),
}

impl Error for GenericError {}

impl Display for GenericError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericError::UrlParseError(e) => {
                write!(f, "{}", e.to_string())
            }
            GenericError::DateTimeParseError(e) => {
                write!(f, "{}", e.to_string())
            }
            GenericError::DefaultError(e) => {
                write!(f, "{}", e)
            }
            GenericError::HttpRequestError(e) => {
                write!(f, "{}", e)
            }
            GenericError::IoError(e) => {
                write!(f, "{}", e)
            }
            GenericError::JsonError(e) => {
                write!(f, "{}", e)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TosError {
    TosClientError {
        message: String,
        cause: Option<GenericError>,
        request_url: String,
    },
    TosServerError {
        message: String,
        code: String,
        host_id: String,
        resource: String,
        status_code: isize,
        request_id: String,
        ec: String,
        key: String,
        id2: String,
        header: HashMap<String, String>,
        request_url: String,
    },
}

impl Error for TosError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TosError::TosClientError { cause, .. } => {
                if let Some(x) = cause {
                    return Some(x);
                }
                None
            }
            TosError::TosServerError { .. } => None,
        }
    }
}

impl Display for TosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error())
    }
}

impl TosError {
    pub fn message(&self) -> &str {
        match self {
            Self::TosClientError { message, .. } => message,
            Self::TosServerError { message, .. } => message,
        }
    }

    pub fn request_url(&self) -> &str {
        match self {
            Self::TosClientError { request_url, .. } => request_url,
            Self::TosServerError { request_url, .. } => request_url,
        }
    }

    pub fn is_server_error(&self) -> bool {
        match self {
            Self::TosClientError { .. } => false,
            Self::TosServerError { .. } => true,
        }
    }

    pub fn as_server_error(&self) -> Option<TosServerError> {
        match self {
            Self::TosClientError { .. } => None,
            Self::TosServerError {
                message, code, host_id,
                resource, status_code, request_id,
                ec, id2, header, request_url, key,
            } => Some(TosServerError {
                message,
                request_url,
                code,
                host_id,
                resource,
                status_code: status_code.to_owned(),
                request_id,
                ec,
                key,
                id2,
                header,
            }),
        }
    }

    pub fn error(&self) -> String {
        match self {
            Self::TosClientError { message, cause, .. } => {
                if let Some(e) = cause {
                    format!("message: {}, cause: {}", message, e)
                } else {
                    format!("message: {}", message)
                }
            }
            Self::TosServerError { status_code, code, ec, request_id, message, .. } => {
                if ec == "" {
                    format!("status_code: {}, code: {}, request_id: {}, message: {}", status_code, code, request_id, message)
                } else {
                    format!("status_code: {}, code: {}, request_id: {}, ec: {}, message: {}", status_code, code, request_id, ec, message)
                }
            }
        }
    }

    pub fn as_request_info(&self) -> Option<RequestInfo> {
        match self {
            Self::TosClientError { .. } => None::<RequestInfo>,
            Self::TosServerError { request_id, id2, status_code, header, .. } => Some(RequestInfo {
                request_id: request_id.to_string(),
                id2: id2.to_string(),
                status_code: status_code.to_owned(),
                header: header.to_owned(),
            }),
        }
    }

    pub(crate) fn set_request_url(&mut self, url: impl Into<String>) {
        match self {
            TosError::TosClientError { request_url, .. } => {
                *request_url = url.into();
            }
            TosError::TosServerError { request_url, .. } => {
                *request_url = url.into();
            }
        }
    }

    pub(crate) fn client_error(message: impl Into<String>) -> Self {
        Self::TosClientError {
            message: message.into(),
            request_url: "".to_string(),
            cause: None,
        }
    }

    pub(crate) fn client_error_with_cause(message: impl Into<String>, cause: GenericError) -> Self {
        Self::TosClientError {
            message: message.into(),
            request_url: "".to_string(),
            cause: Some(cause),
        }
    }

    pub(crate) fn client_error_result(message: impl Into<String>) -> Result<(), TosError> {
        Err(Self::client_error(message))
    }

    pub(crate) fn client_error_result_with_cause(message: impl Into<String>, cause: GenericError) -> Result<(), TosError> {
        Err(Self::client_error_with_cause(message, cause))
    }

    pub(crate) fn server_error(message: impl Into<String>, request_info: RequestInfo) -> Self {
        Self::TosServerError {
            message: message.into(),
            request_url: "".to_string(),
            code: "".to_string(),
            host_id: "".to_string(),
            resource: "".to_string(),
            status_code: request_info.status_code,
            request_id: request_info.request_id,
            ec: "".to_string(),
            key: "".to_string(),
            id2: request_info.id2,
            header: request_info.header,
        }
    }

    pub(crate) fn server_error_with_code(code: impl Into<String>, ec: impl Into<String>, key: impl Into<String>, message: impl Into<String>, host_id: impl Into<String>,
                                         resource: impl Into<String>, request_info: RequestInfo) -> Self {
        Self::TosServerError {
            message: message.into(),
            request_url: "".to_string(),
            code: code.into(),
            host_id: host_id.into(),
            resource: resource.into(),
            status_code: request_info.status_code,
            request_id: request_info.request_id,
            ec: ec.into(),
            key: key.into(),
            id2: request_info.id2,
            header: request_info.header,
        }
    }
}

pub struct TosServerError<'a> {
    message: &'a str,
    request_url: &'a str,
    code: &'a str,
    host_id: &'a str,
    resource: &'a str,
    status_code: isize,
    request_id: &'a str,
    ec: &'a str,
    key: &'a str,
    id2: &'a str,
    header: &'a HashMap<String, String>,
}

impl<'a> TosServerError<'a> {
    pub fn message(&self) -> &'a str {
        self.message
    }
    pub fn code(&self) -> &'a str {
        self.code
    }
    pub fn host_id(&self) -> &'a str {
        self.host_id
    }
    pub fn resource(&self) -> &'a str {
        self.resource
    }
    pub fn status_code(&self) -> isize {
        self.status_code
    }
    pub fn request_id(&self) -> &'a str {
        self.request_id
    }
    pub fn ec(&self) -> &'a str {
        self.ec
    }
    pub fn key(&self) -> &'a str {
        self.key
    }
    pub fn id2(&self) -> &'a str {
        self.id2
    }
    pub fn header(&self) -> &'a HashMap<String, String> {
        self.header
    }
    pub fn request_url(&self) -> &'a str {
        self.request_url
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct ErrorResponse {
    #[serde(default)]
    #[serde(rename = "Code")]
    pub(crate) code: String,
    #[serde(default)]
    #[serde(rename = "Message")]
    pub(crate) message: String,
    #[serde(default)]
    #[serde(rename = "HostId")]
    pub(crate) host_id: String,
    #[serde(default)]
    #[serde(rename = "Resource")]
    pub(crate) resource: String,
    #[serde(default)]
    #[serde(rename = "EC")]
    pub(crate) ec: String,
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "canonical_request")]
    pub(crate) canonical_request: String,
    #[serde(default)]
    #[serde(rename = "StringToSign")]
    pub(crate) string_to_sign: String,
    #[serde(default)]
    #[serde(rename = "SignatureProvided")]
    pub(crate) signature_provided: String,
}


