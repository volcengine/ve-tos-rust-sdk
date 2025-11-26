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

use crate::config::ConfigHolder;
use crate::constant::{HEADER_AUTHORIZATION, HEADER_CONTENT_SHA256, HEADER_CONTENT_TYPE_LOWER, HEADER_HOST, HEADER_HOST_LOWER, HEADER_PREFIX, HEADER_REQUEST_DATE, HEADER_SECURITY_TOKEN, ISO8601_DATE_FORMAT, LONG_DATE_FORMAT, NOT_ALLOWED_REQUEST_HEADER, QUERY_ALGORITHM, QUERY_CREDENTIAL, QUERY_DATE, QUERY_EXPIRES, QUERY_POLICY, QUERY_SECURITY_TOKEN, QUERY_SIGNATURE, QUERY_SIGNED_HEADERS};
use crate::enumeration::HttpMethodType;
use crate::error::{GenericError, TosError};
use crate::http::HttpRequest;
use crate::internal::{base64, check_bucket, hex, hex_sha256, hmac_sha256, url_encode, url_encode_with_safe, AdditionalContext};
use arc_swap::ArcSwap;
use chrono::{DateTime, TimeDelta, Utc};
use std::collections::HashMap;
use std::ops::Add;
use std::sync::Arc;
use tracing::log::debug;

pub trait SignerAPI {
    fn pre_signed_url(&self, input: &PreSignedURLInput) -> Result<PreSignedURLOutput, TosError>;
    fn pre_signed_post_signature(&self, input: &PreSignedPostSignatureInput) -> Result<PreSignedPostSignatureOutput, TosError>;
    fn pre_signed_policy_url(&self, input: &PreSignedPolicyURLInput) -> Result<PreSignedPolicyURLOutput, TosError>;
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PreSignedURLInput {
    pub(crate) http_method: HttpMethodType,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) expires: i64,
    pub(crate) header: HashMap<String, String>,
    pub(crate) query: HashMap<String, String>,
    pub(crate) alternative_endpoint: String,
    pub(crate) is_custom_domain: Option<bool>,
    pub(crate) is_signed_all_headers: bool,
}

