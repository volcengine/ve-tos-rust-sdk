/*
 * Copyright (2024) Volcengine
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
use crate::common::{DataTransferStatus, RateLimiter};
use crate::enumeration::HttpMethodType;
use crate::enumeration::HttpMethodType::HttpMethodGet;
use reqwest::blocking::Response;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub(crate) struct RequestContext<'a> {
    pub(crate) file_path: &'a str,
    pub(crate) init_crc64: Option<u64>,
    pub(crate) crc64: Option<u64>,
    pub(crate) rate_limiter: Option<Arc<RateLimiter>>,
    pub(crate) data_transfer_listener: Option<Sender<DataTransferStatus>>,
    pub(crate) async_data_transfer_listener: Option<async_channel::Sender<DataTransferStatus>>,
}

#[derive(Debug, Clone)]
pub(crate) struct HttpRequest<'a, B> {
    pub(crate) operation: &'a str,
    pub(crate) method: HttpMethodType,
    pub(crate) bucket: &'a str,
    pub(crate) key: &'a str,
    pub(crate) header: HashMap<&'a str, String>,
    pub(crate) query: Option<HashMap<&'a str, String>>,
    pub(crate) meta: Option<HashMap<String, String>>,
    pub(crate) body: Option<B>,
    pub(crate) retry_count: isize,
    pub(crate) enable_crc: bool,
    pub(crate) request_context: Option<RequestContext<'a>>,
}

impl<'a, B> Default for HttpRequest<'a, B> {
    fn default() -> Self {
        HttpRequest {
            operation: "Default",
            method: HttpMethodGet,
            bucket: "",
            key: "",
            header: HashMap::with_capacity(8),
            query: None,
            meta: None,
            body: None,
            retry_count: 0,
            enable_crc: false,
            request_context: None,
        }
    }
}

pub(crate) type HttpResponse = Response;


