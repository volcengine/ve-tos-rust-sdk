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
use crate::asynchronous::http::HttpResponse;
use crate::asynchronous::internal::{read_response_string, OutputParser};
use crate::common::{Meta, RequestInfo};
use crate::control::{DeleteQosPolicyInput, DeleteQosPolicyOutput, GetQosPolicyInput, GetQosPolicyOutput, PutQosPolicyInput, PutQosPolicyOutput};
use crate::error::TosError;
use crate::http::HttpRequest;
use async_trait::async_trait;

#[async_trait]
pub trait ControlAPI {
    async fn put_qos_policy(&self, input: &PutQosPolicyInput) -> Result<PutQosPolicyOutput, TosError>;
    async fn get_qos_policy(&self, input: &GetQosPolicyInput) -> Result<GetQosPolicyOutput, TosError>;
    async fn delete_qos_policy(&self, input: &DeleteQosPolicyInput) -> Result<DeleteQosPolicyOutput, TosError>;
}

#[async_trait]
impl OutputParser for PutQosPolicyOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

#[async_trait]
impl OutputParser for GetQosPolicyOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let policy = read_response_string(response).await?;
        Ok(Self {
            request_info,
            policy,
        })
    }
}

#[async_trait]
impl OutputParser for DeleteQosPolicyOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}