impl PreSignedURLInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            http_method: Default::default(),
            bucket: bucket.into(),
            key: "".to_string(),
            expires: 3600,
            header: Default::default(),
            query: Default::default(),
            alternative_endpoint: "".to_string(),
            is_custom_domain: None,
            is_signed_all_headers: false,
        }
    }

    pub fn new_with_key(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            http_method: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            expires: 3600,
            header: Default::default(),
            query: Default::default(),
            alternative_endpoint: "".to_string(),
            is_custom_domain: None,
            is_signed_all_headers: false,
        }
    }

    pub fn http_method(&self) -> &HttpMethodType {
        &self.http_method
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn expires(&self) -> i64 {
        self.expires
    }

    pub fn header(&self) -> &HashMap<String, String> {
        &self.header
    }

    pub fn query(&self) -> &HashMap<String, String> {
        &self.query
    }

    pub fn alternative_endpoint(&self) -> &str {
        &self.alternative_endpoint
    }

    pub fn is_custom_domain(&self) -> Option<bool> {
        self.is_custom_domain
    }

    pub fn is_signed_all_headers(&self) -> bool {
        self.is_signed_all_headers
    }

    pub fn set_http_method(&mut self, http_method: impl Into<HttpMethodType>) {
        self.http_method = http_method.into();
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_expires(&mut self, expires: i64) {
        self.expires = expires;
    }

    pub fn set_header(&mut self, header: impl Into<HashMap<String, String>>) {
        self.header = header.into();
    }

    pub fn set_query(&mut self, query: impl Into<HashMap<String, String>>) {
        self.query = query.into();
    }

    pub fn set_alternative_endpoint(&mut self, alternative_endpoint: impl Into<String>) {
        self.alternative_endpoint = alternative_endpoint.into();
    }

    pub fn set_is_custom_domain(&mut self, is_custom_domain: impl Into<bool>) {
        self.is_custom_domain = Some(is_custom_domain.into());
    }

    pub fn set_is_signed_all_headers(&mut self, is_signed_all_headers: bool) {
        self.is_signed_all_headers = is_signed_all_headers;
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PreSignedURLOutput {
    pub(crate) signed_url: String,
    pub(crate) signed_header: HashMap<String, String>,
}

impl PreSignedURLOutput {
    pub fn signed_url(&self) -> &str {
        &self.signed_url
    }

    pub fn signed_header(&self) -> &HashMap<String, String> {
        &self.signed_header
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PreSignedPostSignatureInput {
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) expires: i64,
    pub(crate) conditions: Vec<PostSignatureCondition>,
    pub(crate) content_length_range: Option<ContentLengthRange>,
    pub(crate) multi_values_conditions: Vec<PostSignatureMultiValuesCondition>,
}
impl PreSignedPostSignatureInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            key: "".to_string(),
            expires: 3600,
            conditions: vec![],
            content_length_range: None,
            multi_values_conditions: vec![],
        }
    }
    pub fn new_with_key(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            key: key.into(),
            expires: 3600,
            conditions: vec![],
            content_length_range: None,
            multi_values_conditions: vec![],
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn expires(&self) -> i64 {
        self.expires
    }

    pub fn conditions(&self) -> &Vec<PostSignatureCondition> {
        &self.conditions
    }

    pub fn content_length_range(&self) -> &Option<ContentLengthRange> {
        &self.content_length_range
    }

    pub fn multi_values_conditions(&self) -> &Vec<PostSignatureMultiValuesCondition> {
        &self.multi_values_conditions
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_expires(&mut self, expires: i64) {
        self.expires = expires;
    }

    pub fn set_conditions(&mut self, conditions: impl Into<Vec<PostSignatureCondition>>) {
        self.conditions = conditions.into();
    }

    pub fn add_condition(&mut self, condition: impl Into<PostSignatureCondition>) {
        self.conditions.push(condition.into());
    }

    pub fn set_content_length_range(&mut self, content_length_range: impl Into<ContentLengthRange>) {
        self.content_length_range = Some(content_length_range.into());
    }

    pub fn set_multi_values_conditions(&mut self, multi_values_conditions: impl Into<Vec<PostSignatureMultiValuesCondition>>) {
        self.multi_values_conditions = multi_values_conditions.into();
    }

    pub fn add_multi_values_condition(&mut self, multi_values_condition: impl Into<PostSignatureMultiValuesCondition>) {
        self.multi_values_conditions.push(multi_values_condition.into());
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PostSignatureCondition {
    pub(crate) key: String,
    pub(crate) value: String,
    pub(crate) operator: Option<String>,
}

impl PostSignatureCondition {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            operator: None,
        }
    }

    pub fn new_with_operator(key: impl Into<String>, value: impl Into<String>, operator: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            operator: Some(operator.into()),
        }
    }

    pub(crate) fn to_serde_json_value(&self) -> serde_json::Value {
        match &self.operator {
            None => {
                let mut m = serde_json::Map::with_capacity(2);
                m.insert(self.key.clone(), serde_json::Value::String(self.value.clone()));
                serde_json::Value::Object(m)
            }
            Some(operator) => {
                let mut v = Vec::with_capacity(3);
                v.push(serde_json::Value::String(operator.clone()));
                if !&self.key.starts_with("$") {
                    v.push(serde_json::Value::String(String::from("$") + &self.key));
                } else {
                    v.push(serde_json::Value::String(self.key.clone()));
                }
                v.push(serde_json::Value::String(self.value.clone()));
                serde_json::Value::Array(v)
            }
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn operator(&self) -> &Option<String> {
        &self.operator
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }

    pub fn set_operator(&mut self, operator: impl Into<String>) {
        self.operator = Some(operator.into());
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContentLengthRange {
    pub(crate) range_start: i64,
    pub(crate) range_end: i64,
}
impl ContentLengthRange {
    pub fn new(range_start: i64, range_end: i64) -> Self {
        Self {
            range_start,
            range_end,
        }
    }

    pub fn range_start(&self) -> i64 {
        self.range_start
    }

    pub fn range_end(&self) -> i64 {
        self.range_end
    }

    pub fn set_range_start(&mut self, range_start: i64) {
        self.range_start = range_start;
    }

    pub fn set_range_end(&mut self, range_end: i64) {
        self.range_end = range_end;
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PostSignatureMultiValuesCondition {
    pub(crate) key: String,
    pub(crate) values: Vec<String>,
    pub(crate) operator: String,
}

impl PostSignatureMultiValuesCondition {
    pub fn new(key: impl Into<String>, values: impl Into<Vec<String>>, operator: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            values: values.into(),
            operator: operator.into(),
        }
    }

    pub(crate) fn to_serde_json_value(&self) -> serde_json::Value {
        let mut v = Vec::with_capacity(3);
        v.push(serde_json::Value::String(self.operator.clone()));
        if !&self.key.starts_with("$") {
            v.push(serde_json::Value::String(String::from("$") + &self.key));
        } else {
            v.push(serde_json::Value::String(self.key.clone()));
        }
        let mut values = Vec::with_capacity(self.values.len());
        for value in self.values.iter() {
            values.push(serde_json::Value::String(value.clone()));
        }
        v.push(serde_json::Value::Array(values));
        serde_json::Value::Array(v)
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn values(&self) -> &Vec<String> {
        &self.values
    }

    pub fn operator(&self) -> &str {
        &self.operator
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_values(&mut self, values: impl Into<Vec<String>>) {
        self.values = values.into();
    }

    pub fn set_operator(&mut self, operator: impl Into<String>) {
        self.operator = operator.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PreSignedPostSignatureOutput {
    pub(crate) origin_policy: String,
    pub(crate) policy: String,
    pub(crate) algorithm: String,
    pub(crate) credential: String,
    pub(crate) date: String,
    pub(crate) signature: String,
}

impl PreSignedPostSignatureOutput {
    pub fn origin_policy(&self) -> &str {
        &self.origin_policy
    }

    pub fn policy(&self) -> &str {
        &self.policy
    }

    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }

    pub fn credential(&self) -> &str {
        &self.credential
    }

    pub fn date(&self) -> &str {
        &self.date
    }

    pub fn signature(&self) -> &str {
        &self.signature
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PreSignedPolicyURLInput {
    pub(crate) bucket: String,
    pub(crate) expires: i64,
    pub(crate) conditions: Vec<PolicySignatureCondition>,
    pub(crate) alternative_endpoint: String,
    pub(crate) is_custom_domain: Option<bool>,
}
impl PreSignedPolicyURLInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            expires: 3600,
            conditions: vec![],
            alternative_endpoint: "".to_string(),
            is_custom_domain: None,
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn expires(&self) -> i64 {
        self.expires
    }

    pub fn conditions(&self) -> &Vec<PolicySignatureCondition> {
        &self.conditions
    }

    pub fn alternative_endpoint(&self) -> &str {
        &self.alternative_endpoint
    }

    pub fn is_custom_domain(&self) -> Option<bool> {
        self.is_custom_domain
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_expires(&mut self, expires: i64) {
        self.expires = expires;
    }

    pub fn add_condition(&mut self, condition: impl Into<PolicySignatureCondition>) {
        self.conditions.push(condition.into());
    }

    pub fn set_conditions(&mut self, conditions: impl Into<Vec<PolicySignatureCondition>>) {
        self.conditions = conditions.into();
    }

    pub fn set_alternative_endpoint(&mut self, alternative_endpoint: impl Into<String>) {
        self.alternative_endpoint = alternative_endpoint.into();
    }

    pub fn set_is_custom_domain(&mut self, is_custom_domain: bool) {
        self.is_custom_domain = Some(is_custom_domain);
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PolicySignatureCondition {
    pub(crate) key: String,
    pub(crate) value: String,
    pub(crate) operator: String,
}
impl PolicySignatureCondition {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            operator: "".to_string(),
        }
    }

    pub fn new_with_operator(key: impl Into<String>, value: impl Into<String>, operator: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            operator: operator.into(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn operator(&self) -> &str {
        &self.operator
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }

    pub fn set_operator(&mut self, operator: impl Into<String>) {
        self.operator = operator.into();
    }
}
#[derive(Debug, Clone, Default)]
pub struct PreSignedPolicyURLOutput {
    pub(crate) signed_query: String,
    pub(crate) config_holder: Arc<ConfigHolder>,
    pub(crate) is_custom_domain: bool,
    pub(crate) bucket: String,
    pub(crate) domain: String,
    pub(crate) schema: String,
}

impl PreSignedPolicyURLOutput {
    pub fn signed_query(&self) -> &str {
        &self.signed_query
    }

    pub fn get_signed_url_for_list(&self, additional_query: Option<HashMap<impl AsRef<str>, impl AsRef<str>>>) -> String {
        let mut result = self.config_holder.get_endpoint_with_domain(&self.bucket, "", &self.schema, &self.domain, false, self.is_custom_domain);
        result.push('?');
        result.push_str(&self.signed_query);
        if let Some(additional_query) = additional_query {
            for (key, value) in additional_query.iter() {
                result.push('&');
                result.push_str(key.as_ref());
                result.push('=');
                result.push_str(value.as_ref());
            }
        }
        result
    }
    pub fn get_signed_url_for_get_or_head(&self, key: impl AsRef<str>, additional_query: Option<HashMap<impl AsRef<str>, impl AsRef<str>>>) -> String {
        let mut result = self.config_holder.get_endpoint_with_domain(&self.bucket, key.as_ref(), &self.schema, &self.domain, true, self.is_custom_domain);
        result.push('?');
        result.push_str(&self.signed_query);
        if let Some(additional_query) = additional_query {
            for (key, value) in additional_query.iter() {
                result.push('&');
                result.push_str(key.as_ref());
                result.push('=');
                result.push_str(value.as_ref());
            }
        }
        result
    }
}

pub(crate) fn pre_signed_policy_url(config_holder: &ArcSwap<ConfigHolder>, ak: &str, sk: &str, security_token: &str, input: &PreSignedPolicyURLInput) -> Result<PreSignedPolicyURLOutput, TosError> {
    let config_holder = config_holder.load();
    if input.conditions().len() == 0 {
        return Err(TosError::client_error("empty conditions"));
    }

    let mut is_custom_domain = config_holder.is_custom_domain;
    if let Some(x) = input.is_custom_domain() {
        is_custom_domain = x;
    }
    let bucket = input.bucket().trim();
    if !is_custom_domain {
        check_bucket(bucket)?;
    }
    let schema;
    let domain;
    if input.alternative_endpoint() != "" {
        (schema, domain, _) = config_holder.split_endpoint(input.alternative_endpoint())?;
    } else {
        schema = "".to_string();
        domain = "".to_string();
    }

    let is_anonymous = ak == "" || sk == "";
    let mut conditions = Vec::<serde_json::Value>::with_capacity(input.conditions().len() + 1);
    conditions.push(PostSignatureCondition::new("bucket", input.bucket()).to_serde_json_value());
    let (long_date, short_date, credential_scope) = calc_date_and_credential_scope(&config_holder.region, None);
    for condition in input.conditions() {
        if condition.key() != "key" && condition.key() != "$key" {
            return Err(TosError::client_error("condition key must be 'key'"));
        }
        conditions.push(PostSignatureCondition::new_with_operator(condition.key(), condition.value(), condition.operator()).to_serde_json_value());
    }

    let mut query = HashMap::with_capacity(7);
    query.insert(QUERY_ALGORITHM, ALGORITHM.to_string());
    query.insert(QUERY_DATE, long_date.clone());
    query.insert(QUERY_EXPIRES, input.expires().to_string());
    if !is_anonymous {
        query.insert(QUERY_CREDENTIAL, format!("{}/{}", ak, credential_scope));
        if security_token != "" {
            query.insert(QUERY_SECURITY_TOKEN, security_token.to_string());
        }
    }
    match serde_json::to_string(&HashMap::from([("conditions", serde_json::Value::Array(conditions))])) {
        Err(ex) => Err(TosError::client_error_with_cause("trans json error",
                                                         GenericError::JsonError(ex.to_string()))),
        Ok(origin_policy) => {
            let policy = base64(origin_policy.as_str());
            query.insert(QUERY_POLICY, policy);
            let mut query = Some(query);
            if !is_anonymous {
                let canonical_request = calc_canonical_request("", &mut query, "", "", "", UNSIGNED_PAYLOAD);
                let string_to_sign = calc_string_to_sign(&long_date, &credential_scope, &canonical_request);
                let signature = calc_signature(&string_to_sign, &short_date, &config_holder.region, sk)?;
                query.as_mut().unwrap().insert(QUERY_SIGNATURE, signature);
            }
            let mut signed_query = String::new();
            if let Some(query) = query.as_ref() {
                for (idx, kv) in query.iter().enumerate() {
                    signed_query.push_str(url_encode(*kv.0).as_str());
                    signed_query.push('=');
                    signed_query.push_str(kv.1);
                    if idx != query.len() - 1 {
                        signed_query.push('&');
                    }
                }
            }
            Ok(PreSignedPolicyURLOutput {
                signed_query,
                config_holder: config_holder.clone(),
                is_custom_domain,
                bucket: bucket.to_string(),
                domain,
                schema,
            })
        }
    }
}

pub(crate) fn pre_signed_post_signature(config_holder: &ArcSwap<ConfigHolder>, ak: &str, sk: &str, security_token: &str, input: &PreSignedPostSignatureInput) -> Result<PreSignedPostSignatureOutput, TosError> {
    let config_holder = config_holder.load();
    let (long_date, short_date, mut credential) = calc_date_and_credential(ak, &config_holder.region);
    let mut conditions = Vec::<serde_json::Value>::with_capacity(7 + input.conditions().len() + input.multi_values_conditions().len());
    conditions.push(PostSignatureCondition::new(QUERY_ALGORITHM, ALGORITHM).to_serde_json_value());
    conditions.push(PostSignatureCondition::new(QUERY_DATE, long_date.clone()).to_serde_json_value());
    let is_anonymous = ak == "" || sk == "";
    if !is_anonymous {
        conditions.push(PostSignatureCondition::new(QUERY_CREDENTIAL, credential.clone()).to_serde_json_value());
        if security_token != "" {
            conditions.push(PostSignatureCondition::new(QUERY_SECURITY_TOKEN, security_token).to_serde_json_value());
        }
    }

    let bucket = input.bucket().trim();
    if bucket != "" {
        check_bucket(bucket)?;
        conditions.push(PostSignatureCondition::new("bucket", bucket).to_serde_json_value());
    }
    let key = input.key();
    if key != "" {
        conditions.push(PostSignatureCondition::new("key", key).to_serde_json_value());
    }
    if input.conditions().len() > 0 {
        for condition in input.conditions() {
            conditions.push(condition.to_serde_json_value());
        }
    }
    if let Some(content_length_range) = &input.content_length_range {
        let mut condition = PostSignatureCondition::new(content_length_range.range_start.to_string(), content_length_range.range_end.to_string());
        condition.set_operator("content-length-range");
        conditions.push(condition.to_serde_json_value());
    }
    if input.multi_values_conditions().len() > 0 {
        for condition in input.multi_values_conditions() {
            conditions.push(condition.to_serde_json_value());
        }
    }
    let expiration = Utc::now().add(TimeDelta::new(input.expires(), 0).unwrap()).format(ISO8601_DATE_FORMAT).to_string();
    let mut origin_policy = HashMap::with_capacity(2);
    origin_policy.insert("expiration", serde_json::Value::String(expiration));
    origin_policy.insert("conditions", serde_json::Value::Array(conditions));

    match serde_json::to_string(&origin_policy) {
        Err(ex) => Err(TosError::client_error_with_cause("trans json error",
                                                         GenericError::JsonError(ex.to_string()))),
        Ok(result) => {
            let origin_policy = result;
            let policy = base64(origin_policy.as_str());
            let mut signature = String::new();
            if is_anonymous {
                credential = String::new();
            } else {
                signature = calc_signature(&policy, &short_date, &config_holder.region, sk)?;
            }
            Ok(PreSignedPostSignatureOutput {
                origin_policy,
                policy,
                algorithm: ALGORITHM.to_string(),
                credential,
                date: long_date,
                signature,
            })
        }
    }
}
pub(crate) fn pre_signed_url(config_holder: &ArcSwap<ConfigHolder>, ak: &str, sk: &str, security_token: &str, input: &PreSignedURLInput) -> Result<PreSignedURLOutput, TosError> {
    let config_holder = config_holder.load();
    let mut is_custom_domain = config_holder.is_custom_domain;
    if let Some(x) = input.is_custom_domain() {
        is_custom_domain = x;
    }
    let bucket = input.bucket().trim();
    if !is_custom_domain {
        check_bucket(bucket)?;
    }
    let schema;
    let domain;
    if input.alternative_endpoint() != "" {
        (schema, domain, _) = config_holder.split_endpoint(input.alternative_endpoint())?;
    } else {
        schema = "".to_string();
        domain = "".to_string();
    }

    let is_anonymous = ak == "" || sk == "";
    if is_anonymous {
        let signed_url = config_holder.get_endpoint_with_domain(bucket, input.key(), &schema, &domain, true, is_custom_domain);
        return Ok(PreSignedURLOutput {
            signed_url,
            signed_header: HashMap::default(),
        });
    }

    let (long_date, short_date, credential_scope) = calc_date_and_credential_scope(&config_holder.region, None);

    let mut signed_header = HashMap::with_capacity(input.header.len() + 1);
    for (key, value) in &input.header {
        signed_header.insert(key.clone(), value.to_string());
    }

    signed_header.insert(HEADER_HOST.to_string(), config_holder.get_host_with_domain(bucket, &domain, is_custom_domain));
    let (canonical_headers, signed_headers, mut content_sha256) = calc_canonical_headers(&signed_header, &None, input.is_signed_all_headers);

    let mut query = HashMap::with_capacity(input.query.len() + 7);
    for (key, value) in &input.query {
        query.insert(key.as_str(), value.to_string());
    }
    let mut expires = input.expires;
    if expires <= 0 {
        expires = 3600;
    }
    query.insert(QUERY_ALGORITHM, ALGORITHM.to_string());
    query.insert(QUERY_CREDENTIAL, format!("{}/{}", ak, credential_scope));
    query.insert(QUERY_DATE, long_date.clone());
    query.insert(QUERY_EXPIRES, expires.to_string());
    query.insert(QUERY_SIGNED_HEADERS, signed_headers.clone());
    if security_token != "" {
        query.insert(QUERY_SECURITY_TOKEN, security_token.to_string());
    }
    if content_sha256 == "" {
        content_sha256 = UNSIGNED_PAYLOAD.to_string();
    }

    let mut query = Some(query);
    let canonical_request = calc_canonical_request(input.http_method.as_str(), &mut query, input.key(), &canonical_headers, &signed_headers, content_sha256.as_str());
    let string_to_sign = calc_string_to_sign(&long_date, &credential_scope, &canonical_request);
    let signature = calc_signature(&string_to_sign, &short_date, &config_holder.region, sk)?;
    query.as_mut().unwrap().insert(QUERY_SIGNATURE, signature);
    let mut signed_url = config_holder.get_endpoint_with_domain(input.bucket(), input.key(), &schema, &domain, true, is_custom_domain);

    if let Some(query) = query.as_ref() {
        if query.len() > 0 {
            signed_url.push('?');
            for (idx, kv) in query.iter().enumerate() {
                signed_url.push_str(url_encode(*kv.0).as_str());
                signed_url.push('=');
                signed_url.push_str(kv.1);
                if idx != query.len() - 1 {
                    signed_url.push('&');
                }
            }
        }
    }
    Ok(PreSignedURLOutput {
        signed_url,
        signed_header,
    })
}

pub(crate) const SERVICE_TAG: &str = "tos";
pub(crate) const REQUEST_TAG: &str = "request";
pub(crate) const EMPTY_HASH_PAYLOAD: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
pub(crate) const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";
pub(crate) const ALGORITHM: &str = "TOS4-HMAC-SHA256";

pub(crate) fn sign_header<'a, 'c, B>(request: &mut HttpRequest<'c, B>, ak: &str, sk: &str, security_token: &str,
                                     config_holder: &ConfigHolder, ac: &AdditionalContext<'a>) -> Result<(), TosError>
where
    'a: 'c,
{
    let is_anonymous = ak == "" || sk == "";
    if !is_anonymous && security_token != "" {
        request.header.insert(HEADER_SECURITY_TOKEN, security_token.to_string());
    }
    let (long_date, short_date, credential_scope) = calc_date_and_credential_scope(&config_holder.region, ac.request_date);
    let bucket = request.bucket;
    if ac.request_host != "" {
        request.header.insert(HEADER_HOST, ac.request_host.to_string());
    } else if ac.is_control_operation {
        request.header.insert(HEADER_HOST, config_holder.get_host_with_domain(bucket, &config_holder.domain_control, config_holder.is_custom_domain));
    } else {
        request.header.insert(HEADER_HOST, config_holder.get_host(bucket));
    }
    request.header.insert(HEADER_REQUEST_DATE, long_date.clone());
    if let Some(request_header) = ac.request_header {
        if request_header.len() > 0 {
            for (k, v) in request_header.iter() {
                if request.header.contains_key(k.as_str()) {
                    continue;
                }

                if NOT_ALLOWED_REQUEST_HEADER.contains_key(k.to_lowercase().as_str()) {
                    continue;
                }

                request.header.insert(k, v.to_string());
            }
        }
    }

    if let Some(request_query) = ac.request_query {
        if request_query.len() > 0 {
            if request.query.is_none() {
                request.query = Some(HashMap::with_capacity(request_query.len()));
            }
            if let Some(query) = &mut request.query {
                for (k, v) in request_query.iter() {
                    if query.contains_key(k.as_str()) {
                        continue;
                    }
                    query.insert(k, v.to_string());
                }
            }
        }
    }
    let (canonical_headers, signed_headers, _) = calc_canonical_headers(&request.header, &request.meta, false);
    let canonical_request = calc_canonical_request(request.method.as_str(), &mut request.query, request.key, &canonical_headers, &signed_headers, EMPTY_HASH_PAYLOAD);
    debug!("canonical_request: {}", canonical_request);
    if is_anonymous {
        return Ok(());
    }

    let string_to_sign = calc_string_to_sign(&long_date, &credential_scope, &canonical_request);
    // println!("{}", string_to_sign);
    debug!("string_to_sign: {}", string_to_sign);
    let signature = calc_signature(&string_to_sign, &short_date, &config_holder.region, sk)?;
    // println!("{}", signature);

    let mut authorization = String::with_capacity(ALGORITHM.len() + ak.len() +
        credential_scope.len() + signed_headers.len() + signature.len() + 64);
    authorization = authorization + ALGORITHM +
        " Credential=" + ak + "/" + credential_scope.as_str() +
        ", SignedHeaders=" + signed_headers.as_str() +
        ", Signature=" + signature.as_str();
    // println!("{}", authorization);
    request.header.insert(HEADER_AUTHORIZATION, authorization);
    Ok(())
}

pub(crate) fn calc_string_to_sign(long_date: &str, credential_scope: &str, canonical_request: &str) -> String {
    let mut string_to_sign = String::with_capacity(ALGORITHM.len() + long_date.len() + credential_scope.len() + 3);
    string_to_sign = string_to_sign + ALGORITHM + "\n" + long_date + "\n" + credential_scope + "\n" + hex_sha256(canonical_request).as_str();
    // println!("{}", string_to_sign);
    string_to_sign
}

pub(crate) fn calc_signature(string_to_sign: &str, short_date: &str, region: &str, sk: &str) -> Result<String, TosError> {
    let result = hmac_sha256(string_to_sign, hmac_sha256(REQUEST_TAG,
                                                         hmac_sha256(SERVICE_TAG,
                                                                     hmac_sha256(region,
                                                                                 hmac_sha256(short_date, sk)?)?)?)?)?;
    // println!("{}", hex(result.as_ref()));
    Ok(hex(result))
}

pub(crate) fn calc_canonical_request(method: &str, query: &mut Option<HashMap<&str, String>>, key: &str, canonical_headers: &str, signed_headers: &str, content_sha256: &str) -> String {
    let mut canonical_request = String::with_capacity(key.len() * 2 + 64);
    if method != "" {
        canonical_request.push_str(method);
        canonical_request.push('\n');
        canonical_request.push('/');
    }
    if key != "" {
        canonical_request.push_str(url_encode_with_safe(key, "/").as_str());
    }

    if method != "" {
        canonical_request.push('\n');
    }
    if let Some(query) = query.as_mut() {
        let mut keys = Vec::<&str>::with_capacity(query.len());
        for key in query.keys() {
            keys.push(key);
        }
        keys.sort();
        let mut value;
        let mut encoded_value;
        for (idx, key) in keys.iter().enumerate() {
            value = query.get(*key).unwrap();
            canonical_request.push_str(url_encode(*key).as_str());
            canonical_request.push('=');
            if *value != "" {
                encoded_value = url_encode(value);
            } else {
                encoded_value = String::new();
            }
            canonical_request.push_str(encoded_value.as_str());
            if idx != keys.len() - 1 {
                canonical_request.push('&');
            }
            query.insert(*key, encoded_value);
        }
    }
    canonical_request.push('\n');

    if canonical_headers != "" {
        canonical_request.push_str(canonical_headers);
    }
    if method != "" {
        canonical_request.push('\n');
    }

    if signed_headers != "" {
        canonical_request.push_str(signed_headers);
    }
    if method != "" {
        canonical_request.push('\n');
    }

    canonical_request.push_str(content_sha256);
    // println!("{}", canonical_request);
    canonical_request
}

pub(crate) fn calc_date_and_credential(ak: &str, region: &str) -> (String, String, String) {
    let long_date = Utc::now().format(LONG_DATE_FORMAT).to_string();
    let short_date = &long_date[0..8];
    let mut credential_scope = String::with_capacity(ak.len() + short_date.len() + region.len() + SERVICE_TAG.len() + REQUEST_TAG.len() + 4);
    credential_scope.push_str(ak);
    credential_scope.push('/');
    credential_scope.push_str(short_date);
    credential_scope.push('/');
    credential_scope.push_str(region);
    credential_scope.push('/');
    credential_scope.push_str(SERVICE_TAG);
    credential_scope.push('/');
    credential_scope.push_str(REQUEST_TAG);
    let short_date = short_date.to_string();
    (long_date, short_date, credential_scope)
}

pub(crate) fn calc_date_and_credential_scope(region: &str, request_date: Option<DateTime<Utc>>) -> (String, String, String) {
    let now;
    if let Some(request_date) = request_date {
        now = request_date;
    } else {
        now = Utc::now();
    }
    let long_date = now.format(LONG_DATE_FORMAT).to_string();
    let short_date = &long_date[0..8];
    let mut credential_scope = String::with_capacity(short_date.len() + region.len() + SERVICE_TAG.len() + REQUEST_TAG.len() + 3);
    credential_scope.push_str(short_date);
    credential_scope.push('/');
    credential_scope.push_str(region);
    credential_scope.push('/');
    credential_scope.push_str(SERVICE_TAG);
    credential_scope.push('/');
    credential_scope.push_str(REQUEST_TAG);
    let short_date = short_date.to_string();
    (long_date, short_date, credential_scope)
}

pub(crate) fn calc_canonical_headers(header: &HashMap<impl AsRef<str>, String>, meta: &Option<HashMap<String, String>>, is_signed_all_headers: bool) -> (String, String, String) {
    let mut all_header: HashMap<String, &str>;
    let mut keys;
    let mut total_len = 0;
    if let Some(m) = meta {
        keys = Vec::<String>::with_capacity(header.len() + m.len());
        all_header = HashMap::with_capacity(header.len() + m.len());
        for (key, value) in m {
            let key = key.to_lowercase();
            all_header.insert(key.clone(), value);
            total_len += key.len();
            keys.push(key);
        }
    } else {
        keys = Vec::<String>::with_capacity(header.len());
        all_header = HashMap::with_capacity(header.len());
    }

    for (key, value) in header {
        let key = key.as_ref().to_lowercase();
        all_header.insert(key.clone(), value);
        total_len += key.len();
        keys.push(key);
    }

    keys.sort();
    let mut content_sha256 = "".to_string();
    let mut signed_headers = String::with_capacity(total_len + keys.len());
    let mut canonical_headers = String::with_capacity(total_len * 2 + keys.len() * 2);
    for key in keys.iter() {
        if !is_signed_all_headers && key != HEADER_HOST_LOWER && key != HEADER_CONTENT_TYPE_LOWER && !key.starts_with(HEADER_PREFIX) {
            continue;
        }

        let val = all_header.get(key.as_str()).unwrap().trim();
        if key == HEADER_CONTENT_SHA256 {
            content_sha256 = val.to_string();
        }

        signed_headers.push_str(key);
        signed_headers.push(';');
        canonical_headers.push_str(key);
        canonical_headers.push(':');
        canonical_headers.push_str(val);
        canonical_headers.push('\n');
    }

    signed_headers = (&signed_headers[0..signed_headers.len() - 1]).to_string();
    (canonical_headers, signed_headers, content_sha256)
}