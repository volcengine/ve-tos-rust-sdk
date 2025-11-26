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
use serde::de::StdError;
use std::env;
use std::fmt::Debug;

pub trait Credentials: Sized {
    fn ak(&self) -> &str;
    fn sk(&self) -> &str;
    fn security_token(&self) -> &str;

    fn new(ak: impl Into<String>, sk: impl Into<String>, security_token: impl Into<String>) -> Result<Self, Box<dyn StdError + Send + Sync>>;
}


pub trait CredentialsProvider<C>: Sized
where
    C: Credentials,
{
    fn credentials(&self, expires: i64) -> Result<&C, Box<dyn StdError + Send + Sync>>;

    fn new(c: C) -> Result<Self, Box<dyn StdError + Send + Sync>>;
}


#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommonCredentialsProvider<C> {
    pub(crate) credentials: C,
}

impl<C> CredentialsProvider<C> for CommonCredentialsProvider<C>
where
    C: Credentials,
{
    fn credentials(&self, _: i64) -> Result<&C, Box<dyn StdError + Send + Sync>> {
        Ok(&self.credentials)
    }

    fn new(c: C) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        Ok(CommonCredentialsProvider {
            credentials: c,
        })
    }
}


#[derive(Debug, Clone, PartialEq, Default)]
pub struct CommonCredentials {
    pub(crate) ak: String,
    pub(crate) sk: String,
    pub(crate) security_token: String,
}

impl Credentials for CommonCredentials {
    fn ak(&self) -> &str {
        &self.ak
    }

    fn sk(&self) -> &str {
        &self.sk
    }

    fn security_token(&self) -> &str {
        &self.security_token
    }

    fn new(ak: impl Into<String>, sk: impl Into<String>, security_token: impl Into<String>) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        Ok(CommonCredentials {
            ak: ak.into(),
            sk: sk.into(),
            security_token: security_token.into(),
        })
    }
}


#[derive(Default)]
pub struct StaticCredentialsProvider<C> {
    pub(crate) cred: C,
}

impl<C> StaticCredentialsProvider<C>
where
    C: Credentials,
{
    pub(crate) fn new(ak: impl Into<String>, sk: impl Into<String>, security_token: impl Into<String>) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        let mut ak = ak.into().trim().to_owned();
        let mut sk = sk.into().trim().to_owned();
        let mut security_token = security_token.into().trim().to_owned();
        if ak == "" {
            sk = "".to_owned();
            security_token = "".to_owned();
        } else if sk == "" {
            ak = "".to_owned();
            security_token = "".to_owned();
        }

        let cred = C::new(ak, sk, security_token)?;
        Ok(Self { cred })
    }
}

impl<C> CredentialsProvider<C> for StaticCredentialsProvider<C>
where
    C: Credentials,
{
    fn credentials(&self, _: i64) -> Result<&C, Box<dyn StdError + Send + Sync>> {
        Ok(&self.cred)
    }

    fn new(cred: C) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        Ok(Self {
            cred
        })
    }
}

#[derive(Default)]
pub struct EnvCredentialsProvider<C> {
    pub(crate) cred: C,
}

impl<C> EnvCredentialsProvider<C>
where
    C: Credentials,
{
    pub(crate) fn new() -> Result<Self, Box<dyn StdError + Send + Sync>> {
        let mut ak = "".to_string();
        let mut sk = "".to_string();
        let mut security_token = "".to_string();
        if let Ok(_ak) = env::var("TOS_ACCESS_KEY") {
            if let Ok(_sk) = env::var("TOS_SECRET_KEY") {
                ak = _ak;
                sk = _sk;
                if let Ok(_security_token) = env::var("TOS_SECURITY_TOKEN") {
                    security_token = _security_token;
                }
            }
        }
        let cred = C::new(ak, sk, security_token)?;
        Ok(Self { cred })
    }
}


impl<C> CredentialsProvider<C> for EnvCredentialsProvider<C>
where
    C: Credentials,
{
    fn credentials(&self, _: i64) -> Result<&C, Box<dyn StdError + Send + Sync>> {
        Ok(&self.cred)
    }

    fn new(cred: C) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        Ok(Self {
            cred
        })
    }
}

