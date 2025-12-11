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
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefCell};
use std::cmp::Ordering;
use std::collections::{HashMap, LinkedList};
use std::io::Read;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use ve_tos_generic::{AclHeader, CallbackHeader, CopySourceHeader, CopySourceIfConditionHeader, CopySourceSSecHeader, GenericInput, HttpBasicHeader, ListCommonQuery, MiscHeader, MultipartUploadQuery, RequestInfo, SseHeader, SsecHeader};

use crate::common::{DataTransferListener, DataTransferStatus, GenericInput, ListedCommonPrefix, Meta, Owner, RateLimiter, RequestInfo, TempCopyResult};
use crate::config::ConfigHolder;
use crate::constant::{HEADER_CALLBACK, HEADER_COMPLETE_ALL, HEADER_CONTENT_LENGTH, HEADER_CONTENT_MD5, HEADER_COPY_SOURCE_RANGE, HEADER_COPY_SOURCE_VERSION_ID, HEADER_ETAG, HEADER_FORBID_OVERWRITE, HEADER_HASH_CRC64ECMA, HEADER_IF_NONE_MATCH, HEADER_LOCATION, HEADER_OBJECT_EXPIRES, HEADER_SERVER_SIDE_ENCRYPTION, HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID, HEADER_SSEC_ALGORITHM, HEADER_SSEC_KEY_MD5, HEADER_TAGGING, HEADER_TRAFFIC_LIMIT, HEADER_VERSION_ID, HEADER_X_IF_MATCH, QUERY_ENCODING_TYPE, QUERY_KEY_MARKER, QUERY_MAX_PARTS, QUERY_MAX_UPLOADS, QUERY_PART_NUMBER, QUERY_PART_NUMBER_MARKER, QUERY_UPLOAD_ID_MARKER};
use crate::enumeration::HttpMethodType::{HttpMethodDelete, HttpMethodGet, HttpMethodPost, HttpMethodPut};
use crate::enumeration::{ACLType, StorageClassType};
use crate::error::{ErrorResponse, GenericError, TosError};
use crate::http::{HttpRequest, HttpResponse, RequestContext};
use crate::internal::{get_header_value, get_header_value_from_str, get_map_value_from_str, get_map_value_str, map_insert, parse_date_time_iso8601, parse_json, parse_json_by_buf, parse_response_string_by_buf, read_response, set_acl_header, set_callback_header, set_copy_source_header, set_copy_source_if_condition_header, set_copy_source_ssec_header, set_http_basic_header, set_list_common_query, set_misc_header, set_multipart_upload_query, set_sse_header, set_ssec_header, set_upload_id, trans_meta, InputDescriptor, InputTranslator, OutputParser};
use crate::reader::{BuildBufferReader, BuildFileReader, BuildMultiBufferReader, MultiBytes};

pub trait MultipartAPI {
    fn create_multipart_upload(&self, input: &CreateMultipartUploadInput) -> Result<CreateMultipartUploadOutput, TosError>;
    fn upload_part<B>(&self, input: &UploadPartInput<B>) -> Result<UploadPartOutput, TosError>
    where
        B: Read + Send + 'static;
    fn upload_part_from_buffer(&self, input: &UploadPartFromBufferInput) -> Result<UploadPartOutput, TosError>;
    fn upload_part_from_file(&self, input: &UploadPartFromFileInput) -> Result<UploadPartOutput, TosError>;
    fn complete_multipart_upload(&self, input: &CompleteMultipartUploadInput) -> Result<CompleteMultipartUploadOutput, TosError>;
    fn abort_multipart_upload(&self, input: &AbortMultipartUploadInput) -> Result<AbortMultipartUploadOutput, TosError>;
    fn upload_part_copy(&self, input: &UploadPartCopyInput) -> Result<UploadPartCopyOutput, TosError>;
    fn list_multipart_uploads(&self, input: &ListMultipartUploadsInput) -> Result<ListMultipartUploadsOutput, TosError>;
    fn list_parts(&self, input: &ListPartsInput) -> Result<ListPartsOutput, TosError>;
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Default,
    HttpBasicHeader,
    AclHeader,
    MiscHeader,
    SseHeader,
    SsecHeader,
    GenericInput
)]
pub struct CreateMultipartUploadInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) encoding_type: String,
    pub(crate) content_length: i64,
    pub(crate) cache_control: String,
    pub(crate) content_disposition: String,
    pub(crate) content_encoding: String,
    pub(crate) content_language: String,
    pub(crate) content_type: String,
    pub(crate) expires: Option<DateTime<Utc>>,
    pub(crate) acl: Option<ACLType>,
    pub(crate) grant_full_control: String,
    pub(crate) grant_read: String,
    pub(crate) grant_read_acp: String,
    pub(crate) grant_write: String,
    pub(crate) grant_write_acp: String,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key: String,
    pub(crate) ssec_key_md5: String,
    pub(crate) server_side_encryption: String,
    pub(crate) server_side_encryption_key_id: String,
    pub(crate) meta: HashMap<String, String>,
    pub(crate) website_redirect_location: String,
    pub(crate) storage_class: Option<StorageClassType>,
    pub(crate) forbid_overwrite: bool,
    pub(crate) tagging: String,
    pub(crate) if_match: String,
    pub(crate) if_none_match: String,
    pub(crate) object_expires: i64,
}

impl CreateMultipartUploadInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.content_length = -1;
        input.object_expires = -1;
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn encoding_type(&self) -> &str {
        &self.encoding_type
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.meta
    }
    pub fn forbid_overwrite(&self) -> bool {
        self.forbid_overwrite
    }
    pub fn tagging(&self) -> &str {
        &self.tagging
    }
    pub fn object_expires(&self) -> i64 {
        self.object_expires
    }
    pub fn if_match(&self) -> &str {
        &self.if_match
    }

    pub fn if_none_match(&self) -> &str {
        &self.if_none_match
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_encoding_type(&mut self, encoding_type: impl Into<String>) {
        self.encoding_type = encoding_type.into();
    }
    pub fn set_meta(&mut self, meta: impl Into<HashMap<String, String>>) {
        self.meta = meta.into();
    }
    pub fn set_forbid_overwrite(&mut self, forbid_overwrite: bool) {
        self.forbid_overwrite = forbid_overwrite;
    }
    pub fn set_tagging(&mut self, tagging: impl Into<String>) {
        self.tagging = tagging.into();
    }

    pub fn set_object_expires(&mut self, object_expires: i64) {
        self.object_expires = object_expires;
    }

    pub fn set_if_match(&mut self, if_match: impl Into<String>) {
        self.if_match = if_match.into();
    }

    pub fn set_if_none_match(&mut self, if_none_match: impl Into<String>) {
        self.if_none_match = if_none_match.into();
    }
}

