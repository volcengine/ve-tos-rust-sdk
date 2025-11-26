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

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::Arc;

use crate::asynchronous::http::HttpResponse;
use crate::asynchronous::reader::read_at_most;
use crate::common::{Meta, RequestInfo};
use crate::config::ConfigHolder;
use crate::constant::{DEFAULT_READ_BUFFER_SIZE, HEADER_EC, HEADER_ID2, HEADER_PREFIX_META, HEADER_REQUEST_ID, MAX_READ_BUFFER_SIZE_FOR_JSON};
use crate::enumeration::HttpMethodType::HttpMethodHead;
use crate::error::{ErrorResponse, GenericError, TosError};
use crate::http::HttpRequest;
use crate::internal::{parse_json_by_buf, parse_response_string_by_buf, trans_header_value, InputDescriptor, MockAsyncInputTranslator};
use crate::reader::InternalReader;

#[async_trait]
pub(crate) trait AsyncInputTranslator<B>: InputDescriptor {
    async fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError>;
}

#[async_trait]
impl<B> AsyncInputTranslator<B> for MockAsyncInputTranslator {
    async fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        Err(TosError::client_error("invoke MockAsyncInputTranslator.trans"))
    }
}

#[async_trait]
pub(crate) trait OutputParser: Sized {
    async fn check_and_parse<B>(request: HttpRequest<'_, B>, response: HttpResponse) -> Result<Self, TosError>
    where
        B: Send,
    {
        let status = response.status();
        let status_code = status.as_u16();
        let mut header = HashMap::<String, String>::with_capacity(response.headers().len());
        let mut request_id = "".to_string();
        let mut id2 = "".to_string();
        let mut ec = "".to_string();
        let mut k;
        let mut v;
        let mut meta = Meta::new();
        for (key, value) in response.headers() {
            k = key.to_string();
            v = trans_header_value(value);
            if k == HEADER_REQUEST_ID {
                request_id = trans_header_value(value);
            } else if k == HEADER_ID2 {
                id2 = trans_header_value(value);
            } else if k == HEADER_EC {
                ec = trans_header_value(value);
            } else if k.starts_with(HEADER_PREFIX_META) {
                if let Ok(dk) = urlencoding::decode(&k[HEADER_PREFIX_META.len()..]) {
                    if let Ok(dv) = urlencoding::decode(v.as_str()) {
                        meta.insert(dk.to_string(), dv.to_string());
                    }
                }
            }
            header.insert(k, v);
        }

        let request_info = RequestInfo {
            status_code: status_code as isize,
            request_id,
            id2,
            header,
        };

        if status_code >= 300 {
            let status = response.status();
            if request.method != HttpMethodHead {
                if let Ok(error_response) = parse_json::<ErrorResponse>(response).await {
                    // println!("{}", error_response.canonical_request);
                    // println!("{}", error_response.string_to_sign);
                    // println!("{}", error_response.signature_provided);
                    return Err(TosError::server_error_with_code(error_response.code, error_response.ec, error_response.key, error_response.message,
                                                                error_response.host_id, error_response.resource, request_info));
                }
            }
            return Err(TosError::server_error_with_code("", ec, "", String::from("unexpected status code: ") + status.as_str(),
                                                        "", "", request_info));
        }

        Self::parse(request, response, request_info, meta).await
    }
    async fn parse<B>(request: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, meta: Meta) -> Result<Self, TosError>
    where
        B: Send;
}

pub(crate) async fn read_response(response: HttpResponse) -> Result<Vec<u8>, TosError> {
    let mut buf;
    match response.content_length() {
        None => {
            buf = Vec::with_capacity(DEFAULT_READ_BUFFER_SIZE);
            let mut readable = InternalReader::new(response.bytes_stream());
            match read_at_most(&mut readable, &mut buf, MAX_READ_BUFFER_SIZE_FOR_JSON).await {
                Err(e) => Err(TosError::client_error_with_cause("stream read error", GenericError::IoError(e.to_string()))),
                Ok(_) => Ok(buf)
            }
        }
        Some(x) => {
            buf = Vec::with_capacity(x as usize);
            // wrap with reader length check
            let mut readable = InternalReader::sized(response.bytes_stream(), x as usize);
            match read_at_most(&mut readable, &mut buf, MAX_READ_BUFFER_SIZE_FOR_JSON).await {
                Err(e) => Err(TosError::client_error_with_cause("stream read error", GenericError::IoError(e.to_string()))),
                Ok(_) => {
                    Ok(buf)
                }
            }
        }
    }
}

pub(crate) async fn read_response_string(response: HttpResponse) -> Result<String, TosError> {
    let buf = read_response(response).await?;
    parse_response_string_by_buf(buf)
}


pub(crate) async fn parse_json<T>(response: HttpResponse) -> Result<T, TosError>
where
    T: DeserializeOwned,
{
    let buf = read_response(response).await?;
    parse_json_by_buf(buf.as_slice())
}


