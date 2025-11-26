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
use crate::enumeration::{CannedType, GranteeType, PermissionType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::ops::Sub;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::time::Duration;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

pub fn init_tracing_log(directives: impl AsRef<str>, directory: impl AsRef<Path>,
                        file_name_prefix: impl AsRef<Path>) -> WorkerGuard {
    let file_appender = tracing_appender::rolling::daily(directory, file_name_prefix);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_writer(non_blocking)
        .with_env_filter(EnvFilter::new(directives))
        .with_ansi(false).init();
    guard
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GenericInput {
    pub(crate) request_date: Option<DateTime<Utc>>,
    pub(crate) request_host: String,
    pub(crate) request_header: Option<HashMap<String, String>>,
    pub(crate) request_query: Option<HashMap<String, String>>,
}

pub trait GenericInputTrait {
    fn request_date(&self) -> Option<DateTime<Utc>>;

    fn request_host(&self) -> &str;

    fn request_header(&self) -> &Option<HashMap<String, String>>;
    fn request_query(&self) -> &Option<HashMap<String, String>>;
}

pub trait RequestInfoTrait {
    fn request_id(&self) -> &str;

    fn id2(&self) -> &str;

    fn status_code(&self) -> isize;
    fn header(&self) -> &HashMap<String, String>;
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RequestInfo {
    pub(crate) request_id: String,
    pub(crate) id2: String,
    pub(crate) status_code: isize,
    pub(crate) header: HashMap<String, String>,
}

impl RequestInfo {
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn id2(&self) -> &str {
        &self.id2
    }

    pub fn status_code(&self) -> isize {
        self.status_code
    }

    pub fn header(&self) -> &HashMap<String, String> {
        &self.header
    }
}

impl RequestInfoTrait for RequestInfo {
    fn request_id(&self) -> &str {
        &self.request_id
    }

    fn id2(&self) -> &str {
        &self.id2
    }

    fn status_code(&self) -> isize {
        self.status_code
    }

    fn header(&self) -> &HashMap<String, String> {
        &self.header
    }
}

pub type Meta = HashMap<String, String>;

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct Owner {
    #[serde(default)]
    #[serde(rename = "ID")]
    pub(crate) id: String,
}

impl Owner {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct Grant {
    #[serde(default)]
    #[serde(rename = "Grantee")]
    pub(crate) grantee: Grantee,
    #[serde(default)]
    #[serde(rename = "Permission")]
    pub(crate) permission: PermissionType,
}

impl Grant {
    pub fn new(grantee: impl Into<Grantee>, permission: impl Into<PermissionType>) -> Self {
        Self {
            grantee: grantee.into(),
            permission: permission.into(),
        }
    }
    pub fn grantee(&self) -> &Grantee {
        &self.grantee
    }
    pub fn permission(&self) -> &PermissionType {
        &self.permission
    }
    pub fn set_grantee(&mut self, grantee: impl Into<Grantee>) {
        self.grantee = grantee.into();
    }
    pub fn set_permission(&mut self, permission: impl Into<PermissionType>) {
        self.permission = permission.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct Grantee {
    #[serde(default)]
    #[serde(rename = "ID")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) id: String,
    #[serde(default)]
    #[serde(rename = "Type")]
    pub(crate) grantee_type: GranteeType,
    #[serde(default)]
    #[serde(rename = "Canned")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) canned: Option<CannedType>,
}

impl Grantee {
    pub fn new(grantee_type: impl Into<GranteeType>) -> Self {
        Self {
            id: "".to_string(),
            grantee_type: grantee_type.into(),
            canned: None,
        }
    }
    pub fn new_with_id(grantee_type: impl Into<GranteeType>, id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            grantee_type: grantee_type.into(),
            canned: None,
        }
    }
    pub fn new_with_canned(grantee_type: impl Into<GranteeType>, canned: impl Into<CannedType>) -> Self {
        Self {
            id: "".to_string(),
            grantee_type: grantee_type.into(),
            canned: Some(canned.into()),
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn grantee_type(&self) -> &GranteeType {
        &self.grantee_type
    }
    pub fn canned(&self) -> &Option<CannedType> {
        &self.canned
    }
    pub fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }
    pub fn set_grantee_type(&mut self, grantee_type: impl Into<GranteeType>) {
        self.grantee_type = grantee_type.into();
    }
    pub fn set_canned(&mut self, canned: impl Into<CannedType>) {
        self.canned = Some(canned.into());
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ListedCommonPrefix {
    #[serde(default)]
    #[serde(rename = "Prefix")]
    pub(crate) prefix: String,
    #[serde(default)]
    #[serde(rename = "LastModified")]
    pub(crate) last_modified_string: Option<String>,
    #[serde(skip)]
    pub(crate) last_modified: Option<DateTime<Utc>>,
}

impl ListedCommonPrefix {
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TagSet {
    #[serde(rename = "Tags")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tags: Vec<Tag>,
}

impl<'de> Deserialize<'de> for TagSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<Value>::deserialize(deserializer)? {
            None => Ok(Self::default()),
            Some(value) => {
                if value.is_object() {
                    if let Some(ts) = value.get("Tags") {
                        if ts.is_array() {
                            if let Some(ts) = ts.as_array() {
                                let mut tags = Vec::with_capacity(ts.len());
                                for tag in ts {
                                    let key = tag.get("Key").unwrap().as_str().unwrap().to_string();
                                    let value = tag.get("Value").unwrap().as_str().unwrap().to_string();
                                    tags.push(Tag {
                                        key,
                                        value,
                                    });
                                }
                                return Ok(Self {
                                    tags,
                                });
                            }
                        }
                    }
                }

                Ok(Self {
                    tags: vec![],
                })
            }
        }
    }
}

impl TagSet {
    pub fn new(tags: impl Into<Vec<Tag>>) -> Self {
        Self {
            tags: tags.into(),
        }
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    pub fn set_tags(&mut self, tags: impl Into<Vec<Tag>>) {
        self.tags = tags.into();
    }
}


#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(rename = "Value")]
    pub(crate) value: String,
}

impl Tag {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub(crate) struct TempCopyResult {
    #[serde(default)]
    #[serde(rename = "ETag")]
    pub(crate) etag: String,
    #[serde(default)]
    #[serde(rename = "LastModified")]
    pub(crate) last_modified: String,
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
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub(crate) struct TempFetchResult {
    #[serde(default)]
    #[serde(rename = "ETag")]
    pub(crate) etag: String,
    #[serde(default)]
    #[serde(rename = "SourceContentType")]
    pub(crate) source_content_type: String,
    #[serde(default)]
    #[serde(rename = "SourceContentLength")]
    pub(crate) source_content_length: i64,
    #[serde(default)]
    #[serde(rename = "MD5")]
    pub(crate) md5: String,
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
}
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub(crate) struct UserMeta {
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "Value")]
    pub(crate) value: String,
}

#[derive(Debug)]
pub struct RateLimiter {
    pub(crate) capacity: i64,
    pub(crate) rate: i64,
    pub(crate) tokens_checkpoint: Mutex<(i64, DateTime<Utc>)>,
}

impl PartialEq for RateLimiter {
    fn eq(&self, other: &Self) -> bool {
        self.capacity == other.capacity && self.rate == other.rate
    }
}

impl RateLimiter {
    pub fn new(mut capacity: i64, rate: i64) -> Self {
        if capacity < rate {
            capacity = rate;
        }
        Self {
            capacity,
            rate,
            tokens_checkpoint: Mutex::new((0, Utc::now())),
        }
    }

    pub fn acquire(&self, want: i64) -> (bool, Option<Duration>) {
        if self.capacity <= 0 || want <= 0 {
            return (true, None);
        }
        let mut tokens_checkpoint = self.tokens_checkpoint.lock().unwrap();
        let now = Utc::now();
        let delta = now.sub(tokens_checkpoint.1).num_milliseconds() * self.rate / 1000 + 1;
        let mut tokens = tokens_checkpoint.0;
        if tokens + delta >= self.capacity {
            tokens = self.capacity;
        } else {
            tokens += delta;
        }

        if tokens >= want {
            tokens_checkpoint.0 = tokens - want;
            tokens_checkpoint.1 = now;
            return (true, None);
        }

        tokens_checkpoint.0 = tokens;
        tokens_checkpoint.1 = now;
        let mut result = (want - tokens) * 1000 / self.rate + 1;
        if result < 10 {
            result = 10;
        }

        (false, Some(Duration::from_millis(result as u64)))
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum DataTransferType {
    DataTransferStarted,
    DataTransferRW,
    DataTransferSucceed,
    DataTransferFailed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataTransferStatus {
    pub(crate) operation: String,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) consumed_bytes: i64,
    pub(crate) total_bytes: i64,
    pub(crate) rw_once_bytes: i64,
    pub(crate) data_transfer_type: DataTransferType,
    pub(crate) retry_count: isize,
}

impl DataTransferStatus {
    pub(crate) fn new(data_transfer_type: DataTransferType, retry_count: isize) -> Self {
        Self {
            operation: "".to_string(),
            bucket: "".to_string(),
            key: "".to_string(),
            consumed_bytes: -1,
            total_bytes: -1,
            rw_once_bytes: -1,
            data_transfer_type,
            retry_count,
        }
    }
    pub(crate) fn set_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = operation.into();
        self
    }
    pub(crate) fn set_bucket(mut self, bucket: impl Into<String>) -> Self {
        self.bucket = bucket.into();
        self
    }
    pub(crate) fn set_key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }
    pub(crate) fn set_consumed_bytes(mut self, consumed_bytes: i64) -> Self {
        self.consumed_bytes = consumed_bytes;
        self
    }
    pub(crate) fn set_total_bytes(mut self, total_bytes: i64) -> Self {
        self.total_bytes = total_bytes;
        self
    }
    pub(crate) fn set_rw_once_bytes(mut self, rw_once_bytes: i64) -> Self {
        self.rw_once_bytes = rw_once_bytes;
        self
    }
    pub fn operation(&self) -> &str {
        &self.operation
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn consumed_bytes(&self) -> i64 {
        self.consumed_bytes
    }

    pub fn total_bytes(&self) -> i64 {
        self.total_bytes
    }

    pub fn rw_once_bytes(&self) -> i64 {
        self.rw_once_bytes
    }

    pub fn data_transfer_type(&self) -> &DataTransferType {
        &self.data_transfer_type
    }

    pub fn retry_count(&self) -> isize {
        self.retry_count
    }
}

pub trait DataTransferListener {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>>;
    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>);
}