impl InputDescriptor for CreateMultipartUploadInput {
    fn operation(&self) -> &str {
        "CreateMultipartUpload"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for CreateMultipartUploadInput {
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodPost;
        let header = &mut request.header;
        set_http_basic_header(header, config_holder.disable_encoding_meta, self);
        set_acl_header(header, self);
        set_sse_header(header, self)?;
        set_ssec_header(header, &self.server_side_encryption, self)?;
        request.meta = trans_meta(&self.meta, config_holder.disable_encoding_meta);
        set_misc_header(header, self);
        map_insert(header, HEADER_TAGGING, &self.tagging);
        if self.forbid_overwrite {
            header.insert(HEADER_FORBID_OVERWRITE, self.forbid_overwrite.to_string());
        }
        map_insert(header, HEADER_X_IF_MATCH, &self.if_match);
        map_insert(header, HEADER_IF_NONE_MATCH, &self.if_none_match);
        if self.object_expires >= 0 {
            header.insert(HEADER_OBJECT_EXPIRES, self.object_expires.to_string());
        }
        let mut query = HashMap::with_capacity(2);
        query.insert("uploads", "".to_string());
        map_insert(&mut query, QUERY_ENCODING_TYPE, &self.encoding_type);
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct CreateMultipartUploadOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Bucket")]
    pub(crate) bucket: String,
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "UploadId")]
    pub(crate) upload_id: String,
    #[serde(skip)]
    pub(crate) ssec_algorithm: String,
    #[serde(skip)]
    pub(crate) ssec_key_md5: String,
    #[serde(default)]
    #[serde(rename = "EncodingType")]
    pub(crate) encoding_type: String,
    #[serde(skip)]
    pub(crate) server_side_encryption: String,
    #[serde(skip)]
    pub(crate) server_side_encryption_key_id: String,
}

impl OutputParser for CreateMultipartUploadOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let mut result = parse_json::<Self>(response)?;
        result.ssec_algorithm = get_header_value(response.headers(), HEADER_SSEC_ALGORITHM);
        result.ssec_key_md5 = get_header_value(response.headers(), HEADER_SSEC_KEY_MD5);
        result.server_side_encryption = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION);
        result.server_side_encryption_key_id = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID);
        result.request_info = request_info;
        Ok(result)
    }
}

impl CreateMultipartUploadOutput {
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }
    pub fn ssec_algorithm(&self) -> &str {
        &self.ssec_algorithm
    }
    pub fn ssec_key_md5(&self) -> &str {
        &self.ssec_key_md5
    }
    pub fn encoding_type(&self) -> &str {
        &self.encoding_type
    }
    pub fn server_side_encryption(&self) -> &str {
        &self.server_side_encryption
    }
    pub fn server_side_encryption_key_id(&self) -> &str {
        &self.server_side_encryption_key_id
    }
}

#[derive(Debug, Clone, SsecHeader, MultipartUploadQuery, GenericInput)]
pub(crate) struct UploadPartBasicInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) upload_id: String,
    pub(crate) part_number: isize,
    pub(crate) content_md5: String,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key: String,
    pub(crate) ssec_key_md5: String,
    pub(crate) traffic_limit: i64,
    pub(crate) rate_limiter: Option<Arc<RateLimiter>>,
    pub(crate) data_transfer_listener: Option<Sender<DataTransferStatus>>,
    pub(crate) async_data_transfer_listener: Option<async_channel::Sender<DataTransferStatus>>,
}

impl Default for UploadPartBasicInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            key: "".to_string(),
            upload_id: "".to_string(),
            part_number: 1,
            content_md5: "".to_string(),
            ssec_algorithm: "".to_string(),
            ssec_key: "".to_string(),
            ssec_key_md5: "".to_string(),
            traffic_limit: 0,
            rate_limiter: None,
            data_transfer_listener: None,
            async_data_transfer_listener: None,
        }
    }
}

impl InputDescriptor for UploadPartBasicInput {
    fn operation(&self) -> &str {
        "UploadPart"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for UploadPartBasicInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodPut;
        if let Some(ref rl) = self.rate_limiter {
            let mut rc = RequestContext::default();
            rc.rate_limiter = Some(rl.clone());
            request.request_context = Some(rc);
        }

        if let Some(ref dts) = self.data_transfer_listener {
            if request.request_context.is_some() {
                request.request_context.as_mut().unwrap().data_transfer_listener = Some(dts.clone());
            } else {
                let mut rc = RequestContext::default();
                rc.data_transfer_listener = Some(dts.clone());
                request.request_context = Some(rc);
            }
        } else if let Some(ref adts) = self.async_data_transfer_listener {
            if request.request_context.is_some() {
                request.request_context.as_mut().unwrap().async_data_transfer_listener = Some(adts.clone());
            } else {
                let mut rc = RequestContext::default();
                rc.async_data_transfer_listener = Some(adts.clone());
                request.request_context = Some(rc);
            }
        }

        let header = &mut request.header;
        map_insert(header, HEADER_CONTENT_MD5, &self.content_md5);
        set_ssec_header(header, "", self)?;
        if self.traffic_limit > 0 {
            header.insert(HEADER_TRAFFIC_LIMIT, self.traffic_limit.to_string());
        }
        let mut query = HashMap::with_capacity(2);
        set_multipart_upload_query(&mut query, self)?;
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, SsecHeader, MultipartUploadQuery, GenericInput)]
#[handle_content]
#[use_inner]
pub struct UploadPartInput<B>
{
    pub(crate) inner: UploadPartBasicInput,
    pub(crate) content: Arc<RefCell<Option<B>>>,
    pub(crate) content_length: i64,
}

