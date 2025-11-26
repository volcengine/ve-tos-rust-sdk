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
use crate::common::{GenericInput, RequestInfo};
use crate::config::ConfigHolder;
use crate::constant::{HEADER_ACCOUNT_ID, HEADER_CONTENT_LENGTH, HEADER_CONTENT_MD5};
use crate::enumeration::HttpMethodType::{HttpMethodDelete, HttpMethodGet, HttpMethodPut};
use crate::error::TosError;
use crate::http::HttpRequest;
use crate::internal::{base64_md5, InputDescriptor, InputTranslator};
use crate::reader::BuildBufferReader;
use std::sync::Arc;
use ve_tos_generic::{GenericInput, RequestInfo};

#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct PutQosPolicyInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) account_id: String,
    pub(crate) policy: String,
}

impl PutQosPolicyInput {
    pub fn new(account_id: impl Into<String>, policy: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            account_id: account_id.into(),
            policy: policy.into(),
        }
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn policy(&self) -> &str {
        &self.policy
    }

    pub fn set_account_id(&mut self, account_id: impl Into<String>) {
        self.account_id = account_id.into();
    }

    pub fn set_policy(&mut self, policy: impl Into<String>) {
        self.policy = policy.into();
    }
}

impl InputDescriptor for PutQosPolicyInput {
    fn operation(&self) -> &str {
        "PutQosPolicy"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.account_id)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok("qospolicy")
    }

    fn is_control_operation(&self) -> bool {
        true
    }
}

impl<B> InputTranslator<B> for PutQosPolicyInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.policy == "" {
            return Err(TosError::client_error("empty policy"));
        }
        let mut request = self.trans_key()?;
        request.method = HttpMethodPut;
        request.header.insert(HEADER_CONTENT_MD5, base64_md5(&self.policy));
        request.header.insert(HEADER_ACCOUNT_ID, self.account_id.to_string());
        let (body, len) = B::new(self.policy.clone().into_bytes())?;
        request.body = Some(body);
        request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutQosPolicyOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct GetQosPolicyInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) account_id: String,
}

impl GetQosPolicyInput {
    pub fn new(account_id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            account_id: account_id.into(),
        }
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn set_account_id(&mut self, account_id: impl Into<String>) {
        self.account_id = account_id.into();
    }
}

impl InputDescriptor for GetQosPolicyInput {
    fn operation(&self) -> &str {
        "GetQosPolicy"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.account_id)
    }
    fn key(&self) -> Result<&str, TosError> {
        Ok("qospolicy")
    }
    fn is_control_operation(&self) -> bool {
        true
    }
}

impl<B> InputTranslator<B> for GetQosPolicyInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.header.insert(HEADER_ACCOUNT_ID, self.account_id.to_string());
        request.method = HttpMethodGet;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct GetQosPolicyOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) policy: String,
}

impl GetQosPolicyOutput {
    pub fn policy(&self) -> &str {
        &self.policy
    }
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct DeleteQosPolicyInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) account_id: String,
}

impl DeleteQosPolicyInput {
    pub fn new(account_id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            account_id: account_id.into(),
        }
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn set_account_id(&mut self, account_id: impl Into<String>) {
        self.account_id = account_id.into();
    }
}

impl InputDescriptor for DeleteQosPolicyInput {
    fn operation(&self) -> &str {
        "DeleteQosPolicy"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.account_id)
    }
    fn key(&self) -> Result<&str, TosError> {
        Ok("qospolicy")
    }
    fn is_control_operation(&self) -> bool {
        true
    }
}

impl<B> InputTranslator<B> for DeleteQosPolicyInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.header.insert(HEADER_ACCOUNT_ID, self.account_id.to_string());
        request.method = HttpMethodDelete;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteQosPolicyOutput {
    pub(crate) request_info: RequestInfo,
}
