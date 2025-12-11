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
use crate::credential::{CommonCredentialsProvider, Credentials, EnvCredentialsProvider, StaticCredentialsProvider};
use async_trait::async_trait;
use serde::de::StdError;

#[async_trait]
pub trait CredentialsProvider<C>: Sized
where
    C: Credentials,
{
    async fn credentials(&self, expires: i64) -> Result<&C, Box<dyn StdError + Send + Sync>>;

    fn new(c: C) -> Result<Self, Box<dyn StdError + Send + Sync>>;
}

#[async_trait]
impl<C> CredentialsProvider<C> for CommonCredentialsProvider<C>
where
    C: Credentials + Sync,
{
    async fn credentials(&self, _: i64) -> Result<&C, Box<dyn StdError + Send + Sync>> {
        Ok(&self.credentials)
    }

    fn new(c: C) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        Ok(CommonCredentialsProvider {
            credentials: c,
        })
    }
}

#[async_trait]
impl<C> CredentialsProvider<C> for StaticCredentialsProvider<C>
where
    C: Credentials + Sync,
{
    async fn credentials(&self, _: i64) -> Result<&C, Box<dyn StdError + Send + Sync>> {
        Ok(&self.cred)
    }

    fn new(cred: C) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        Ok(Self {
            cred
        })
    }
}

#[async_trait]
impl<C> CredentialsProvider<C> for EnvCredentialsProvider<C>
where
    C: Credentials + Sync,
{
    async fn credentials(&self, _: i64) -> Result<&C, Box<dyn StdError + Send + Sync>> {
        Ok(&self.cred)
    }

    fn new(cred: C) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        Ok(Self {
            cred
        })
    }
}