unsafe impl<B> Sync for UploadPartInput<B> {}
unsafe impl<B> Send for UploadPartInput<B> {}

unsafe impl<B> Send for UploadPartInput<B> {}

impl<B> InputDescriptor for UploadPartInput<B>
{
    fn operation(&self) -> &str {
        "UploadPart"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.inner.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.inner.key)
    }
}

impl<B> InputTranslator<B> for UploadPartInput<B>
{
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.inner.trans(config_holder)?;
        request.operation = self.operation();
        request.body = self.content.take();
        if self.content_length >= 0 {
            request.header.insert(HEADER_CONTENT_LENGTH, self.content_length.to_string());
        }
        Ok(request)
    }
}

impl<B> UploadPartInput<B>
{
    pub fn new(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input
    }
    pub fn new_with_part_number(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>, part_number: isize) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input.inner.part_number = part_number;
        input
    }

    pub fn new_with_content(bucket: impl Into<String>, key: impl Into<String>,
                            upload_id: impl Into<String>, content: impl Into<B>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input.set_content(content);
        input
    }

    pub fn new_with_part_number_content(bucket: impl Into<String>, key: impl Into<String>,
                                        upload_id: impl Into<String>, part_number: isize, content: impl Into<B>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input.inner.part_number = part_number;
        input.set_content(content);
        input
    }

    pub fn bucket(&self) -> &str {
        &self.inner.bucket
    }
    pub fn key(&self) -> &str {
        &self.inner.key
    }
    pub fn content(&self) -> Ref<Option<B>> {
        self.content.borrow()
    }
    pub fn content_length(&self) -> i64 {
        self.content_length
    }
    pub fn content_md5(&self) -> &str {
        &self.inner.content_md5
    }
    pub fn traffic_limit(&self) -> i64 {
        self.inner.traffic_limit
    }
    pub fn rate_limiter(&self) -> &Option<Arc<RateLimiter>> {
        &self.inner.rate_limiter
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.inner.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.inner.key = key.into();
    }
    pub fn set_content(&mut self, content: impl Into<B>) {
        self.content = Arc::new(RefCell::new(Some(content.into())));
    }
    pub fn set_content_length(&mut self, content_length: i64) {
        self.content_length = content_length;
    }
    pub fn set_content_md5(&mut self, content_md5: impl Into<String>) {
        self.inner.content_md5 = content_md5.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.inner.traffic_limit = traffic_limit;
    }
    pub fn set_rate_limiter(&mut self, rate_limiter: impl Into<Arc<RateLimiter>>) {
        self.inner.rate_limiter = Some(rate_limiter.into());
    }
}

impl<B> Default for UploadPartInput<B>
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            content: Arc::new(RefCell::new(None)),
            content_length: -1,
        }
    }
}

impl<B> DataTransferListener for UploadPartInput<B> {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>> {
        &self.inner.data_transfer_listener
    }

    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>) {
        self.inner.data_transfer_listener = Some(listener.into());
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct UploadPartOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) part_number: isize,
    pub(crate) etag: String,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key_md5: String,
    pub(crate) hash_crc64ecma: u64,
    pub(crate) server_side_encryption: String,
    pub(crate) server_side_encryption_key_id: String,
}

impl OutputParser for UploadPartOutput {
    fn parse_by_ref<B>(request: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let hash_crc64ecma = get_header_value_from_str::<u64>(response.headers(), HEADER_HASH_CRC64ECMA, 0)?;
        if let Some(ref rc) = request.request_context {
            if let Some(calc_hash_crc64ecma) = rc.crc64 {
                if calc_hash_crc64ecma != hash_crc64ecma {
                    return Err(TosError::client_error(format!("expect crc64 {hash_crc64ecma}, actual crc64 {calc_hash_crc64ecma}")));
                }
            }
        }
        let result = Self {
            request_info: request_info,
            part_number: get_map_value_from_str(request.query.as_ref().unwrap(), QUERY_PART_NUMBER, 1)?,
            etag: get_header_value(response.headers(), HEADER_ETAG),
            ssec_algorithm: get_header_value(response.headers(), HEADER_SSEC_ALGORITHM),
            ssec_key_md5: get_header_value(response.headers(), HEADER_SSEC_KEY_MD5),
            hash_crc64ecma: hash_crc64ecma,
            server_side_encryption: get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION),
            server_side_encryption_key_id: get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID),
        };
        Ok(result)
    }
}

impl UploadPartOutput {
    pub fn part_number(&self) -> isize {
        self.part_number
    }
    pub fn etag(&self) -> &str {
        &self.etag
    }
    pub fn ssec_algorithm(&self) -> &str {
        &self.ssec_algorithm
    }
    pub fn ssec_key_md5(&self) -> &str {
        &self.ssec_key_md5
    }
    pub fn hash_crc64ecma(&self) -> u64 {
        self.hash_crc64ecma
    }
    pub fn server_side_encryption(&self) -> &str {
        &self.server_side_encryption
    }
    pub fn server_side_encryption_key_id(&self) -> &str {
        &self.server_side_encryption_key_id
    }
}

#[derive(Debug, SsecHeader, MultipartUploadQuery, GenericInput)]
#[use_inner]
pub struct UploadPartFromBufferInput {
    pub(crate) inner: UploadPartBasicInput,
    pub(crate) content: Option<MultiBytes>,
    pub(crate) content_length: i64,
}

impl InputDescriptor for UploadPartFromBufferInput {
    fn operation(&self) -> &str {
        "UploadPartFromBuffer"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.inner.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.inner.key)
    }
}

impl<B> InputTranslator<B> for UploadPartFromBufferInput
where
    B: BuildMultiBufferReader,
{
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.inner.trans(config_holder)?;
        request.operation = self.operation();
        if self.content_length >= 0 {
            request.header.insert(HEADER_CONTENT_LENGTH, self.content_length.to_string());
        }
        if let Some(content) = &self.content {
            let (body, len) = B::new(content.clone())?;
            request.body = Some(body);
            if self.content_length < 0 {
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
            }
        }
        Ok(request)
    }
}

