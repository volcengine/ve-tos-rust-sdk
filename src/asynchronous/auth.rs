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
use crate::auth::{PreSignedPolicyURLInput, PreSignedPolicyURLOutput, PreSignedPostSignatureInput, PreSignedPostSignatureOutput, PreSignedURLInput, PreSignedURLOutput};
use crate::error::TosError;
use async_trait::async_trait;

#[async_trait]
pub trait SignerAPI {
    async fn pre_signed_url(&self, input: &PreSignedURLInput) -> Result<PreSignedURLOutput, TosError>;
    async fn pre_signed_post_signature(&self, input: &PreSignedPostSignatureInput) -> Result<PreSignedPostSignatureOutput, TosError>;
    async fn pre_signed_policy_url(&self, input: &PreSignedPolicyURLInput) -> Result<PreSignedPolicyURLOutput, TosError>;
}