impl UploadPartFromBufferInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input
    }
    pub fn new_with_part_number(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>, part_number: isize) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input.inner.part_number = part_number;
        input
    }

    pub fn new_with_content(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>, content: impl AsRef<[u8]>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input.set_content(content);
        input
    }

    pub fn new_with_part_number_content(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>, part_number: isize, content: impl AsRef<[u8]>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input.inner.part_number = part_number;
        input.set_content(content);
        input
    }

    pub fn bucket(&self) -> &str {
        &self.inner.bucket
    }
    pub fn key(&self) -> &str {
        &self.inner.key
    }
    pub fn content(&self) -> Option<impl Iterator<Item=&Bytes>> {
        match &self.content {
            None => None,
            Some(x) => Some(x.inner.iter()),
        }
    }
    pub fn content_length(&self) -> i64 {
        self.content_length
    }
    pub fn content_md5(&self) -> &str {
        &self.inner.content_md5
    }
    pub fn traffic_limit(&self) -> i64 {
        self.inner.traffic_limit
    }
    pub fn rate_limiter(&self) -> &Option<Arc<RateLimiter>> {
        &self.inner.rate_limiter
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.inner.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.inner.key = key.into();
    }
    pub fn set_content_with_bytes_list(&mut self, bytes_list: impl Iterator<Item=impl Into<Bytes>>) {
        let mut list = LinkedList::new();
        let mut size = 0;
        for item in bytes_list {
            let item = item.into();
            size += item.len();
            list.push_back(item);
        }
        self.content = Some(MultiBytes::new(list, size));
    }
    pub fn set_content(&mut self, content: impl AsRef<[u8]>) {
        let item = content.as_ref().to_owned();
        let size = item.len();
        let mut list = LinkedList::new();
        list.push_back(Bytes::from(item));
        self.content = Some(MultiBytes::new(list, size));
    }
    pub fn append_content(&mut self, content: impl AsRef<[u8]>) {
        if let Some(contents) = &mut self.content {
            contents.push(Bytes::from(content.as_ref().to_owned()));
        } else {
            self.set_content(content);
        }
    }
    pub fn set_content_nocopy(&mut self, content: impl Into<Vec<u8>>) {
        let item = content.into();
        let size = item.len();
        let mut list = LinkedList::new();
        list.push_back(Bytes::from(item));
        self.content = Some(MultiBytes::new(list, size));
    }
    pub fn append_content_nocopy(&mut self, content: impl Into<Vec<u8>>) {
        if let Some(contents) = &mut self.content {
            contents.push(Bytes::from(content.into()));
        } else {
            self.set_content_nocopy(content);
        }
    }
    pub fn set_content_length(&mut self, content_length: i64) {
        self.content_length = content_length;
    }
    pub fn set_content_md5(&mut self, content_md5: impl Into<String>) {
        self.inner.content_md5 = content_md5.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.inner.traffic_limit = traffic_limit;
    }
    pub fn set_rate_limiter(&mut self, rate_limiter: impl Into<Arc<RateLimiter>>) {
        self.inner.rate_limiter = Some(rate_limiter.into());
    }
}

impl Default for UploadPartFromBufferInput {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            content: None,
            content_length: -1,
        }
    }
}

impl DataTransferListener for UploadPartFromBufferInput {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>> {
        &self.inner.data_transfer_listener
    }

    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>) {
        self.inner.data_transfer_listener = Some(listener.into());
    }
}

#[derive(Debug, SsecHeader, MultipartUploadQuery, GenericInput)]
#[use_inner]
pub struct UploadPartFromFileInput {
    pub(crate) inner: UploadPartBasicInput,
    pub(crate) offset: i64,
    pub(crate) part_size: i64,
    pub(crate) file_path: String,
}


impl InputDescriptor for UploadPartFromFileInput {
    fn operation(&self) -> &str {
        "UploadPartFromFile"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.inner.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.inner.key)
    }
}

impl<B> InputTranslator<B> for UploadPartFromFileInput
where
    B: BuildFileReader,
{
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.inner.trans(config_holder)?;
        request.operation = self.operation();
        if self.offset < 0 {
            return Err(TosError::client_error("invalid offset for upload part"));
        }
        if self.part_size >= 0 {
            request.header.insert(HEADER_CONTENT_LENGTH, self.part_size.to_string());
        }
        if self.file_path != "" {
            let (body, len) = B::new_with_offset(&self.file_path, self.offset)?;
            request.body = Some(body);
            if let Some(l) = len {
                if self.part_size < 0 {
                    request.header.insert(HEADER_CONTENT_LENGTH, (l - self.offset as usize).to_string());
                }
            }
        }
        Ok(request)
    }
}

impl UploadPartFromFileInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input
    }
    pub fn new_with_part_number(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>, part_number: isize) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input.inner.part_number = part_number;
        input
    }

    pub fn new_with_file_path(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>, file_path: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input.file_path = file_path.into();
        input
    }

    pub fn new_with_part_number_file_path(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>, part_number: isize, file_path: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.upload_id = upload_id.into();
        input.inner.part_number = part_number;
        input.file_path = file_path.into();
        input
    }

    pub fn bucket(&self) -> &str {
        &self.inner.bucket
    }
    pub fn key(&self) -> &str {
        &self.inner.key
    }
    pub fn file_path(&self) -> &str {
        &self.file_path
    }
    pub fn offset(&self) -> i64 {
        self.offset
    }
    pub fn part_size(&self) -> i64 {
        self.part_size
    }
    pub fn content_md5(&self) -> &str {
        &self.inner.content_md5
    }
    pub fn traffic_limit(&self) -> i64 {
        self.inner.traffic_limit
    }
    pub fn rate_limiter(&self) -> &Option<Arc<RateLimiter>> {
        &self.inner.rate_limiter
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.inner.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.inner.key = key.into();
    }
    pub fn set_file_path(&mut self, file_path: impl Into<String>) {
        self.file_path = file_path.into();
    }
    pub fn set_offset(&mut self, offset: i64) {
        self.offset = offset;
    }
    pub fn set_part_size(&mut self, part_size: i64) {
        self.part_size = part_size;
    }
    pub fn set_content_md5(&mut self, content_md5: impl Into<String>) {
        self.inner.content_md5 = content_md5.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.inner.traffic_limit = traffic_limit;
    }
    pub fn set_rate_limiter(&mut self, rate_limiter: impl Into<Arc<RateLimiter>>) {
        self.inner.rate_limiter = Some(rate_limiter.into());
    }
}

impl Default for UploadPartFromFileInput {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            offset: 0,
            part_size: -1,
            file_path: "".to_string(),
        }
    }
}

impl DataTransferListener for UploadPartFromFileInput {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>> {
        &self.inner.data_transfer_listener
    }

    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>) {
        self.inner.data_transfer_listener = Some(listener.into());
    }
}


#[derive(Debug, Clone, PartialEq, Default, CallbackHeader, GenericInput)]
pub struct CompleteMultipartUploadInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) upload_id: String,
    pub(crate) parts: Vec<UploadedPart>,
    pub(crate) complete_all: bool,
    pub(crate) callback: String,
    pub(crate) callback_var: String,
    pub(crate) forbid_overwrite: bool,
    pub(crate) if_match: String,
    pub(crate) if_none_match: String,
}

impl InputDescriptor for CompleteMultipartUploadInput {
    fn operation(&self) -> &str {
        "CompleteMultipartUpload"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for CompleteMultipartUploadInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodPost;
        let header = &mut request.header;
        if self.complete_all {
            header.insert(HEADER_COMPLETE_ALL, "yes".to_string());
        } else if self.parts.len() == 0 {
            return Err(TosError::client_error("empty parts for complete multipart upload"));
        } else {
            let mut parts = Vec::with_capacity(self.parts.len());
            for part in self.parts.iter() {
                parts.push(part);
            }
            parts.sort();
            match serde_json::to_string(&TempUploadedParts { parts }) {
                Err(e) => return Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
                Ok(json) => {
                    let (body, len) = B::new(Bytes::from(json.into_bytes()))?;
                    request.body = Some(body);
                    header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                }
            }
        }
        set_callback_header(header, self);
        map_insert(header, HEADER_X_IF_MATCH, &self.if_match);
        map_insert(header, HEADER_IF_NONE_MATCH, &self.if_none_match);
        if self.forbid_overwrite {
            header.insert(HEADER_FORBID_OVERWRITE, self.forbid_overwrite.to_string());
        }
        let mut query = HashMap::with_capacity(2);
        set_upload_id(&mut query, &self.upload_id)?;
        request.query = Some(query);
        Ok(request)
    }
}

impl CompleteMultipartUploadInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.upload_id = upload_id.into();
        input
    }

    pub fn new_with_parts(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>, parts: impl Into<Vec<UploadedPart>>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.upload_id = upload_id.into();
        input.parts = parts.into();
        input
    }

    pub fn new_with_complete_all(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>, complete_all: bool) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.upload_id = upload_id.into();
        input.complete_all = complete_all;
        input
    }

    pub fn add_part(&mut self, part: impl Into<UploadedPart>) {
        self.parts.push(part.into());
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }
    pub fn parts(&self) -> &Vec<UploadedPart> {
        &self.parts
    }
    pub fn complete_all(&self) -> bool {
        self.complete_all
    }
    pub fn forbid_overwrite(&self) -> bool {
        self.forbid_overwrite
    }
    pub fn if_match(&self) -> &str {
        &self.if_match
    }

    pub fn if_none_match(&self) -> &str {
        &self.if_none_match
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_upload_id(&mut self, upload_id: impl Into<String>) {
        self.upload_id = upload_id.into();
    }
    pub fn set_parts(&mut self, parts: impl Into<Vec<UploadedPart>>) {
        self.parts = parts.into();
    }
    pub fn set_complete_all(&mut self, complete_all: bool) {
        self.complete_all = complete_all;
    }
    pub fn set_forbid_overwrite(&mut self, forbid_overwrite: bool) {
        self.forbid_overwrite = forbid_overwrite;
    }
    pub fn set_if_match(&mut self, if_match: impl Into<String>) {
        self.if_match = if_match.into();
    }
    pub fn set_if_none_match(&mut self, if_none_match: impl Into<String>) {
        self.if_none_match = if_none_match.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub(crate) struct TempUploadedParts<'a> {
    #[serde(default)]
    #[serde(rename = "Parts")]
    pub(crate) parts: Vec<&'a UploadedPart>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
pub struct UploadedPart {
    #[serde(default)]
    #[serde(rename = "PartNumber")]
    pub(crate) part_number: isize,
    #[serde(default)]
    #[serde(rename = "ETag")]
    pub(crate) etag: String,
    #[serde(default)]
    #[serde(rename = "Size")]
    #[serde(skip_serializing)]
    pub(crate) size: i64,
    #[serde(default)]
    #[serde(rename = "LastModified")]
    #[serde(skip_serializing)]
    pub(crate) last_modified_string: Option<String>,
    #[serde(skip)]
    pub(crate) last_modified: Option<DateTime<Utc>>,
}

impl Eq for UploadedPart {}

impl PartialOrd<Self> for UploadedPart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.part_number.cmp(&other.part_number))
    }
}

impl Ord for UploadedPart {
    fn cmp(&self, other: &Self) -> Ordering {
        self.part_number.cmp(&other.part_number)
    }
}

impl UploadedPart {
    pub fn new(part_number: isize, etag: impl Into<String>) -> Self {
        Self {
            part_number: part_number,
            etag: etag.into(),
            size: 0,
            last_modified_string: None,
            last_modified: None,
        }
    }
    pub fn part_number(&self) -> isize {
        self.part_number
    }
    pub fn etag(&self) -> &str {
        &self.etag
    }
    pub fn size(&self) -> i64 {
        self.size
    }
    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }
    pub fn set_part_number(&mut self, part_number: isize) {
        self.part_number = part_number;
    }
    pub fn set_etag(&mut self, etag: impl Into<String>) {
        self.etag = etag.into();
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct CompleteMultipartUploadOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Bucket")]
    pub(crate) bucket: String,
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "ETag")]
    pub(crate) etag: String,
    #[serde(default)]
    #[serde(rename = "Location")]
    pub(crate) location: String,
    #[serde(skip)]
    pub(crate) version_id: String,
    #[serde(skip)]
    pub(crate) hash_crc64ecma: u64,
    #[serde(default)]
    #[serde(rename = "CompletedParts")]
    pub(crate) completed_parts: Option<Vec<UploadedPart>>,
    #[serde(skip)]
    pub(crate) callback_result: String,
    #[serde(skip)]
    pub(crate) server_side_encryption: String,
    #[serde(skip)]
    pub(crate) server_side_encryption_key_id: String,
}

impl OutputParser for CompleteMultipartUploadOutput {
    fn parse_by_ref<B>(request: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let mut result;
        if get_map_value_str(&request.header, HEADER_CALLBACK) != "" { // callback
            let buf = read_response(response)?;
            if request_info.status_code == 203 {
                if let Ok(error_response) = parse_json_by_buf::<ErrorResponse>(buf.as_slice()) {
                    return Err(TosError::server_error_with_code(error_response.code, error_response.ec, error_response.key, error_response.message,
                                                                error_response.host_id, error_response.resource, request_info));
                }
            }
            result = Self::default();
            result.bucket = request.bucket.to_string();
            result.key = request.key.to_string();
            result.etag = get_header_value(response.headers(), HEADER_ETAG);
            result.location = get_header_value(response.headers(), HEADER_LOCATION);
            result.callback_result = parse_response_string_by_buf(buf)?;
        } else {
            result = parse_json::<Self>(response)?;
        }

        result.version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        result.hash_crc64ecma = get_header_value_from_str::<u64>(response.headers(), HEADER_HASH_CRC64ECMA, 0)?;
        result.server_side_encryption = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION);
        result.server_side_encryption_key_id = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID);
        result.request_info = request_info;
        Ok(result)
    }
}

impl CompleteMultipartUploadOutput {
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn etag(&self) -> &str {
        &self.etag
    }
    pub fn location(&self) -> &str {
        &self.location
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn hash_crc64ecma(&self) -> u64 {
        self.hash_crc64ecma
    }
    pub fn completed_parts(&self) -> &Option<Vec<UploadedPart>> {
        &self.completed_parts
    }
    pub fn callback_result(&self) -> &str {
        &self.callback_result
    }
    pub fn server_side_encryption(&self) -> &str {
        &self.server_side_encryption
    }
    pub fn server_side_encryption_key_id(&self) -> &str {
        &self.server_side_encryption_key_id
    }
}

#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct AbortMultipartUploadInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) upload_id: String,
}

impl AbortMultipartUploadInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>) -> Self {
        Self { generic_input: Default::default(), bucket: bucket.into(), key: key.into(), upload_id: upload_id.into() }
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_upload_id(&mut self, upload_id: impl Into<String>) {
        self.upload_id = upload_id.into();
    }
}

impl InputDescriptor for AbortMultipartUploadInput {
    fn operation(&self) -> &str {
        "AbortMultipartUpload"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for AbortMultipartUploadInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodDelete;
        let mut query = HashMap::with_capacity(1);
        set_upload_id(&mut query, &self.upload_id)?;
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct AbortMultipartUploadOutput {
    pub(crate) request_info: RequestInfo,
}

impl OutputParser for AbortMultipartUploadOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, _: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        Ok(Self { request_info })
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    SsecHeader,
    CopySourceHeader,
    CopySourceSSecHeader,
    CopySourceIfConditionHeader,
    MultipartUploadQuery,
    GenericInput
)]
pub struct UploadPartCopyInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) upload_id: String,
    pub(crate) part_number: isize,

    pub(crate) src_bucket: String,
    pub(crate) src_key: String,
    pub(crate) src_version_id: String,
    pub(crate) copy_source_range_start: i64,
    pub(crate) copy_source_range_end: i64,
    pub(crate) copy_source_range: String,

    pub(crate) copy_source_if_match: String,
    pub(crate) copy_source_if_modified_since: Option<DateTime<Utc>>,
    pub(crate) copy_source_if_none_match: String,
    pub(crate) copy_source_if_unmodified_since: Option<DateTime<Utc>>,

    pub(crate) copy_source_ssec_algorithm: String,
    pub(crate) copy_source_ssec_key: String,
    pub(crate) copy_source_ssec_key_md5: String,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key: String,
    pub(crate) ssec_key_md5: String,
    pub(crate) traffic_limit: i64,
}

impl InputDescriptor for UploadPartCopyInput {
    fn operation(&self) -> &str {
        "UploadPartCopy"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for UploadPartCopyInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodPut;
        let header = &mut request.header;
        if self.copy_source_range != "" {
            if !self.copy_source_range.starts_with("bytes=") {
                return Err(TosError::client_error("invalid copy source range format"));
            }
            header.insert(HEADER_COPY_SOURCE_RANGE, self.copy_source_range.clone());
        } else if self.copy_source_range_start >= 0 && self.copy_source_range_end >= 0 && self.copy_source_range_start <= self.copy_source_range_end {
            header.insert(HEADER_COPY_SOURCE_RANGE, format!("bytes={}-{}", self.copy_source_range_start, self.copy_source_range_end));
        }

        set_copy_source_header(header, self)?;
        set_copy_source_if_condition_header(header, self);
        set_ssec_header(header, "", self)?;
        set_copy_source_ssec_header(header, self)?;
        if self.traffic_limit > 0 {
            header.insert(HEADER_TRAFFIC_LIMIT, self.traffic_limit.to_string());
        }
        let mut query = HashMap::with_capacity(2);
        set_multipart_upload_query(&mut query, self)?;
        request.query = Some(query);
        Ok(request)
    }
}

impl Default for UploadPartCopyInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            key: "".to_string(),
            upload_id: "".to_string(),
            part_number: 1,
            src_bucket: "".to_string(),
            src_key: "".to_string(),
            src_version_id: "".to_string(),
            copy_source_range_start: -1,
            copy_source_range_end: -1,
            copy_source_range: "".to_string(),
            copy_source_if_match: "".to_string(),
            copy_source_if_modified_since: None,
            copy_source_if_none_match: "".to_string(),
            copy_source_if_unmodified_since: None,
            copy_source_ssec_algorithm: "".to_string(),
            copy_source_ssec_key: "".to_string(),
            copy_source_ssec_key_md5: "".to_string(),
            ssec_algorithm: "".to_string(),
            ssec_key: "".to_string(),
            ssec_key_md5: "".to_string(),
            traffic_limit: 0,
        }
    }
}

impl UploadPartCopyInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>,
               src_bucket: impl Into<String>, src_key: impl Into<String>, upload_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.src_bucket = src_bucket.into();
        input.src_key = src_key.into();
        input.upload_id = upload_id.into();
        input
    }

    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>,
                               src_bucket: impl Into<String>, src_key: impl Into<String>, upload_id: impl Into<String>, src_version_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.src_bucket = src_bucket.into();
        input.src_key = src_key.into();
        input.upload_id = upload_id.into();
        input.src_version_id = src_version_id.into();
        input
    }
    pub fn new_with_part_number(bucket: impl Into<String>, key: impl Into<String>, src_bucket: impl Into<String>, src_key: impl Into<String>,
                                upload_id: impl Into<String>, part_number: isize) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.src_bucket = src_bucket.into();
        input.src_key = src_key.into();
        input.upload_id = upload_id.into();
        input.part_number = part_number;
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn copy_source_range_start(&self) -> i64 {
        self.copy_source_range_start
    }
    pub fn copy_source_range_end(&self) -> i64 {
        self.copy_source_range_end
    }
    pub fn copy_source_range(&self) -> &str {
        &self.copy_source_range
    }
    pub fn traffic_limit(&self) -> i64 {
        self.traffic_limit
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_copy_source_range_start(&mut self, copy_source_range_start: i64) {
        self.copy_source_range_start = copy_source_range_start;
    }
    pub fn set_copy_source_range_end(&mut self, copy_source_range_end: i64) {
        self.copy_source_range_end = copy_source_range_end;
    }
    pub fn set_copy_source_range(&mut self, copy_source_range: impl Into<String>) {
        self.copy_source_range = copy_source_range.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.traffic_limit = traffic_limit;
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct UploadPartCopyOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) part_number: isize,
    pub(crate) etag: String,
    pub(crate) last_modified: Option<DateTime<Utc>>,
    pub(crate) copy_source_version_id: String,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key_md5: String,
    pub(crate) server_side_encryption: String,
    pub(crate) server_side_encryption_key_id: String,
}

impl OutputParser for UploadPartCopyOutput {
    fn parse_by_ref<B>(request: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let temp_result = parse_json::<TempCopyResult>(response)?;
        if temp_result.etag == "" {
            return Err(TosError::server_error_with_code(temp_result.code, temp_result.ec, temp_result.key, temp_result.message,
                                                        temp_result.host_id, temp_result.resource, request_info));
        }
        let result = Self {
            request_info: request_info,
            part_number: get_map_value_from_str(request.query.as_ref().unwrap(), QUERY_PART_NUMBER, 1)?,
            etag: temp_result.etag,
            last_modified: parse_date_time_iso8601(&temp_result.last_modified)?,
            copy_source_version_id: get_header_value(response.headers(), HEADER_COPY_SOURCE_VERSION_ID),
            ssec_algorithm: get_header_value(response.headers(), HEADER_SSEC_ALGORITHM),
            ssec_key_md5: get_header_value(response.headers(), HEADER_SSEC_KEY_MD5),
            server_side_encryption: get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION),
            server_side_encryption_key_id: get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID),
        };
        Ok(result)
    }
}

impl UploadPartCopyOutput {
    pub fn part_number(&self) -> isize {
        self.part_number
    }
    pub fn etag(&self) -> &str {
        &self.etag
    }
    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }
    pub fn copy_source_version_id(&self) -> &str {
        &self.copy_source_version_id
    }
    pub fn ssec_algorithm(&self) -> &str {
        &self.ssec_algorithm
    }
    pub fn ssec_key_md5(&self) -> &str {
        &self.ssec_key_md5
    }
    pub fn server_side_encryption(&self) -> &str {
        &self.server_side_encryption
    }
    pub fn server_side_encryption_key_id(&self) -> &str {
        &self.server_side_encryption_key_id
    }
}

#[derive(Debug, Clone, PartialEq, ListCommonQuery, GenericInput)]
pub struct ListMultipartUploadsInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) prefix: String,
    pub(crate) delimiter: String,
    pub(crate) key_marker: String,
    pub(crate) upload_id_marker: String,
    pub(crate) max_uploads: isize,
    pub(crate) encoding_type: String,
}

impl InputDescriptor for ListMultipartUploadsInput {
    fn operation(&self) -> &str {
        "ListMultipartUploads"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for ListMultipartUploadsInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(8);
        query.insert("uploads", "".to_string());
        set_list_common_query(&mut query, self);
        map_insert(&mut query, QUERY_KEY_MARKER, &self.key_marker);
        map_insert(&mut query, QUERY_UPLOAD_ID_MARKER, &self.upload_id_marker);
        if self.max_uploads >= 0 {
            query.insert(QUERY_MAX_UPLOADS, self.max_uploads.to_string());
        }
        request.query = Some(query);
        Ok(request)
    }
}

impl Default for ListMultipartUploadsInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            prefix: "".to_string(),
            delimiter: "".to_string(),
            key_marker: "".to_string(),
            upload_id_marker: "".to_string(),
            max_uploads: -1,
            encoding_type: "".to_string(),
        }
    }
}

impl ListMultipartUploadsInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key_marker(&self) -> &str {
        &self.key_marker
    }
    pub fn upload_id_marker(&self) -> &str {
        &self.upload_id_marker
    }
    pub fn max_uploads(&self) -> isize {
        self.max_uploads
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key_marker(&mut self, key_marker: impl Into<String>) {
        self.key_marker = key_marker.into();
    }
    pub fn set_upload_id_marker(&mut self, upload_id_marker: impl Into<String>) {
        self.upload_id_marker = upload_id_marker.into();
    }
    pub fn set_max_uploads(&mut self, max_uploads: isize) {
        self.max_uploads = max_uploads;
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct ListMultipartUploadsOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Bucket")]
    pub(crate) bucket: String,
    #[serde(default)]
    #[serde(rename = "Prefix")]
    pub(crate) prefix: String,
    #[serde(default)]
    #[serde(rename = "KeyMarker")]
    pub(crate) key_marker: String,
    #[serde(default)]
    #[serde(rename = "UploadIdMarker")]
    pub(crate) upload_id_marker: String,
    #[serde(default)]
    #[serde(rename = "MaxUploads")]
    pub(crate) max_uploads: isize,
    #[serde(default)]
    #[serde(rename = "Delimiter")]
    pub(crate) delimiter: String,
    #[serde(default)]
    #[serde(rename = "IsTruncated")]
    pub(crate) is_truncated: bool,
    #[serde(default)]
    #[serde(rename = "EncodingType")]
    pub(crate) encoding_type: String,
    #[serde(default)]
    #[serde(rename = "NextKeyMarker")]
    pub(crate) next_key_marker: String,
    #[serde(default)]
    #[serde(rename = "NextUploadIdMarker")]
    pub(crate) next_upload_id_marker: String,
    #[serde(default)]
    #[serde(rename = "CommonPrefixes")]
    pub(crate) common_prefixes: Vec<ListedCommonPrefix>,
    #[serde(default)]
    #[serde(rename = "Uploads")]
    pub(crate) uploads: Vec<ListedUpload>,
}

impl OutputParser for ListMultipartUploadsOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let mut result = parse_json::<Self>(response)?;
        for upload in &mut result.uploads {
            if let Some(x) = upload.initiated_string.take() {
                upload.initiated = parse_date_time_iso8601(&x)?;
            }
        }
        result.request_info = request_info;
        Ok(result)
    }
}

impl ListMultipartUploadsOutput {
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    pub fn key_marker(&self) -> &str {
        &self.key_marker
    }
    pub fn upload_id_marker(&self) -> &str {
        &self.upload_id_marker
    }
    pub fn max_uploads(&self) -> isize {
        self.max_uploads
    }
    pub fn delimiter(&self) -> &str {
        &self.delimiter
    }
    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }
    pub fn encoding_type(&self) -> &str {
        &self.encoding_type
    }
    pub fn next_key_marker(&self) -> &str {
        &self.next_key_marker
    }
    pub fn next_upload_id_marker(&self) -> &str {
        &self.next_upload_id_marker
    }
    pub fn common_prefixes(&self) -> &Vec<ListedCommonPrefix> {
        &self.common_prefixes
    }
    pub fn uploads(&self) -> &Vec<ListedUpload> {
        &self.uploads
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ListedUpload {
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "UploadId")]
    pub(crate) upload_id: String,
    #[serde(default)]
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
    #[serde(default)]
    #[serde(rename = "StorageClass")]
    pub(crate) storage_class: Option<StorageClassType>,
    #[serde(default)]
    #[serde(rename = "Initiated")]
    pub(crate) initiated_string: Option<String>,
    #[serde(skip)]
    pub(crate) initiated: Option<DateTime<Utc>>,
}

impl ListedUpload {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }
    pub fn initiated(&self) -> Option<DateTime<Utc>> {
        self.initiated
    }
}

#[derive(Debug, Clone, PartialEq, GenericInput)]
pub struct ListPartsInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) upload_id: String,
    pub(crate) part_number_marker: isize,
    pub(crate) max_parts: isize,
}

impl InputDescriptor for ListPartsInput {
    fn operation(&self) -> &str {
        "ListParts"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for ListPartsInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(8);
        set_upload_id(&mut query, &self.upload_id)?;
        if self.part_number_marker >= 0 {
            query.insert(QUERY_PART_NUMBER_MARKER, self.part_number_marker.to_string());
        }
        if self.max_parts >= 0 {
            query.insert(QUERY_MAX_PARTS, self.max_parts.to_string());
        }
        request.query = Some(query);
        Ok(request)
    }
}

impl Default for ListPartsInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            key: "".to_string(),
            upload_id: "".to_string(),
            part_number_marker: 0,
            max_parts: -1,
        }
    }
}

impl ListPartsInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>, upload_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.upload_id = upload_id.into();
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }
    pub fn part_number_marker(&self) -> isize {
        self.part_number_marker
    }
    pub fn max_parts(&self) -> isize {
        self.max_parts
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_upload_id(&mut self, upload_id: impl Into<String>) {
        self.upload_id = upload_id.into();
    }
    pub fn set_part_number_marker(&mut self, part_number_marker: isize) {
        self.part_number_marker = part_number_marker;
    }
    pub fn set_max_parts(&mut self, max_parts: isize) {
        self.max_parts = max_parts;
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct ListPartsOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Bucket")]
    pub(crate) bucket: String,
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "UploadId")]
    pub(crate) upload_id: String,
    #[serde(default)]
    #[serde(rename = "PartNumberMarker")]
    pub(crate) part_number_marker: isize,
    #[serde(default)]
    #[serde(rename = "MaxParts")]
    pub(crate) max_parts: isize,
    #[serde(default)]
    #[serde(rename = "IsTruncated")]
    pub(crate) is_truncated: bool,
    #[serde(default)]
    #[serde(rename = "NextPartNumberMarker")]
    pub(crate) next_part_number_marker: isize,
    #[serde(default)]
    #[serde(rename = "StorageClass")]
    pub(crate) storage_class: Option<StorageClassType>,
    #[serde(default)]
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
    #[serde(default)]
    #[serde(rename = "Parts")]
    pub(crate) parts: Vec<UploadedPart>,
}

impl OutputParser for ListPartsOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let mut result = parse_json::<Self>(response)?;
        for part in &mut result.parts {
            if let Some(x) = part.last_modified_string.take() {
                part.last_modified = parse_date_time_iso8601(&x)?;
            }
        }
        result.request_info = request_info;
        Ok(result)
    }
}

impl ListPartsOutput {
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn upload_id(&self) -> &str {
        &self.upload_id
    }
    pub fn part_number_marker(&self) -> isize {
        self.part_number_marker
    }
    pub fn max_parts(&self) -> isize {
        self.max_parts
    }
    pub fn is_truncated(&self) -> bool {
        self.is_truncated
    }
    pub fn next_part_number_marker(&self) -> isize {
        self.next_part_number_marker
    }
    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
    pub fn parts(&self) -> &Vec<UploadedPart> {
        &self.parts
    }
}