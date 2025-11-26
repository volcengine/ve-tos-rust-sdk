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

use crate::bucket::is_non_positive;
use crate::common::{DataTransferListener, DataTransferStatus, Grant, ListedCommonPrefix, Meta, Owner, RateLimiter, RequestInfo, RequestInfoTrait, TagSet, TempCopyResult};
use crate::common::{GenericInput, UserMeta};
use crate::config::ConfigHolder;
use crate::constant::{DEFAULT_READ_BUFFER_SIZE, HEADER_CACHE_CONTROL, HEADER_CALLBACK, HEADER_CONTENT_DISPOSITION, HEADER_CONTENT_ENCODING, HEADER_CONTENT_LANGUAGE, HEADER_CONTENT_LENGTH, HEADER_CONTENT_MD5, HEADER_CONTENT_RANGE, HEADER_CONTENT_SHA256, HEADER_CONTENT_TYPE, HEADER_COPY_SOURCE_VERSION_ID, HEADER_DELETE_MARKER, HEADER_DIRECTORY, HEADER_ETAG, HEADER_EXPIRATION, HEADER_EXPIRES, HEADER_FETCH_DETECT_MIME_TYPE, HEADER_FORBID_OVERWRITE, HEADER_HASH_CRC64ECMA, HEADER_IF_NONE_MATCH, HEADER_LAST_MODIFIED, HEADER_LAST_MODIFIED_NS, HEADER_METADATA_DIRECTIVE, HEADER_MODIFY_TIMESTAMP, HEADER_MODIFY_TIMESTAMP_NS, HEADER_NEXT_APPEND_OFFSET, HEADER_NOTIFICATION_CUSTOM_PARAMETERS, HEADER_OBJECT_EXPIRES, HEADER_OBJECT_TYPE, HEADER_PREFIX_META, HEADER_RANGE, HEADER_RECURSIVE_MKDIR, HEADER_REPLICATION_STATUS, HEADER_RESTORE, HEADER_RESTORE_EXPIRY_DAYS, HEADER_RESTORE_REQUEST_DATE, HEADER_RESTORE_TIER, HEADER_SERVER_SIDE_ENCRYPTION, HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID, HEADER_SSEC_ALGORITHM, HEADER_SSEC_KEY_MD5, HEADER_STORAGE_CLASS, HEADER_SYMLINK_BUCKET, HEADER_SYMLINK_TARGET, HEADER_SYMLINK_TARGET_SIZE, HEADER_TAGGING, HEADER_TAGGING_COUNT, HEADER_TAGGING_DIRECTIVE, HEADER_TRAFFIC_LIMIT, HEADER_VERSION_ID, HEADER_WEBSITE_REDIRECT_LOCATION, HEADER_X_IF_MATCH, QUERY_CONTINUATION_TOKEN, QUERY_FETCH_META, QUERY_FETCH_OWNER, QUERY_KEY_MARKER, QUERY_MARKER, QUERY_MAX_KEYS, QUERY_OFFSET, QUERY_PROCESS, QUERY_RECURSIVE, QUERY_SKIP_TRASH, QUERY_START_AFTER, QUERY_TASK_ID, QUERY_VERSION_ID, QUERY_VERSION_ID_MARKER, TRUE, UUID_NODE};
use crate::enumeration::HttpMethodType::{HttpMethodDelete, HttpMethodGet, HttpMethodHead, HttpMethodPost, HttpMethodPut};
use crate::enumeration::{ACLType, DocPreviewDstType, DocPreviewSrcType, MetadataDirectiveType, ObjectLockModeType, ReplicationStatusType, StorageClassType, TaggingDirectiveType, TierType};
use crate::error::{ErrorResponse, GenericError, TosError};
use crate::http::{HttpRequest, HttpResponse, RequestContext};
use crate::internal::{base64_md5, get_header_value_ref, map_insert, parse_json_by_buf, parse_response_string_by_buf, read_response, set_acl_header, set_copy_source_header, set_copy_source_if_condition_header, set_copy_source_ssec_header, set_data_process_query, set_http_basic_header, set_http_basic_header_for_fetch, set_if_match_header, set_misc_header, set_misc_header_for_fetch, set_object_lock_header, set_rewrite_response_query, set_sse_header, trans_meta, url_encode_with_safe};
use crate::internal::{get_header_value, get_header_value_from_str, get_header_value_str, get_header_value_url_decoded, get_map_value_str, parse_date_time_iso8601, parse_date_time_rfc1123, parse_json, set_callback_header, set_if_condition_header, set_list_common_query, set_ssec_header, InputDescriptor, InputTranslator, OutputParser};
use crate::reader::{BuildBufferReader, BuildFileReader, MultifunctionalReader};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use futures_core::Stream;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::ops::Add;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use ve_tos_generic::{AclHeader, CallbackHeader, CopySourceHeader, CopySourceIfConditionHeader, CopySourceSSecHeader, DataProcessQuery, GenericInput, HttpBasicHeader, IfConditionHeader, IfMatch, ListCommonQuery, MiscHeader, ObjectLock, RequestInfo, RewriteResponseQuery, SseHeader, SsecHeader};

pub trait ObjectAPI {
    fn copy_object(&self, input: &CopyObjectInput) -> Result<CopyObjectOutput, TosError>;
    fn delete_object(&self, input: &DeleteObjectInput) -> Result<DeleteObjectOutput, TosError>;
    fn delete_multi_objects(&self, input: &DeleteMultiObjectsInput) -> Result<DeleteMultiObjectsOutput, TosError>;
    fn get_object(&self, input: &GetObjectInput) -> Result<GetObjectOutput, TosError>;
    fn get_object_to_file(&self, input: &GetObjectToFileInput) -> Result<GetObjectToFileOutput, TosError>;
    fn get_object_acl(&self, input: &GetObjectACLInput) -> Result<GetObjectACLOutput, TosError>;
    fn head_object(&self, input: &HeadObjectInput) -> Result<HeadObjectOutput, TosError>;
    fn append_object<B>(&self, input: &AppendObjectInput<B>) -> Result<AppendObjectOutput, TosError>
    where
        B: Read + Send + 'static;
    fn append_object_from_buffer(&self, input: &AppendObjectFromBufferInput) -> Result<AppendObjectOutput, TosError>;
    #[deprecated]
    fn list_objects(&self, input: &ListObjectsInput) -> Result<ListObjectsOutput, TosError>;
    fn list_objects_type2(&self, input: &ListObjectsType2Input) -> Result<ListObjectsType2Output, TosError>;
    fn list_object_versions(&self, input: &ListObjectVersionsInput) -> Result<ListObjectVersionsOutput, TosError>;
    fn put_object<B>(&self, input: &PutObjectInput<B>) -> Result<PutObjectOutput, TosError>
    where
        B: Read + Send + 'static;
    fn put_object_from_buffer(&self, input: &PutObjectFromBufferInput) -> Result<PutObjectOutput, TosError>;
    fn put_object_from_file(&self, input: &PutObjectFromFileInput) -> Result<PutObjectOutput, TosError>;
    fn put_object_acl(&self, input: &PutObjectACLInput) -> Result<PutObjectACLOutput, TosError>;
    fn set_object_meta(&self, input: &SetObjectMetaInput) -> Result<SetObjectMetaOutput, TosError>;
}

pub trait ObjectContent {
    type Content: ?Sized;

    fn content(&mut self) -> Option<&mut Self::Content>;

    fn read_all(&mut self) -> Result<Vec<u8>, TosError>;
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
    CopySourceHeader,
    CopySourceSSecHeader,
    CopySourceIfConditionHeader,
    GenericInput
)]
pub struct CopyObjectInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,

    pub(crate) src_bucket: String,
    pub(crate) src_key: String,
    pub(crate) src_version_id: String,

    pub(crate) content_length: i64,
    pub(crate) cache_control: String,
    pub(crate) content_disposition: String,
    pub(crate) content_encoding: String,
    pub(crate) content_language: String,
    pub(crate) content_type: String,
    pub(crate) expires: Option<DateTime<Utc>>,

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
    pub(crate) server_side_encryption: String,
    pub(crate) server_side_encryption_key_id: String,

    pub(crate) acl: Option<ACLType>,
    pub(crate) grant_full_control: String,
    pub(crate) grant_read: String,
    pub(crate) grant_read_acp: String,
    pub(crate) grant_write: String,
    pub(crate) grant_write_acp: String,

    pub(crate) metadata_directive: Option<MetadataDirectiveType>,
    pub(crate) meta: HashMap<String, String>,
    pub(crate) website_redirect_location: String,
    pub(crate) storage_class: Option<StorageClassType>,
    pub(crate) traffic_limit: i64,
    pub(crate) forbid_overwrite: bool,
    pub(crate) if_match: String,
    pub(crate) if_none_match: String,

    pub(crate) tagging: String,
    pub(crate) tagging_directive: Option<TaggingDirectiveType>,
    pub(crate) object_expires: i64,
}

impl CopyObjectInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>,
               src_bucket: impl Into<String>, src_key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.src_bucket = src_bucket.into();
        input.src_key = src_key.into();
        input.content_length = -1;
        input.object_expires = -1;
        input
    }

    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>,
                               src_bucket: impl Into<String>, src_key: impl Into<String>, src_version_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.src_bucket = src_bucket.into();
        input.src_key = src_key.into();
        input.content_length = -1;
        input.src_version_id = src_version_id.into();
        input.object_expires = -1;
        input
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn metadata_directive(&self) -> &Option<MetadataDirectiveType> {
        &self.metadata_directive
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.meta
    }
    pub fn traffic_limit(&self) -> i64 {
        self.traffic_limit
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
    pub fn tagging(&self) -> &str {
        &self.tagging
    }

    pub fn tagging_directive(&self) -> &Option<TaggingDirectiveType> {
        &self.tagging_directive
    }
    pub fn object_expires(&self) -> i64 {
        self.object_expires
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_metadata_directive(&mut self, metadata_directive: impl Into<MetadataDirectiveType>) {
        self.metadata_directive = Some(metadata_directive.into());
    }
    pub fn set_meta(&mut self, meta: impl Into<HashMap<String, String>>) {
        self.meta = meta.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.traffic_limit = traffic_limit;
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
    pub fn set_tagging(&mut self, tagging: impl Into<String>) {
        self.tagging = tagging.into();
    }
    pub fn set_tagging_directive(&mut self, tagging_directive: impl Into<TaggingDirectiveType>) {
        self.tagging_directive = Some(tagging_directive.into());
    }
    pub fn set_object_expires(&mut self, object_expires: i64) {
        self.object_expires = object_expires;
    }
}

impl InputDescriptor for CopyObjectInput {
    fn operation(&self) -> &str {
        "CopyObject"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for CopyObjectInput {
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodPut;
        let header = &mut request.header;
        set_copy_source_header(header, self)?;
        set_copy_source_if_condition_header(header, self);
        set_http_basic_header(header, config_holder.disable_encoding_meta, self);
        set_sse_header(header, self)?;
        set_ssec_header(header, &self.server_side_encryption, self)?;
        set_copy_source_ssec_header(header, self)?;
        set_acl_header(header, self);
        request.meta = trans_meta(&self.meta, config_holder.disable_encoding_meta);
        set_misc_header(header, self);
        if let Some(metadata_directive) = &self.metadata_directive {
            header.insert(HEADER_METADATA_DIRECTIVE, metadata_directive.as_str().to_string());
        }
        if self.forbid_overwrite {
            header.insert(HEADER_FORBID_OVERWRITE, self.forbid_overwrite.to_string());
        }
        if self.traffic_limit > 0 {
            header.insert(HEADER_TRAFFIC_LIMIT, self.traffic_limit.to_string());
        }
        map_insert(header, HEADER_X_IF_MATCH, &self.if_match);
        map_insert(header, HEADER_IF_NONE_MATCH, &self.if_none_match);

        if self.tagging != "" {
            if let Some(tagging_directive) = &self.tagging_directive {
                header.insert(HEADER_TAGGING_DIRECTIVE, tagging_directive.as_str().to_string());
            }
            header.insert(HEADER_TAGGING, self.tagging.clone());
        }
        if self.object_expires >= 0 {
            header.insert(HEADER_OBJECT_EXPIRES, self.object_expires.to_string());
        }

        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct CopyObjectOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) etag: String,
    pub(crate) last_modified: Option<DateTime<Utc>>,
    pub(crate) copy_source_version_id: String,
    pub(crate) version_id: String,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key_md5: String,
    pub(crate) server_side_encryption: String,
    pub(crate) server_side_encryption_key_id: String,
}

impl CopyObjectOutput {
    pub fn etag(&self) -> &str {
        &self.etag
    }
    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }
    pub fn copy_source_version_id(&self) -> &str {
        &self.copy_source_version_id
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
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

impl OutputParser for CopyObjectOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let temp_result = parse_json::<TempCopyResult>(response)?;
        if temp_result.etag == "" {
            return Err(TosError::server_error_with_code(temp_result.code, temp_result.ec, temp_result.key, temp_result.message,
                                                        temp_result.host_id, temp_result.resource, request_info));
        }

        let mut result = Self {
            request_info: RequestInfo::default(),
            etag: temp_result.etag,
            last_modified: parse_date_time_iso8601(&temp_result.last_modified)?,
            copy_source_version_id: get_header_value(response.headers(), HEADER_COPY_SOURCE_VERSION_ID),
            version_id: get_header_value(response.headers(), HEADER_VERSION_ID),
            ssec_algorithm: get_header_value(response.headers(), HEADER_SSEC_ALGORITHM),
            ssec_key_md5: get_header_value(response.headers(), HEADER_SSEC_KEY_MD5),
            server_side_encryption: get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION),
            server_side_encryption_key_id: get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID),
        };

        result.request_info = request_info;
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct DeleteObjectInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) version_id: String,
    pub(crate) recursive: bool,
    pub(crate) skip_trash: bool,
    pub(crate) if_match: String,
}

impl DeleteObjectInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            version_id: "".to_string(),
            recursive: false,
            skip_trash: false,
            if_match: "".to_string(),
        }
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            version_id: version_id.into(),
            recursive: false,
            skip_trash: false,
            if_match: "".to_string(),
        }
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn recursive(&self) -> bool {
        self.recursive
    }
    pub fn if_match(&self) -> &str {
        &self.if_match
    }
    pub fn skip_trash(&self) -> bool {
        self.skip_trash
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }
    pub fn set_recursive(&mut self, recursive: bool) {
        self.recursive = recursive;
    }

    pub fn set_if_match(&mut self, if_match: impl Into<String>) {
        self.if_match = if_match.into();
    }
    pub fn set_skip_trash(&mut self, skip_trash: bool) {
        self.skip_trash = skip_trash;
    }
}

impl InputDescriptor for DeleteObjectInput {
    fn operation(&self) -> &str {
        "DeleteObject"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for DeleteObjectInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodDelete;
        map_insert(&mut request.header, HEADER_X_IF_MATCH, &self.if_match);
        let mut query = HashMap::with_capacity(3);
        map_insert(&mut query, QUERY_VERSION_ID, &self.version_id);
        if self.recursive {
            query.insert(QUERY_RECURSIVE, self.recursive.to_string());
        }
        if self.skip_trash {
            query.insert(QUERY_SKIP_TRASH, self.skip_trash.to_string());
        }
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteObjectOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) delete_marker: bool,
    pub(crate) version_id: String,
}

impl DeleteObjectOutput {
    pub fn delete_marker(&self) -> bool {
        self.delete_marker
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
}

impl OutputParser for DeleteObjectOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        let delete_marker = get_header_value_ref(response.headers(), HEADER_DELETE_MARKER) == TRUE;
        Ok(Self {
            request_info,
            delete_marker,
            version_id,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct DeleteMultiObjectsInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Objects")]
    pub(crate) objects: Vec<ObjectTobeDeleted>,
    #[serde(rename = "Quiet")]
    pub(crate) quiet: bool,
    #[serde(skip)]
    pub(crate) recursive: bool,
    #[serde(skip)]
    pub(crate) skip_trash: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct ObjectTobeDeleted {
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(rename = "VersionId")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) version_id: String,
}

impl ObjectTobeDeleted {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into(), version_id: "".to_string() }
    }
    pub fn new_with_version_id(key: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self { key: key.into(), version_id: version_id.into() }
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }
}


impl DeleteMultiObjectsInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            objects: vec![],
            quiet: false,
            recursive: false,
            skip_trash: false,
        }
    }
    pub fn new_with_objects(bucket: impl Into<String>, objects: impl Into<Vec<ObjectTobeDeleted>>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            objects: objects.into(),
            quiet: false,
            recursive: false,
            skip_trash: false,
        }
    }

    pub fn add_object(&mut self, object: impl Into<ObjectTobeDeleted>) {
        self.objects.push(object.into());
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn objects(&self) -> &Vec<ObjectTobeDeleted> {
        &self.objects
    }
    pub fn quiet(&self) -> bool {
        self.quiet
    }
    pub fn recursive(&self) -> bool {
        self.recursive
    }

    pub fn skip_trash(&self) -> bool {
        self.skip_trash
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_objects(&mut self, objects: impl Into<Vec<ObjectTobeDeleted>>) {
        self.objects = objects.into();
    }
    pub fn set_quiet(&mut self, quiet: bool) {
        self.quiet = quiet;
    }

    pub fn set_recursive(&mut self, recursive: bool) {
        self.recursive = recursive;
    }

    pub fn set_skip_trash(&mut self, skip_trash: bool) {
        self.skip_trash = skip_trash;
    }
}

impl InputDescriptor for DeleteMultiObjectsInput {
    fn operation(&self) -> &str {
        "DeleteMultiObjects"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for DeleteMultiObjectsInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.objects.len() == 0 {
            return Err(TosError::client_error("empty objects for delete"));
        }

        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_bucket()?;
                request.method = HttpMethodPost;
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());

                let mut query = HashMap::with_capacity(3);
                if self.recursive {
                    query.insert(QUERY_RECURSIVE, self.recursive.to_string());
                }
                if self.skip_trash {
                    query.insert(QUERY_SKIP_TRASH, self.skip_trash.to_string());
                }
                query.insert("delete", "".to_string());
                request.query = Some(query);
                Ok(request)
            }
        }
    }
}


#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct DeleteMultiObjectsOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Deleted")]
    pub(crate) deleted: Vec<Deleted>,
    #[serde(default)]
    #[serde(rename = "Error")]
    pub(crate) error: Vec<DeleteError>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Deleted {
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "VersionId")]
    pub(crate) version_id: String,
    #[serde(default)]
    #[serde(rename = "DeleteMarker")]
    pub(crate) delete_marker: bool,
    #[serde(default)]
    #[serde(rename = "DeleteMarkerVersionId")]
    pub(crate) delete_marker_version_id: String,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct DeleteError {
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "VersionId")]
    pub(crate) version_id: String,
    #[serde(default)]
    #[serde(rename = "Code")]
    pub(crate) code: String,
    #[serde(default)]
    #[serde(rename = "Message")]
    pub(crate) message: String,
}

impl Deleted {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn delete_marker(&self) -> bool {
        self.delete_marker
    }
    pub fn delete_marker_version_id(&self) -> &str {
        &self.delete_marker_version_id
    }
}


impl DeleteError {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn code(&self) -> &str {
        &self.code
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl DeleteMultiObjectsOutput {
    pub fn deleted(&self) -> &Vec<Deleted> {
        &self.deleted
    }
    pub fn error(&self) -> &Vec<DeleteError> {
        &self.error
    }
}

impl OutputParser for DeleteMultiObjectsOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let mut result = parse_json::<Self>(response)?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[derive(
    Debug,
    Clone,
    IfConditionHeader,
    SsecHeader,
    RewriteResponseQuery,
    DataProcessQuery,
    GenericInput
)]
pub struct GetObjectInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) version_id: String,
    pub(crate) if_match: String,
    pub(crate) if_modified_since: Option<DateTime<Utc>>,
    pub(crate) if_none_match: String,
    pub(crate) if_unmodified_since: Option<DateTime<Utc>>,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key: String,
    pub(crate) ssec_key_md5: String,

    pub(crate) response_cache_control: String,
    pub(crate) response_content_disposition: String,
    pub(crate) response_content_encoding: String,
    pub(crate) response_content_language: String,
    pub(crate) response_content_type: String,
    pub(crate) response_expires: Option<DateTime<Utc>>,

    pub(crate) range_start: i64,
    pub(crate) range_end: i64,

    pub(crate) range: String,
    pub(crate) traffic_limit: i64,

    pub(crate) process: String,

    pub(crate) doc_page: isize,
    pub(crate) src_type: Option<DocPreviewSrcType>,
    pub(crate) dst_type: Option<DocPreviewDstType>,

    pub(crate) save_bucket: String,
    pub(crate) save_object: String,
    pub(crate) rate_limiter: Option<Arc<RateLimiter>>,
    pub(crate) data_transfer_listener: Option<Sender<DataTransferStatus>>,
    pub(crate) async_data_transfer_listener: Option<async_channel::Sender<DataTransferStatus>>,
}

impl InputDescriptor for GetObjectInput {
    fn operation(&self) -> &str {
        "GetObject"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for GetObjectInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodGet;
        if let Some(rl) = self.rate_limiter() {
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
        set_if_condition_header(header, self);
        set_ssec_header(header, "", self)?;
        if self.range != "" {
            if !self.range.starts_with("bytes=") {
                return Err(TosError::client_error("invalid range format"));
            }
            header.insert(HEADER_RANGE, self.range.clone());
        } else if self.range_start >= 0 && self.range_end >= 0 && self.range_start <= self.range_end {
            header.insert(HEADER_RANGE, format!("bytes={}-{}", self.range_start, self.range_end));
        }
        if self.traffic_limit > 0 {
            header.insert(HEADER_TRAFFIC_LIMIT, self.traffic_limit.to_string());
        }

        let mut query = HashMap::with_capacity(16);
        map_insert(&mut query, QUERY_VERSION_ID, &self.version_id);
        set_rewrite_response_query(&mut query, self);
        set_data_process_query(&mut query, self);

        request.query = Some(query);
        Ok(request)
    }
}

impl Default for GetObjectInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            key: "".to_string(),
            version_id: "".to_string(),
            if_match: "".to_string(),
            if_modified_since: None,
            if_none_match: "".to_string(),
            if_unmodified_since: None,
            ssec_algorithm: "".to_string(),
            ssec_key: "".to_string(),
            ssec_key_md5: "".to_string(),
            response_cache_control: "".to_string(),
            response_content_disposition: "".to_string(),
            response_content_encoding: "".to_string(),
            response_content_language: "".to_string(),
            response_content_type: "".to_string(),
            response_expires: None,
            range_start: -1,
            range_end: -1,
            range: "".to_string(),
            traffic_limit: 0,
            process: "".to_string(),
            doc_page: 0,
            src_type: None,
            dst_type: None,
            save_bucket: "".to_string(),
            save_object: "".to_string(),
            rate_limiter: None,
            data_transfer_listener: None,
            async_data_transfer_listener: None,
        }
    }
}

impl DataTransferListener for GetObjectInput {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>> {
        &self.data_transfer_listener
    }

    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>) {
        self.data_transfer_listener = Some(listener.into());
    }
}

impl GetObjectInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.version_id = version_id.into();
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn range_start(&self) -> i64 {
        self.range_start
    }
    pub fn range_end(&self) -> i64 {
        self.range_end
    }
    pub fn range(&self) -> &str {
        &self.range
    }
    pub fn traffic_limit(&self) -> i64 {
        self.traffic_limit
    }
    pub fn rate_limiter(&self) -> &Option<Arc<RateLimiter>> {
        &self.rate_limiter
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }
    pub fn set_range_start(&mut self, range_start: i64) {
        self.range_start = range_start;
    }
    pub fn set_range_end(&mut self, range_end: i64) {
        self.range_end = range_end;
    }
    pub fn set_range(&mut self, range: impl Into<String>) {
        self.range = range.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.traffic_limit = traffic_limit;
    }
    pub fn set_rate_limiter(&mut self, rate_limiter: impl Into<Arc<RateLimiter>>) {
        self.rate_limiter = Some(rate_limiter.into());
    }
}

#[derive(Default)]
pub struct GetObjectOutput {
    pub(crate) content_range: String,
    pub(crate) content: Option<MultifunctionalReader<HttpResponse>>,
    pub(crate) async_content: Option<MultifunctionalReader<Box<dyn Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Unpin>>>,
    pub(crate) head_object_output: HeadObjectOutput,
}

unsafe impl Sync for GetObjectOutput {}

impl Debug for GetObjectOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {})", self.head_object_output, self.content_range)
    }
}

impl OutputParser for GetObjectOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, _: &mut HttpResponse, _: RequestInfo, _: Meta) -> Result<Self, TosError> {
        Err(TosError::client_error("unimplemented"))
    }

    fn parse<B>(request: HttpRequest<B>, response: HttpResponse, request_info: RequestInfo, meta: Meta) -> Result<Self, TosError> {
        let head_object_output = HeadObjectOutput::parse_by_header(response.headers(), request_info, meta)?;
        let content_range = get_header_value(response.headers(), HEADER_CONTENT_RANGE);
        let mut target_crc64 = None;
        if request.enable_crc && !request.header.contains_key(HEADER_RANGE) &&
            (request.query.is_none() || !request.query.as_ref().unwrap().contains_key(QUERY_PROCESS)) {
            target_crc64 = Some(head_object_output.hash_crc64ecma);
        }
        let mut crc64 = None;
        if target_crc64.is_some() {
            crc64 = Some(Arc::new(AtomicU64::new(0)));
        }
        let mut reader = MultifunctionalReader::with_target_crc64(response, crc64, head_object_output.content_length,
                                                                  &request, target_crc64);

        if let Some(ref rc) = request.request_context {
            if let Some(ref rl) = rc.rate_limiter {
                reader.set_rate_limiter(rl.clone());
            }
            if let Some(ref dts) = rc.data_transfer_listener {
                reader.set_data_transfer_listener(dts.clone());
                reader.inner.operation = request.operation.to_string();
                reader.inner.bucket = request.bucket.to_string();
                reader.inner.key = request.key.to_string();
                reader.inner.retry_count = request.retry_count;
            }
        }

        Ok(Self {
            content_range,
            content: Some(reader),
            async_content: None,
            head_object_output,
        })
    }
}

impl Read for GetObjectOutput {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.content.as_mut() {
            None => {
                Err(std::io::Error::new(ErrorKind::Other, "empty content"))
            }
            Some(content) => {
                content.read(buf)
            }
        }
    }
}

impl ObjectContent for GetObjectOutput {
    type Content = dyn Read;

    fn content(&mut self) -> Option<&mut Self::Content> {
        match self.content.as_mut() {
            None => None,
            Some(x) => Some(x as &mut dyn Read),
        }
    }

    fn read_all(&mut self) -> Result<Vec<u8>, TosError> {
        let x = self.content_length();
        if x == 0 {
            return Ok(vec![]);
        }
        let mut buf;
        if x > 0 {
            buf = Vec::with_capacity(x as usize);
        } else {
            buf = Vec::new();
        }
        match self.content.as_mut() {
            None => Err(TosError::client_error("empty content")),
            Some(r) => {
                match r.read_to_end(&mut buf) {
                    Err(e) => Err(TosError::client_error_with_cause("read error", GenericError::IoError(e.to_string()))),
                    Ok(_) => Ok(buf),
                }
            }
        }
    }
}

impl RequestInfoTrait for GetObjectOutput {
    fn request_id(&self) -> &str {
        &self.head_object_output.request_info.request_id
    }

    fn id2(&self) -> &str {
        &self.head_object_output.request_info.id2
    }

    fn status_code(&self) -> isize {
        self.head_object_output.request_info.status_code
    }

    fn header(&self) -> &HashMap<String, String> {
        &self.head_object_output.request_info.header
    }
}

impl GetObjectOutput {
    pub fn request_id(&self) -> &str {
        &self.head_object_output.request_info.request_id
    }

    pub fn id2(&self) -> &str {
        &self.head_object_output.request_info.id2
    }

    pub fn status_code(&self) -> isize {
        self.head_object_output.request_info.status_code
    }

    pub fn header(&self) -> &HashMap<String, String> {
        &self.head_object_output.request_info.header
    }
    pub fn content_range(&self) -> &str {
        &self.content_range
    }
    pub fn etag(&self) -> &str {
        &self.head_object_output.etag
    }
    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.head_object_output.last_modified
    }
    pub fn last_modified_timestamp(&self) -> Option<DateTime<Utc>> {
        self.head_object_output.last_modify_timestamp
    }
    pub fn delete_marker(&self) -> bool {
        self.head_object_output.delete_marker
    }
    pub fn ssec_algorithm(&self) -> &str {
        &self.head_object_output.ssec_algorithm
    }
    pub fn ssec_key_md5(&self) -> &str {
        &self.head_object_output.ssec_key_md5
    }
    pub fn version_id(&self) -> &str {
        &self.head_object_output.version_id
    }
    pub fn website_redirect_location(&self) -> &str {
        &self.head_object_output.website_redirect_location
    }
    pub fn object_type(&self) -> &str {
        &self.head_object_output.object_type
    }
    pub fn hash_crc64ecma(&self) -> u64 {
        self.head_object_output.hash_crc64ecma
    }
    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.head_object_output.storage_class
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.head_object_output.meta
    }
    pub fn content_length(&self) -> i64 {
        self.head_object_output.content_length
    }
    pub fn cache_control(&self) -> &str {
        &self.head_object_output.cache_control
    }
    pub fn content_disposition(&self) -> &str {
        &self.head_object_output.content_disposition
    }
    pub fn content_encoding(&self) -> &str {
        &self.head_object_output.content_encoding
    }
    pub fn content_language(&self) -> &str {
        &self.head_object_output.content_language
    }
    pub fn content_type(&self) -> &str {
        &self.head_object_output.content_type
    }
    pub fn expires(&self) -> Option<DateTime<Utc>> {
        self.head_object_output.expires
    }
    pub fn restore_info(&self) -> &Option<RestoreInfo> {
        &self.head_object_output.restore_info
    }
    pub fn server_side_encryption(&self) -> &str {
        &self.head_object_output.server_side_encryption
    }
    pub fn server_side_encryption_key_id(&self) -> &str {
        &self.head_object_output.server_side_encryption_key_id
    }
    pub fn replication_status(&self) -> &Option<ReplicationStatusType> {
        &self.head_object_output.replication_status
    }
    pub fn tagging_count(&self) -> isize {
        self.head_object_output.tagging_count
    }
    pub fn expiration(&self) -> &str {
        &self.head_object_output.expiration
    }
    pub fn is_directory(&self) -> bool {
        self.head_object_output.is_directory
    }
}

#[derive(
    Debug,
    Clone,
    IfConditionHeader,
    SsecHeader,
    RewriteResponseQuery,
    DataProcessQuery,
    GenericInput
)]
#[use_inner]
pub struct GetObjectToFileInput {
    pub(crate) inner: GetObjectInput,
    pub(crate) file_path: String,
}


impl InputDescriptor for GetObjectToFileInput {
    fn operation(&self) -> &str {
        "GetObjectToFile"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.inner.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.inner.key)
    }
}

impl<B> InputTranslator<B> for GetObjectToFileInput {
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.file_path == "" {
            return Err(TosError::client_error("empty file path"));
        }
        let mut request = self.inner.trans(config_holder)?;
        if request.request_context.is_some() {
            request.request_context.as_mut().unwrap().file_path = &self.file_path
        } else {
            let mut rc = RequestContext::default();
            rc.file_path = &self.file_path;
            request.request_context = Some(rc);
        }
        Ok(request)
    }
}

impl Default for GetObjectToFileInput {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            file_path: "".to_string(),
        }
    }
}

impl DataTransferListener for GetObjectToFileInput {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>> {
        &self.inner.data_transfer_listener
    }

    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>) {
        self.inner.data_transfer_listener = Some(listener.into());
    }
}

impl GetObjectToFileInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>, file_path: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.file_path = file_path.into();
        input
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>, file_path: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.version_id = version_id.into();
        input.file_path = file_path.into();
        input
    }
    pub fn bucket(&self) -> &str {
        &self.inner.bucket
    }
    pub fn key(&self) -> &str {
        &self.inner.key
    }
    pub fn version_id(&self) -> &str {
        &self.inner.version_id
    }
    pub fn range_start(&self) -> i64 {
        self.inner.range_start
    }
    pub fn range_end(&self) -> i64 {
        self.inner.range_end
    }
    pub fn range(&self) -> &str {
        &self.inner.range
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
    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.inner.version_id = version_id.into();
    }
    pub fn set_range_start(&mut self, range_start: i64) {
        self.inner.range_start = range_start;
    }
    pub fn set_range_end(&mut self, range_end: i64) {
        self.inner.range_end = range_end;
    }
    pub fn set_range(&mut self, range: impl Into<String>) {
        self.inner.range = range.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.inner.traffic_limit = traffic_limit;
    }
    pub fn set_rate_limiter(&mut self, rate_limiter: impl Into<Arc<RateLimiter>>) {
        self.inner.rate_limiter = Some(rate_limiter.into());
    }
}
#[derive(Default)]
pub struct GetObjectToFileOutput {
    pub(crate) content_range: String,
    pub(crate) head_object_output: HeadObjectOutput,
}

impl Debug for GetObjectToFileOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {})", self.head_object_output, self.content_range)
    }
}

impl OutputParser for GetObjectToFileOutput {
    fn parse_by_ref<B>(request: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, meta: Meta) -> Result<Self, TosError> {
        let head_object_output = HeadObjectOutput::parse_by_header(response.headers(), request_info, meta)?;
        let content_range = get_header_value(response.headers(), HEADER_CONTENT_RANGE);
        let mut target_crc64 = None;
        if request.enable_crc && !request.header.contains_key(HEADER_RANGE) &&
            (request.query.is_none() || !request.query.as_ref().unwrap().contains_key(QUERY_PROCESS)) {
            target_crc64 = Some(head_object_output.hash_crc64ecma);
        }
        let mut crc64 = None;
        if target_crc64.is_some() {
            crc64 = Some(Arc::new(AtomicU64::new(0)));
        }
        let mut reader = MultifunctionalReader::with_target_crc64(response, crc64, head_object_output.content_length,
                                                                  &request, target_crc64);
        if let Some(ref rc) = request.request_context {
            if let Some(ref rl) = rc.rate_limiter {
                reader.set_rate_limiter(rl.clone());
            }
            if let Some(ref dts) = rc.data_transfer_listener {
                reader.set_data_transfer_listener(dts.clone());
                reader.inner.operation = request.operation.to_string();
                reader.inner.bucket = request.bucket.to_string();
                reader.inner.key = request.key.to_string();
                reader.inner.retry_count = request.retry_count;
            }
        }
        let file_path = &request.request_context.as_ref().unwrap().file_path;
        let path = Path::new(file_path);
        match path.parent() {
            None => return Err(TosError::client_error(format!("cannot get parent for path {}", file_path))),
            Some(p) => {
                if !p.exists() {
                    if let Err(e) = fs::create_dir_all(p) {
                        return Err(TosError::client_error_with_cause(format!("create dir for parent {} error", p.display()),
                                                                     GenericError::IoError(e.to_string())));
                    }
                }
            }
        }
        let final_file_path;
        if path.exists() && path.is_dir() {
            final_file_path = path.join(request.key);
        } else {
            final_file_path = path.to_path_buf();
        }

        let temp_file_path = final_file_path.parent().unwrap().join(Uuid::now_v1(&UUID_NODE).to_string());
        match File::options().write(true).truncate(true).create(true).open(temp_file_path.clone()) {
            Err(e) => {
                return Err(TosError::client_error_with_cause("open file to write error", GenericError::IoError(e.to_string())))
            }
            Ok(mut fd) => {
                let mut data = [0u8; DEFAULT_READ_BUFFER_SIZE];
                loop {
                    match reader.read(&mut data) {
                        Err(re) => {
                            if re.kind() == ErrorKind::Interrupted {
                                continue;
                            }
                            let _ = fs::remove_file(temp_file_path);
                            return Err(TosError::client_error_with_cause("read content to write error", GenericError::IoError(re.to_string())));
                        }
                        Ok(n) => {
                            if n == 0 {
                                break;
                            }
                            if let Err(we) = fd.write_all(&data[..n]) {
                                let _ = fs::remove_file(temp_file_path);
                                return Err(TosError::client_error_with_cause("write data to file error", GenericError::IoError(we.to_string())));
                            }
                        }
                    }
                }
            }
        }
        if let Err(re) = fs::rename(temp_file_path.clone(), final_file_path) {
            let _ = fs::remove_file(temp_file_path);
            return Err(TosError::client_error_with_cause("rename file error", GenericError::IoError(re.to_string())));
        }

        Ok(Self {
            content_range,
            head_object_output,
        })
    }
}

impl RequestInfoTrait for GetObjectToFileOutput {
    fn request_id(&self) -> &str {
        &self.head_object_output.request_info.request_id
    }

    fn id2(&self) -> &str {
        &self.head_object_output.request_info.id2
    }

    fn status_code(&self) -> isize {
        self.head_object_output.request_info.status_code
    }

    fn header(&self) -> &HashMap<String, String> {
        &self.head_object_output.request_info.header
    }
}

impl GetObjectToFileOutput {
    pub fn request_id(&self) -> &str {
        &self.head_object_output.request_info.request_id
    }

    pub fn id2(&self) -> &str {
        &self.head_object_output.request_info.id2
    }

    pub fn status_code(&self) -> isize {
        self.head_object_output.request_info.status_code
    }

    pub fn header(&self) -> &HashMap<String, String> {
        &self.head_object_output.request_info.header
    }
    pub fn content_range(&self) -> &str {
        &self.content_range
    }
    pub fn etag(&self) -> &str {
        &self.head_object_output.etag
    }
    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.head_object_output.last_modified
    }
    pub fn last_modify_timestamp(&self) -> Option<DateTime<Utc>> {
        self.head_object_output.last_modify_timestamp
    }
    pub fn delete_marker(&self) -> bool {
        self.head_object_output.delete_marker
    }
    pub fn ssec_algorithm(&self) -> &str {
        &self.head_object_output.ssec_algorithm
    }
    pub fn ssec_key_md5(&self) -> &str {
        &self.head_object_output.ssec_key_md5
    }
    pub fn version_id(&self) -> &str {
        &self.head_object_output.version_id
    }
    pub fn website_redirect_location(&self) -> &str {
        &self.head_object_output.website_redirect_location
    }
    pub fn object_type(&self) -> &str {
        &self.head_object_output.object_type
    }
    pub fn hash_crc64ecma(&self) -> u64 {
        self.head_object_output.hash_crc64ecma
    }
    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.head_object_output.storage_class
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.head_object_output.meta
    }
    pub fn content_length(&self) -> i64 {
        self.head_object_output.content_length
    }
    pub fn cache_control(&self) -> &str {
        &self.head_object_output.cache_control
    }
    pub fn content_disposition(&self) -> &str {
        &self.head_object_output.content_disposition
    }
    pub fn content_encoding(&self) -> &str {
        &self.head_object_output.content_encoding
    }
    pub fn content_language(&self) -> &str {
        &self.head_object_output.content_language
    }
    pub fn content_type(&self) -> &str {
        &self.head_object_output.content_type
    }
    pub fn expires(&self) -> Option<DateTime<Utc>> {
        self.head_object_output.expires
    }
    pub fn restore_info(&self) -> &Option<RestoreInfo> {
        &self.head_object_output.restore_info
    }
    pub fn server_side_encryption(&self) -> &str {
        &self.head_object_output.server_side_encryption
    }
    pub fn server_side_encryption_key_id(&self) -> &str {
        &self.head_object_output.server_side_encryption_key_id
    }
    pub fn replication_status(&self) -> &Option<ReplicationStatusType> {
        &self.head_object_output.replication_status
    }
    pub fn tagging_count(&self) -> isize {
        self.head_object_output.tagging_count
    }
}

#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct GetObjectACLInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) version_id: String,
}

impl GetObjectACLInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            version_id: "".to_string(),
        }
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            version_id: version_id.into(),
        }
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }
}

impl InputDescriptor for GetObjectACLInput {
    fn operation(&self) -> &str {
        "GetObjectACL"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for GetObjectACLInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(2);
        query.insert("acl", "".to_string());
        map_insert(&mut query, QUERY_VERSION_ID, &self.version_id);
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetObjectACLOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(skip)]
    pub(crate) version_id: String,
    #[serde(default)]
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
    #[serde(default)]
    #[serde(rename = "Grants")]
    pub(crate) grants: Vec<Grant>,
    #[serde(default)]
    #[serde(rename = "BucketOwnerEntrusted")]
    pub(crate) bucket_owner_entrusted: bool,
    #[serde(default)]
    #[serde(rename = "IsDefault")]
    pub(crate) is_default: bool,
}

impl GetObjectACLOutput {
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
    pub fn grants(&self) -> &Vec<Grant> {
        &self.grants
    }
    pub fn bucket_owner_entrusted(&self) -> bool {
        self.bucket_owner_entrusted
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }
}

impl OutputParser for GetObjectACLOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let mut result = parse_json::<Self>(response)?;
        result.version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        result.request_info = request_info;
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Default, IfConditionHeader, SsecHeader, GenericInput)]
pub struct HeadObjectInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) version_id: String,
    pub(crate) if_match: String,
    pub(crate) if_modified_since: Option<DateTime<Utc>>,
    pub(crate) if_none_match: String,
    pub(crate) if_unmodified_since: Option<DateTime<Utc>>,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key: String,
    pub(crate) ssec_key_md5: String,
}

impl HeadObjectInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.version_id = version_id.into();
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }
}

impl InputDescriptor for HeadObjectInput {
    fn operation(&self) -> &str {
        "HeadObject"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for HeadObjectInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodHead;
        if self.version_id != "" {
            request.query = Some(HashMap::from([(QUERY_VERSION_ID, self.version_id.clone())]));
        }
        let header = &mut request.header;
        set_if_condition_header(header, self);
        set_ssec_header(header, "", self)?;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct HeadObjectOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) etag: String,
    pub(crate) last_modified: Option<DateTime<Utc>>,
    pub(crate) last_modify_timestamp: Option<DateTime<Utc>>,
    pub(crate) delete_marker: bool,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key_md5: String,
    pub(crate) version_id: String,
    pub(crate) website_redirect_location: String,
    pub(crate) object_type: String,
    pub(crate) hash_crc64ecma: u64,
    pub(crate) storage_class: Option<StorageClassType>,
    pub(crate) meta: HashMap<String, String>,

    pub(crate) content_length: i64,
    pub(crate) cache_control: String,
    pub(crate) content_disposition: String,
    pub(crate) content_encoding: String,
    pub(crate) content_language: String,
    pub(crate) content_type: String,
    pub(crate) expires: Option<DateTime<Utc>>,
    pub(crate) restore_info: Option<RestoreInfo>,

    pub(crate) server_side_encryption: String,
    pub(crate) server_side_encryption_key_id: String,
    pub(crate) replication_status: Option<ReplicationStatusType>,
    pub(crate) tagging_count: isize,
    pub(crate) symlink_target_size: i64,
    pub(crate) expiration: String,
    pub(crate) is_directory: bool,
}

impl OutputParser for HeadObjectOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, meta: Meta) -> Result<Self, TosError> {
        Self::parse_by_header(response.headers(), request_info, meta)
    }
}

impl HeadObjectOutput {
    pub fn etag(&self) -> &str {
        &self.etag
    }
    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }
    pub fn last_modify_timestamp(&self) -> Option<DateTime<Utc>> {
        self.last_modify_timestamp
    }
    pub fn delete_marker(&self) -> bool {
        self.delete_marker
    }
    pub fn ssec_algorithm(&self) -> &str {
        &self.ssec_algorithm
    }
    pub fn ssec_key_md5(&self) -> &str {
        &self.ssec_key_md5
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn website_redirect_location(&self) -> &str {
        &self.website_redirect_location
    }
    pub fn object_type(&self) -> &str {
        &self.object_type
    }
    pub fn hash_crc64ecma(&self) -> u64 {
        self.hash_crc64ecma
    }
    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.meta
    }
    pub fn content_length(&self) -> i64 {
        self.content_length
    }
    pub fn cache_control(&self) -> &str {
        &self.cache_control
    }
    pub fn content_disposition(&self) -> &str {
        &self.content_disposition
    }
    pub fn content_encoding(&self) -> &str {
        &self.content_encoding
    }
    pub fn content_language(&self) -> &str {
        &self.content_language
    }
    pub fn content_type(&self) -> &str {
        &self.content_type
    }
    pub fn expires(&self) -> Option<DateTime<Utc>> {
        self.expires
    }
    pub fn restore_info(&self) -> &Option<RestoreInfo> {
        &self.restore_info
    }
    pub fn server_side_encryption(&self) -> &str {
        &self.server_side_encryption
    }
    pub fn server_side_encryption_key_id(&self) -> &str {
        &self.server_side_encryption_key_id
    }
    pub fn replication_status(&self) -> &Option<ReplicationStatusType> {
        &self.replication_status
    }
    pub fn tagging_count(&self) -> isize {
        self.tagging_count
    }
    pub fn symlink_target_size(&self) -> i64 {
        self.symlink_target_size
    }
    pub fn expiration(&self) -> &str {
        &self.expiration
    }

    pub fn is_directory(&self) -> bool {
        self.is_directory
    }
    pub(crate) fn parse_by_header(header: &HeaderMap, request_info: RequestInfo, meta: Meta) -> Result<Self, TosError> {
        let mut result = Self::default();
        result.etag = get_header_value(header, HEADER_ETAG);
        result.last_modified = parse_date_time_rfc1123(&get_header_value(header, HEADER_LAST_MODIFIED))?;

        let ns = get_header_value_from_str::<u64>(header, HEADER_LAST_MODIFIED_NS, 0)?;
        if ns > 0 {
            if let Some(last_modified) = &result.last_modified {
                result.last_modify_timestamp = Some(last_modified.add(Duration::from_nanos(ns)));
            }
        }

        result.delete_marker = get_header_value_str(header, HEADER_DELETE_MARKER) == TRUE;
        result.ssec_algorithm = get_header_value(header, HEADER_SSEC_ALGORITHM);
        result.ssec_key_md5 = get_header_value(header, HEADER_SSEC_KEY_MD5);
        result.version_id = get_header_value(header, HEADER_VERSION_ID);
        result.website_redirect_location = get_header_value(header, HEADER_WEBSITE_REDIRECT_LOCATION);
        result.object_type = get_header_value(header, HEADER_OBJECT_TYPE);
        result.hash_crc64ecma = get_header_value_from_str::<u64>(header, HEADER_HASH_CRC64ECMA, 0)?;
        result.content_length = get_header_value_from_str::<i64>(header, HEADER_CONTENT_LENGTH, -1)?;
        result.cache_control = get_header_value(header, HEADER_CACHE_CONTROL);
        result.content_disposition = get_header_value_url_decoded(header, HEADER_CONTENT_DISPOSITION);
        result.content_encoding = get_header_value(header, HEADER_CONTENT_ENCODING);
        result.content_language = get_header_value(header, HEADER_CONTENT_LANGUAGE);
        result.content_type = get_header_value(header, HEADER_CONTENT_TYPE);
        result.expires = parse_date_time_rfc1123(&get_header_value(header, HEADER_EXPIRES))?;
        let restore = get_header_value(header, HEADER_RESTORE);
        let restore_trim = restore.trim();
        if restore_trim != "" {
            if restore_trim == "ongoing-request=\"true\"" {
                result.restore_info = Some(RestoreInfo {
                    restore_status: RestoreStatus {
                        ongoing_request: true,
                        expiry_date: None,
                    },
                    restore_param: Some(RestoreParam {
                        request_date: parse_date_time_rfc1123(&get_header_value(header, HEADER_RESTORE_REQUEST_DATE))?,
                        expiry_days: get_header_value_from_str::<isize>(header, HEADER_RESTORE_EXPIRY_DAYS, 0)?,
                        tier: TierType::from(get_header_value_str(header, HEADER_RESTORE_TIER)),
                    }),
                });
            } else {
                let pattern = "ongoing-request=\"false\", expiry-date=\"";
                if let Some(idx) = restore_trim.find(pattern) {
                    let mut expiry_date = &restore_trim[idx..];
                    if expiry_date.len() > 0 && &expiry_date[expiry_date.len() - 1..] == "\"" {
                        expiry_date = &expiry_date[..expiry_date.len() - 1];
                    }
                    result.restore_info = Some(RestoreInfo {
                        restore_status: RestoreStatus {
                            ongoing_request: false,
                            expiry_date: parse_date_time_rfc1123(expiry_date)?,
                        },
                        restore_param: None,
                    });
                }
            }
        }
        result.server_side_encryption = get_header_value(header, HEADER_SERVER_SIDE_ENCRYPTION);
        result.server_side_encryption_key_id = get_header_value(header, HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID);
        result.storage_class = StorageClassType::from(get_header_value_str(header, HEADER_STORAGE_CLASS));
        result.replication_status = ReplicationStatusType::from(get_header_value_str(header, HEADER_REPLICATION_STATUS));
        result.tagging_count = get_header_value_from_str::<isize>(header, HEADER_TAGGING_COUNT, 0)?;
        result.symlink_target_size = get_header_value_from_str::<i64>(header, HEADER_SYMLINK_TARGET_SIZE, -1)?;
        result.expiration = get_header_value(header, HEADER_EXPIRATION);
        result.is_directory = get_header_value_str(header, HEADER_DIRECTORY) == TRUE;
        result.meta = meta;
        result.request_info = request_info;
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RestoreInfo {
    pub(crate) restore_status: RestoreStatus,
    pub(crate) restore_param: Option<RestoreParam>,
}

impl RestoreInfo {
    pub fn restore_status(&self) -> &RestoreStatus {
        &self.restore_status
    }
    pub fn restore_param(&self) -> &Option<RestoreParam> {
        &self.restore_param
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RestoreStatus {
    pub(crate) ongoing_request: bool,
    pub(crate) expiry_date: Option<DateTime<Utc>>,
}

impl RestoreStatus {
    pub fn ongoing_request(&self) -> bool {
        self.ongoing_request
    }
    pub fn expiry_date(&self) -> Option<DateTime<Utc>> {
        self.expiry_date
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RestoreParam {
    pub(crate) request_date: Option<DateTime<Utc>>,
    pub(crate) expiry_days: isize,
    pub(crate) tier: Option<TierType>,
}

impl RestoreParam {
    pub fn request_date(&self) -> Option<DateTime<Utc>> {
        self.request_date
    }
    pub fn expiry_days(&self) -> isize {
        self.expiry_days
    }
    pub fn tier(&self) -> &Option<TierType> {
        &self.tier
    }
}

#[derive(Debug, Clone, HttpBasicHeader, AclHeader, MiscHeader, GenericInput)]
#[enable_content_length]
pub(crate) struct AppendObjectBasicInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) offset: i64,
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
    pub(crate) meta: HashMap<String, String>,
    pub(crate) website_redirect_location: String,
    pub(crate) storage_class: Option<StorageClassType>,
    pub(crate) traffic_limit: i64,
    pub(crate) if_match: String,
    pub(crate) if_none_match: String,
    pub(crate) pre_hash_crc64ecma: u64,
    pub(crate) object_expires: i64,
    pub(crate) rate_limiter: Option<Arc<RateLimiter>>,
    pub(crate) data_transfer_listener: Option<Sender<DataTransferStatus>>,
    pub(crate) async_data_transfer_listener: Option<async_channel::Sender<DataTransferStatus>>,
    pub(crate) notification_custom_parameters: String,
}


impl InputDescriptor for AppendObjectBasicInput {
    fn operation(&self) -> &str {
        "AppendObject"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for AppendObjectBasicInput {
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.offset < 0
        {
            return Err(TosError::client_error("invalid offset for append object"));
        }
        let mut request = self.trans_key()?;
        request.method = HttpMethodPost;
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

        if self.pre_hash_crc64ecma > 0 {
            if request.request_context.is_some() {
                request.request_context.as_mut().unwrap().init_crc64 = Some(self.pre_hash_crc64ecma);
            } else {
                let mut rc = RequestContext::default();
                rc.init_crc64 = Some(self.pre_hash_crc64ecma);
                request.request_context = Some(rc);
            }
        }
        let header = &mut request.header;
        set_http_basic_header(header, config_holder.disable_encoding_meta, self);
        set_acl_header(header, self);
        request.meta = trans_meta(&self.meta, config_holder.disable_encoding_meta);
        set_misc_header(header, self);
        if self.traffic_limit > 0 {
            header.insert(HEADER_TRAFFIC_LIMIT, self.traffic_limit.to_string());
        }
        map_insert(header, HEADER_X_IF_MATCH, &self.if_match);
        map_insert(header, HEADER_IF_NONE_MATCH, &self.if_none_match);
        if self.object_expires >= 0 {
            header.insert(HEADER_OBJECT_EXPIRES, self.object_expires.to_string());
        }
        map_insert(header, HEADER_NOTIFICATION_CUSTOM_PARAMETERS, &self.notification_custom_parameters);
        let mut query = HashMap::with_capacity(2);
        query.insert("append", "".to_string());
        query.insert(QUERY_OFFSET, self.offset.to_string());
        request.query = Some(query);
        Ok(request)
    }
}

impl Default for AppendObjectBasicInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            key: "".to_string(),
            offset: 0,
            content_length: -1,
            cache_control: "".to_string(),
            content_disposition: "".to_string(),
            content_encoding: "".to_string(),
            content_language: "".to_string(),
            content_type: "".to_string(),
            expires: None,
            acl: None,
            grant_full_control: "".to_string(),
            grant_read: "".to_string(),
            grant_read_acp: "".to_string(),
            grant_write: "".to_string(),
            grant_write_acp: "".to_string(),
            meta: Default::default(),
            website_redirect_location: "".to_string(),
            storage_class: None,
            traffic_limit: 0,
            if_match: "".to_string(),
            if_none_match: "".to_string(),
            pre_hash_crc64ecma: 0,
            object_expires: -1,
            rate_limiter: None,
            data_transfer_listener: None,
            async_data_transfer_listener: None,
            notification_custom_parameters: "".to_string(),
        }
    }
}

#[derive(Debug, HttpBasicHeader, AclHeader, MiscHeader, GenericInput)]
#[enable_content_length]
#[handle_content]
#[use_inner]
pub struct AppendObjectInput<B>
{
    pub(crate) inner: AppendObjectBasicInput,
    pub(crate) content: Arc<RefCell<Option<B>>>,
}

unsafe impl<B> Sync for AppendObjectInput<B> {}

impl<B> InputDescriptor for AppendObjectInput<B>
{
    fn operation(&self) -> &str {
        "AppendObject"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.inner.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.inner.key)
    }
}

impl<B> InputTranslator<B> for AppendObjectInput<B>
{
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.inner.trans(config_holder)?;
        request.operation = self.operation();
        request.body = self.content.take();
        Ok(request)
    }
}

impl<B> AppendObjectInput<B>
{
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input
    }
    pub fn new_with_offset(bucket: impl Into<String>, key: impl Into<String>, offset: i64) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.offset = offset;
        input
    }
    pub fn new_with_content(bucket: impl Into<String>, key: impl Into<String>, content: impl Into<B>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.set_content(content);
        input
    }
    pub fn new_with_offset_content(bucket: impl Into<String>, key: impl Into<String>, offset: i64, content: impl Into<B>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.offset = offset;
        input.set_content(content);
        input
    }
    pub fn bucket(&self) -> &str {
        &self.inner.bucket
    }
    pub fn key(&self) -> &str {
        &self.inner.key
    }
    pub fn offset(&self) -> i64 {
        self.inner.offset
    }
    pub fn content(&self) -> Ref<Option<B>> {
        self.content.borrow()
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.inner.meta
    }
    pub fn traffic_limit(&self) -> i64 {
        self.inner.traffic_limit
    }
    pub fn if_match(&self) -> &str {
        &self.inner.if_match
    }
    pub fn if_none_match(&self) -> &str {
        &self.inner.if_none_match
    }
    pub fn pre_hash_crc64ecma(&self) -> u64 {
        self.inner.pre_hash_crc64ecma
    }
    pub fn object_expires(&self) -> i64 {
        self.inner.object_expires
    }
    pub fn rate_limiter(&self) -> &Option<Arc<RateLimiter>> {
        &self.inner.rate_limiter
    }
    pub fn notification_custom_parameters(&self) -> &str {
        &self.inner.notification_custom_parameters
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.inner.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.inner.key = key.into();
    }
    pub fn set_offset(&mut self, offset: i64) {
        self.inner.offset = offset;
    }
    pub fn set_content(&mut self, content: impl Into<B>) {
        self.content = Arc::new(RefCell::new(Some(content.into())));
    }
    pub fn set_meta(&mut self, meta: impl Into<HashMap<String, String>>) {
        self.inner.meta = meta.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.inner.traffic_limit = traffic_limit;
    }
    pub fn set_if_match(&mut self, if_match: impl Into<String>) {
        self.inner.if_match = if_match.into();
    }
    pub fn set_if_none_match(&mut self, if_none_match: impl Into<String>) {
        self.inner.if_none_match = if_none_match.into();
    }
    pub fn set_pre_hash_crc64ecma(&mut self, pre_hash_crc64ecma: u64) {
        self.inner.pre_hash_crc64ecma = pre_hash_crc64ecma;
    }
    pub fn set_object_expires(&mut self, object_expires: i64) {
        self.inner.object_expires = object_expires;
    }
    pub fn set_rate_limiter(&mut self, rate_limiter: impl Into<Arc<RateLimiter>>) {
        self.inner.rate_limiter = Some(rate_limiter.into());
    }
    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.inner.notification_custom_parameters = notification_custom_parameters.into();
    }
}

impl<B> Default for AppendObjectInput<B>
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            content: Arc::new(RefCell::new(None)),
        }
    }
}

impl<B> DataTransferListener for AppendObjectInput<B> {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>> {
        &self.inner.data_transfer_listener
    }

    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>) {
        self.inner.data_transfer_listener = Some(listener.into());
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct AppendObjectOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) next_append_offset: i64,
    pub(crate) hash_crc64ecma: u64,
}

impl OutputParser for AppendObjectOutput {
    fn parse_by_ref<B>(request: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let hash_crc64ecma = get_header_value_from_str::<u64>(response.headers(), HEADER_HASH_CRC64ECMA, 0)?;
        if let Some(ref rc) = request.request_context {
            if let Some(calc_hash_crc64ecma) = rc.crc64 {
                if calc_hash_crc64ecma != hash_crc64ecma {
                    return Err(TosError::client_error(format!("expect crc64 {hash_crc64ecma}, actual crc64 {calc_hash_crc64ecma}")));
                }
            }
        }
        let mut result = Self::default();
        result.next_append_offset = get_header_value_from_str::<i64>(response.headers(), HEADER_NEXT_APPEND_OFFSET, 0)?;
        result.hash_crc64ecma = hash_crc64ecma;
        result.request_info = request_info;
        Ok(result)
    }
}

impl AppendObjectOutput {
    pub fn next_append_offset(&self) -> i64 {
        self.next_append_offset
    }
    pub fn hash_crc64ecma(&self) -> u64 {
        self.hash_crc64ecma
    }
}

#[derive(Debug, Default, HttpBasicHeader, AclHeader, MiscHeader, GenericInput)]
#[enable_content_length]
#[use_inner]
pub struct AppendObjectFromBufferInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) inner: AppendObjectBasicInput,
    pub(crate) content: Option<Vec<u8>>,
}

impl InputDescriptor for AppendObjectFromBufferInput {
    fn operation(&self) -> &str {
        "AppendObjectFromBuffer"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.inner.bucket)
    }
    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.inner.key)
    }
}

impl<B> InputTranslator<B> for AppendObjectFromBufferInput
where
    B: BuildBufferReader,
{
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.inner.trans(config_holder)?;
        request.operation = self.operation();
        if let Some(content) = &self.content {
            let (body, len) = B::new(content.to_owned())?;
            request.body = Some(body);
            if self.inner.content_length < 0 {
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
            }
        }
        Ok(request)
    }
}

impl DataTransferListener for AppendObjectFromBufferInput {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>> {
        &self.inner.data_transfer_listener
    }

    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>) {
        self.inner.data_transfer_listener = Some(listener.into());
    }
}

impl AppendObjectFromBufferInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input
    }
    pub fn new_with_offset(bucket: impl Into<String>, key: impl Into<String>, offset: i64) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.offset = offset;
        input
    }
    pub fn new_with_content(bucket: impl Into<String>, key: impl Into<String>, content: impl AsRef<[u8]>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.set_content(content);
        input
    }
    pub fn new_with_offset_content(bucket: impl Into<String>, key: impl Into<String>, offset: i64, content: impl AsRef<[u8]>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.inner.offset = offset;
        input.set_content(content);
        input
    }
    pub fn bucket(&self) -> &str {
        &self.inner.bucket
    }
    pub fn key(&self) -> &str {
        &self.inner.key
    }
    pub fn offset(&self) -> i64 {
        self.inner.offset
    }
    pub fn content(&self) -> &Option<impl AsRef<[u8]>> {
        &self.content
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.inner.meta
    }
    pub fn traffic_limit(&self) -> i64 {
        self.inner.traffic_limit
    }
    pub fn if_match(&self) -> &str {
        &self.inner.if_match
    }
    pub fn if_none_match(&self) -> &str {
        &self.inner.if_none_match
    }
    pub fn pre_hash_crc64ecma(&self) -> u64 {
        self.inner.pre_hash_crc64ecma
    }
    pub fn object_expires(&self) -> i64 {
        self.inner.object_expires
    }
    pub fn rate_limiter(&self) -> &Option<Arc<RateLimiter>> {
        &self.inner.rate_limiter
    }
    pub fn notification_custom_parameters(&self) -> &str {
        &self.inner.notification_custom_parameters
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.inner.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.inner.key = key.into();
    }
    pub fn set_offset(&mut self, offset: i64) {
        self.inner.offset = offset;
    }
    pub fn set_content(&mut self, content: impl AsRef<[u8]>) {
        self.content = Some(content.as_ref().to_owned());
    }
    pub fn set_content_nocopy(&mut self, content: impl Into<Vec<u8>>) {
        self.content = Some(content.into());
    }
    pub fn set_meta(&mut self, meta: impl Into<HashMap<String, String>>) {
        self.inner.meta = meta.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.inner.traffic_limit = traffic_limit;
    }
    pub fn set_if_match(&mut self, if_match: impl Into<String>) {
        self.inner.if_match = if_match.into();
    }
    pub fn set_if_none_match(&mut self, if_none_match: impl Into<String>) {
        self.inner.if_none_match = if_none_match.into();
    }
    pub fn set_pre_hash_crc64ecma(&mut self, pre_hash_crc64ecma: u64) {
        self.inner.pre_hash_crc64ecma = pre_hash_crc64ecma;
    }
    pub fn set_object_expires(&mut self, object_expires: i64) {
        self.inner.object_expires = object_expires;
    }
    pub fn set_rate_limiter(&mut self, rate_limiter: impl Into<Arc<RateLimiter>>) {
        self.inner.rate_limiter = Some(rate_limiter.into());
    }
    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.inner.notification_custom_parameters = notification_custom_parameters.into();
    }
}

#[derive(Debug, Clone, PartialEq, ListCommonQuery, GenericInput)]
pub struct ListObjectsInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) prefix: String,
    pub(crate) delimiter: String,
    pub(crate) marker: String,
    pub(crate) max_keys: isize,
    pub(crate) encoding_type: String,
    pub(crate) fetch_meta: bool,
}

impl Default for ListObjectsInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            prefix: "".to_string(),
            delimiter: "".to_string(),
            marker: "".to_string(),
            max_keys: -1,
            encoding_type: "".to_string(),
            fetch_meta: false,
        }
    }
}

impl ListObjectsInput {
    pub fn new(bucket: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn marker(&self) -> &str {
        &self.marker
    }
    pub fn max_keys(&self) -> isize {
        self.max_keys
    }
    pub fn fetch_meta(&self) -> bool {
        self.fetch_meta
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_marker(&mut self, marker: impl Into<String>) {
        self.marker = marker.into();
    }
    pub fn set_max_keys(&mut self, max_keys: isize) {
        self.max_keys = max_keys;
    }
    pub fn set_fetch_meta(&mut self, fetch_meta: bool) {
        self.fetch_meta = fetch_meta;
    }
}

impl InputDescriptor for ListObjectsInput {
    fn operation(&self) -> &str {
        "ListObjects"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for ListObjectsInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(6);
        set_list_common_query(&mut query, self);
        map_insert(&mut query, QUERY_MARKER, &self.marker);
        if self.max_keys >= 0 {
            query.insert(QUERY_MAX_KEYS, self.max_keys.to_string());
        }
        if self.fetch_meta {
            query.insert(QUERY_FETCH_META, self.fetch_meta.to_string());
        }
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct ListObjectsOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Name")]
    pub(crate) name: String,
    #[serde(default)]
    #[serde(rename = "Prefix")]
    pub(crate) prefix: String,
    #[serde(default)]
    #[serde(rename = "Marker")]
    pub(crate) marker: String,
    #[serde(default)]
    #[serde(rename = "MaxKeys")]
    pub(crate) max_keys: isize,
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
    #[serde(rename = "NextMarker")]
    pub(crate) next_marker: String,
    #[serde(default)]
    #[serde(rename = "CommonPrefixes")]
    pub(crate) common_prefixes: Vec<ListedCommonPrefix>,
    #[serde(default)]
    #[serde(rename = "Contents")]
    pub(crate) contents: Vec<ListedObject>,
}

impl OutputParser for ListObjectsOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let mut result = parse_json::<Self>(response)?;
        for content in &mut result.contents {
            if let Some(x) = content.last_modified_string.take() {
                content.last_modified = parse_date_time_iso8601(&x)?;
            }

            if let Some(x) = content.user_meta.take() {
                let mut meta = HashMap::with_capacity(x.len());
                for item in x {
                    if let Ok(dk) = urlencoding::decode(&item.key[HEADER_PREFIX_META.len()..]) {
                        if let Ok(dv) = urlencoding::decode(item.value.as_str()) {
                            meta.insert(dk.to_string(), dv.to_string());
                        }
                    }
                }
                content.meta = meta;
            }
        }
        for common_prefix in &mut result.common_prefixes {
            if let Some(x) = common_prefix.last_modified_string.take() {
                common_prefix.last_modified = parse_date_time_iso8601(&x)?;
            }
        }
        result.request_info = request_info;
        Ok(result)
    }
}

impl ListObjectsOutput {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    pub fn marker(&self) -> &str {
        &self.marker
    }
    pub fn max_keys(&self) -> isize {
        self.max_keys
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
    pub fn next_marker(&self) -> &str {
        &self.next_marker
    }
    pub fn common_prefixes(&self) -> &Vec<ListedCommonPrefix> {
        &self.common_prefixes
    }
    pub fn contents(&self) -> &Vec<ListedObject> {
        &self.contents
    }
}


#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ListedObject {
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "LastModified")]
    pub(crate) last_modified_string: Option<String>,
    #[serde(skip)]
    pub(crate) last_modified: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(rename = "ETag")]
    pub(crate) etag: String,
    #[serde(default)]
    #[serde(rename = "Size")]
    pub(crate) size: i64,
    #[serde(default)]
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
    #[serde(default)]
    #[serde(rename = "StorageClass")]
    pub(crate) storage_class: Option<StorageClassType>,
    #[serde(default)]
    #[serde(rename = "HashCrc64ecma")]
    pub(crate) hash_crc64ecma: String,
    #[serde(default)]
    #[serde(rename = "UserMeta")]
    pub(crate) user_meta: Option<Vec<MetaItem>>,
    #[serde(skip)]
    pub(crate) meta: HashMap<String, String>,
    #[serde(default)]
    #[serde(rename = "Type")]
    pub(crate) object_type: String,
    #[serde(default)]
    #[serde(rename = "HashCrc32c")]
    pub(crate) hash_crc32c: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub(crate) struct MetaItem {
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "Value")]
    pub(crate) value: String,
}


impl ListedObject {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }
    pub fn etag(&self) -> &str {
        &self.etag
    }
    pub fn size(&self) -> i64 {
        self.size
    }
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }
    pub fn hash_crc64ecma(&self) -> u64 {
        self.hash_crc64ecma.parse::<u64>().unwrap_or_else(|_| 0)
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.meta
    }

    pub fn object_type(&self) -> &str {
        &self.object_type
    }

    pub fn hash_crc32c(&self) -> Option<u32> {
        match &self.hash_crc32c {
            None => None,
            Some(hash_crc32c) => Some(hash_crc32c.parse::<u32>().unwrap_or_else(|_| 0))
        }
    }
}

#[derive(Debug, Clone, PartialEq, ListCommonQuery, GenericInput)]
pub struct ListObjectsType2Input {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) prefix: String,
    pub(crate) delimiter: String,
    pub(crate) start_after: String,
    pub(crate) continuation_token: String,
    pub(crate) max_keys: isize,
    pub(crate) encoding_type: String,
    pub(crate) list_only_once: bool,
    pub(crate) fetch_meta: bool,
}

impl Default for ListObjectsType2Input {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            prefix: "".to_string(),
            delimiter: "".to_string(),
            start_after: "".to_string(),
            continuation_token: "".to_string(),
            max_keys: -1,
            encoding_type: "".to_string(),
            list_only_once: false,
            fetch_meta: false,
        }
    }
}

impl ListObjectsType2Input {
    pub fn new(bucket: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn start_after(&self) -> &str {
        &self.start_after
    }
    pub fn continuation_token(&self) -> &str {
        &self.continuation_token
    }
    pub fn max_keys(&self) -> isize {
        self.max_keys
    }
    pub fn list_only_once(&self) -> bool {
        self.list_only_once
    }
    pub fn fetch_meta(&self) -> bool {
        self.fetch_meta
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_start_after(&mut self, start_after: impl Into<String>) {
        self.start_after = start_after.into();
    }
    pub fn set_continuation_token(&mut self, continuation_token: impl Into<String>) {
        self.continuation_token = continuation_token.into();
    }
    pub fn set_max_keys(&mut self, max_keys: isize) {
        self.max_keys = max_keys;
    }
    pub fn set_list_only_once(&mut self, list_only_once: bool) {
        self.list_only_once = list_only_once;
    }
    pub fn set_fetch_meta(&mut self, fetch_meta: bool) {
        self.fetch_meta = fetch_meta;
    }
}

impl InputDescriptor for ListObjectsType2Input {
    fn operation(&self) -> &str {
        "ListObjectsType2"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for ListObjectsType2Input {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(9);
        set_list_common_query(&mut query, self);
        map_insert(&mut query, QUERY_START_AFTER, &self.start_after);
        map_insert(&mut query, QUERY_CONTINUATION_TOKEN, &self.continuation_token);
        if self.max_keys >= 0 {
            query.insert(QUERY_MAX_KEYS, self.max_keys.to_string());
        }
        if self.fetch_meta {
            query.insert(QUERY_FETCH_META, self.fetch_meta.to_string());
        }
        query.insert("list-type", 2.to_string());
        query.insert(QUERY_FETCH_OWNER, true.to_string());
        request.query = Some(query);
        Ok(request)
    }
}


#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct ListObjectsType2Output {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Name")]
    pub(crate) name: String,
    #[serde(default)]
    #[serde(rename = "Prefix")]
    pub(crate) prefix: String,
    #[serde(default)]
    #[serde(rename = "StartAfter")]
    pub(crate) start_after: String,
    #[serde(default)]
    #[serde(rename = "ContinuationToken")]
    pub(crate) continuation_token: String,
    #[serde(default)]
    #[serde(rename = "MaxKeys")]
    pub(crate) max_keys: isize,
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
    #[serde(rename = "KeyCount")]
    pub(crate) key_count: isize,
    #[serde(default)]
    #[serde(rename = "NextContinuationToken")]
    pub(crate) next_continuation_token: String,
    #[serde(default)]
    #[serde(rename = "CommonPrefixes")]
    pub(crate) common_prefixes: Vec<ListedCommonPrefix>,
    #[serde(default)]
    #[serde(rename = "Contents")]
    pub(crate) contents: Vec<ListedObject>,
}

impl ListObjectsType2Output {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    pub fn start_after(&self) -> &str {
        &self.start_after
    }
    pub fn continuation_token(&self) -> &str {
        &self.continuation_token
    }
    pub fn max_keys(&self) -> isize {
        self.max_keys
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
    pub fn key_count(&self) -> isize {
        self.key_count
    }
    pub fn next_continuation_token(&self) -> &str {
        &self.next_continuation_token
    }
    pub fn common_prefixes(&self) -> &Vec<ListedCommonPrefix> {
        &self.common_prefixes
    }
    pub fn contents(&self) -> &Vec<ListedObject> {
        &self.contents
    }
}

impl OutputParser for ListObjectsType2Output {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let mut result = parse_json::<Self>(response)?;
        for content in &mut result.contents {
            if let Some(x) = content.last_modified_string.take() {
                content.last_modified = parse_date_time_iso8601(&x)?;
            }

            if let Some(x) = content.user_meta.take() {
                let mut meta = HashMap::with_capacity(x.len());
                for item in x {
                    if let Ok(dk) = urlencoding::decode(&item.key[HEADER_PREFIX_META.len()..]) {
                        if let Ok(dv) = urlencoding::decode(item.value.as_str()) {
                            meta.insert(dk.to_string(), dv.to_string());
                        }
                    }
                }
                content.meta = meta;
            }
        }
        for common_prefix in &mut result.common_prefixes {
            if let Some(x) = common_prefix.last_modified_string.take() {
                common_prefix.last_modified = parse_date_time_iso8601(&x)?;
            }
        }
        result.request_info = request_info;
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, ListCommonQuery, GenericInput)]
pub struct ListObjectVersionsInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) prefix: String,
    pub(crate) delimiter: String,
    pub(crate) key_marker: String,
    pub(crate) version_id_marker: String,
    pub(crate) max_keys: isize,
    pub(crate) encoding_type: String,
    pub(crate) fetch_meta: bool,
}

impl Default for ListObjectVersionsInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            prefix: "".to_string(),
            delimiter: "".to_string(),
            key_marker: "".to_string(),
            version_id_marker: "".to_string(),
            max_keys: -1,
            encoding_type: "".to_string(),
            fetch_meta: false,
        }
    }
}

impl ListObjectVersionsInput {
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
    pub fn version_id_marker(&self) -> &str {
        &self.version_id_marker
    }
    pub fn max_keys(&self) -> isize {
        self.max_keys
    }
    pub fn fetch_meta(&self) -> bool {
        self.fetch_meta
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key_marker(&mut self, key_marker: impl Into<String>) {
        self.key_marker = key_marker.into();
    }
    pub fn set_version_id_marker(&mut self, version_id_marker: impl Into<String>) {
        self.version_id_marker = version_id_marker.into();
    }
    pub fn set_max_keys(&mut self, max_keys: isize) {
        self.max_keys = max_keys;
    }
    pub fn set_fetch_meta(&mut self, fetch_meta: bool) {
        self.fetch_meta = fetch_meta;
    }
}

impl InputDescriptor for ListObjectVersionsInput {
    fn operation(&self) -> &str {
        "ListObjectVersions"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for ListObjectVersionsInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(8);
        query.insert("versions", "".to_string());
        set_list_common_query(&mut query, self);
        map_insert(&mut query, QUERY_KEY_MARKER, &self.key_marker);
        map_insert(&mut query, QUERY_VERSION_ID_MARKER, &self.version_id_marker);
        if self.max_keys >= 0 {
            query.insert(QUERY_MAX_KEYS, self.max_keys.to_string());
        }
        if self.fetch_meta {
            query.insert(QUERY_FETCH_META, self.fetch_meta.to_string());
        }
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct ListObjectVersionsOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(default)]
    #[serde(rename = "Name")]
    pub(crate) name: String,
    #[serde(default)]
    #[serde(rename = "Prefix")]
    pub(crate) prefix: String,
    #[serde(default)]
    #[serde(rename = "KeyMarker")]
    pub(crate) key_marker: String,
    #[serde(default)]
    #[serde(rename = "VersionIdMarker")]
    pub(crate) version_id_marker: String,
    #[serde(default)]
    #[serde(rename = "MaxKeys")]
    pub(crate) max_keys: isize,
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
    #[serde(rename = "NextVersionIdMarker")]
    pub(crate) next_version_id_marker: String,
    #[serde(default)]
    #[serde(rename = "CommonPrefixes")]
    pub(crate) common_prefixes: Vec<ListedCommonPrefix>,
    #[serde(default)]
    #[serde(rename = "Versions")]
    pub(crate) versions: Vec<ListedObjectVersion>,
    #[serde(default)]
    #[serde(rename = "DeleteMarkers")]
    pub(crate) delete_markers: Vec<ListedDeleteMarker>,
}

impl ListObjectVersionsOutput {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    pub fn key_marker(&self) -> &str {
        &self.key_marker
    }
    pub fn version_id_marker(&self) -> &str {
        &self.version_id_marker
    }
    pub fn max_keys(&self) -> isize {
        self.max_keys
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
    pub fn next_version_id_marker(&self) -> &str {
        &self.next_version_id_marker
    }
    pub fn common_prefixes(&self) -> &Vec<ListedCommonPrefix> {
        &self.common_prefixes
    }
    pub fn versions(&self) -> &Vec<ListedObjectVersion> {
        &self.versions
    }
    pub fn delete_markers(&self) -> &Vec<ListedDeleteMarker> {
        &self.delete_markers
    }
}

impl OutputParser for ListObjectVersionsOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let mut result = parse_json::<Self>(response)?;
        for version in &mut result.versions {
            if let Some(x) = version.last_modified_string.take() {
                version.last_modified = parse_date_time_iso8601(&x)?;
            }

            if let Some(x) = version.user_meta.take() {
                let mut meta = HashMap::with_capacity(x.len());
                for item in x {
                    if let Ok(dk) = urlencoding::decode(&item.key[HEADER_PREFIX_META.len()..]) {
                        if let Ok(dv) = urlencoding::decode(item.value.as_str()) {
                            meta.insert(dk.to_string(), dv.to_string());
                        }
                    }
                }
                version.meta = meta;
            }
        }

        for delete_marker in &mut result.delete_markers {
            if let Some(x) = delete_marker.last_modified_string.take() {
                delete_marker.last_modified = parse_date_time_iso8601(&x)?;
            }
        }

        result.request_info = request_info;
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ListedObjectVersion {
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "LastModified")]
    pub(crate) last_modified_string: Option<String>,
    #[serde(skip)]
    pub(crate) last_modified: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(rename = "ETag")]
    pub(crate) etag: String,
    #[serde(default)]
    #[serde(rename = "IsLatest")]
    pub(crate) is_latest: bool,
    #[serde(default)]
    #[serde(rename = "Size")]
    pub(crate) size: i64,
    #[serde(default)]
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
    #[serde(default)]
    #[serde(rename = "StorageClass")]
    pub(crate) storage_class: Option<StorageClassType>,
    #[serde(default)]
    #[serde(rename = "VersionId")]
    pub(crate) version_id: String,
    #[serde(default)]
    #[serde(rename = "HashCrc64ecma")]
    pub(crate) hash_crc64ecma: String,
    #[serde(default)]
    #[serde(rename = "UserMeta")]
    pub(crate) user_meta: Option<Vec<MetaItem>>,
    #[serde(skip)]
    pub(crate) meta: HashMap<String, String>,
}

impl ListedObjectVersion {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }
    pub fn etag(&self) -> &str {
        &self.etag
    }
    pub fn is_latest(&self) -> bool {
        self.is_latest
    }
    pub fn size(&self) -> i64 {
        self.size
    }
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn hash_crc64ecma(&self) -> &str {
        &self.hash_crc64ecma
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.meta
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ListedDeleteMarker {
    #[serde(default)]
    #[serde(rename = "Key")]
    pub(crate) key: String,
    #[serde(default)]
    #[serde(rename = "LastModified")]
    pub(crate) last_modified_string: Option<String>,
    #[serde(skip)]
    pub(crate) last_modified: Option<DateTime<Utc>>,
    #[serde(default)]
    #[serde(rename = "IsLatest")]
    pub(crate) is_latest: bool,
    #[serde(default)]
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
    #[serde(default)]
    #[serde(rename = "VersionId")]
    pub(crate) version_id: String,
}

impl ListedDeleteMarker {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }
    pub fn is_latest(&self) -> bool {
        self.is_latest
    }
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
}

#[derive(
    Debug,
    Clone,
    HttpBasicHeader,
    AclHeader,
    SseHeader,
    SsecHeader,
    MiscHeader,
    CallbackHeader,
    GenericInput
)]
#[enable_content_length]
pub(crate) struct PutObjectBasicInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) content_md5: String,
    pub(crate) content_sha256: String,
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
    pub(crate) traffic_limit: i64,
    pub(crate) callback: String,
    pub(crate) callback_var: String,
    pub(crate) forbid_overwrite: bool,
    pub(crate) if_match: String,
    pub(crate) if_none_match: String,

    pub(crate) tagging: String,
    pub(crate) object_expires: i64,
    pub(crate) rate_limiter: Option<Arc<RateLimiter>>,
    pub(crate) data_transfer_listener: Option<Sender<DataTransferStatus>>,
    pub(crate) async_data_transfer_listener: Option<async_channel::Sender<DataTransferStatus>>,
    pub(crate) notification_custom_parameters: String,
}

impl Default for PutObjectBasicInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            key: "".to_string(),
            content_md5: "".to_string(),
            content_sha256: "".to_string(),
            content_length: -1,
            cache_control: "".to_string(),
            content_disposition: "".to_string(),
            content_encoding: "".to_string(),
            content_language: "".to_string(),
            content_type: "".to_string(),
            expires: None,
            acl: None,
            grant_full_control: "".to_string(),
            grant_read: "".to_string(),
            grant_read_acp: "".to_string(),
            grant_write: "".to_string(),
            grant_write_acp: "".to_string(),
            ssec_algorithm: "".to_string(),
            ssec_key: "".to_string(),
            ssec_key_md5: "".to_string(),
            server_side_encryption: "".to_string(),
            server_side_encryption_key_id: "".to_string(),
            meta: Default::default(),
            website_redirect_location: "".to_string(),
            storage_class: None,
            traffic_limit: 0,
            callback: "".to_string(),
            callback_var: "".to_string(),
            forbid_overwrite: false,
            if_match: "".to_string(),
            if_none_match: "".to_string(),
            tagging: "".to_string(),
            object_expires: -1,
            rate_limiter: None,
            data_transfer_listener: None,
            async_data_transfer_listener: None,
            notification_custom_parameters: "".to_string(),
        }
    }
}

impl InputDescriptor for PutObjectBasicInput {
    fn operation(&self) -> &str {
        "PutObject"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for PutObjectBasicInput {
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
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
        set_http_basic_header(header, config_holder.disable_encoding_meta, self);
        set_acl_header(header, self);
        set_sse_header(header, self)?;
        set_ssec_header(header, &self.server_side_encryption, self)?;
        request.meta = trans_meta(&self.meta, config_holder.disable_encoding_meta);
        set_misc_header(header, self);
        set_callback_header(header, self);
        map_insert(header, HEADER_CONTENT_MD5, &self.content_md5);
        map_insert(header, HEADER_CONTENT_SHA256, &self.content_sha256);
        if self.forbid_overwrite {
            header.insert(HEADER_FORBID_OVERWRITE, self.forbid_overwrite.to_string());
        }
        if self.traffic_limit > 0 {
            header.insert(HEADER_TRAFFIC_LIMIT, self.traffic_limit.to_string());
        }
        map_insert(header, HEADER_X_IF_MATCH, &self.if_match);
        map_insert(header, HEADER_IF_NONE_MATCH, &self.if_none_match);

        if self.tagging != "" {
            header.insert(HEADER_TAGGING, self.tagging.clone());
        }
        if self.object_expires >= 0 {
            header.insert(HEADER_OBJECT_EXPIRES, self.object_expires.to_string());
        }
        map_insert(header, HEADER_NOTIFICATION_CUSTOM_PARAMETERS, &self.notification_custom_parameters);
        Ok(request)
    }
}

// AclHeader
#[derive(
    Debug,
    HttpBasicHeader,
    AclHeader,
    SseHeader,
    SsecHeader,
    MiscHeader,
    CallbackHeader,
    GenericInput
)]
#[enable_content_length]
#[handle_content]
#[use_inner]
pub struct PutObjectInput<B>
{
    pub(crate) inner: PutObjectBasicInput,
    pub(crate) content: Arc<RefCell<Option<B>>>,
}

unsafe impl<B> Sync for PutObjectInput<B> {}

unsafe impl<B> Send for PutObjectInput<B> {}

impl<B> InputDescriptor for PutObjectInput<B>
{
    fn operation(&self) -> &str {
        "PutObject"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.inner.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.inner.key)
    }
}

impl<B> InputTranslator<B> for PutObjectInput<B>
{
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.inner.trans(config_holder)?;
        request.operation = self.operation();
        request.body = self.content.take();
        Ok(request)
    }
}

impl<B> Default for PutObjectInput<B>
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
            content: Arc::new(RefCell::new(None)),
        }
    }
}

impl<B> DataTransferListener for PutObjectInput<B> {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>> {
        &self.inner.data_transfer_listener
    }

    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>) {
        self.inner.data_transfer_listener = Some(listener.into());
    }
}

impl<B> PutObjectInput<B>
{
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input
    }
    pub fn new_with_content(bucket: impl Into<String>, key: impl Into<String>, content: impl Into<B>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
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
    pub fn content_md5(&self) -> &str {
        &self.inner.content_md5
    }
    pub fn content_sha256(&self) -> &str {
        &self.inner.content_sha256
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.inner.meta
    }
    pub fn traffic_limit(&self) -> i64 {
        self.inner.traffic_limit
    }
    pub fn forbid_overwrite(&self) -> bool {
        self.inner.forbid_overwrite
    }
    pub fn if_match(&self) -> &str {
        &self.inner.if_match
    }
    pub fn if_none_match(&self) -> &str {
        &self.inner.if_none_match
    }
    pub fn tagging(&self) -> &str {
        &self.inner.tagging
    }
    pub fn object_expires(&self) -> i64 {
        self.inner.object_expires
    }
    pub fn rate_limiter(&self) -> &Option<Arc<RateLimiter>> {
        &self.inner.rate_limiter
    }
    pub fn notification_custom_parameters(&mut self) -> &str {
        &self.inner.notification_custom_parameters
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
    pub fn set_content_md5(&mut self, content_md5: impl Into<String>) {
        self.inner.content_md5 = content_md5.into();
    }
    pub fn set_content_sha256(&mut self, content_sha256: impl Into<String>) {
        self.inner.content_sha256 = content_sha256.into();
    }
    pub fn set_meta(&mut self, meta: impl Into<HashMap<String, String>>) {
        self.inner.meta = meta.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.inner.traffic_limit = traffic_limit;
    }
    pub fn set_forbid_overwrite(&mut self, forbid_overwrite: bool) {
        self.inner.forbid_overwrite = forbid_overwrite;
    }
    pub fn set_if_match(&mut self, if_match: impl Into<String>) {
        self.inner.if_match = if_match.into();
    }
    pub fn set_if_none_match(&mut self, if_none_match: impl Into<String>) {
        self.inner.if_none_match = if_none_match.into();
    }
    pub fn set_tagging(&mut self, tagging: impl Into<String>) {
        self.inner.tagging = tagging.into();
    }
    pub fn set_object_expires(&mut self, object_expires: i64) {
        self.inner.object_expires = object_expires;
    }
    pub fn set_rate_limiter(&mut self, rate_limiter: impl Into<Arc<RateLimiter>>) {
        self.inner.rate_limiter = Some(rate_limiter.into());
    }
    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.inner.notification_custom_parameters = notification_custom_parameters.into();
    }
}


#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutObjectOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) etag: String,
    pub(crate) version_id: String,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key_md5: String,
    pub(crate) hash_crc64ecma: u64,
    pub(crate) callback_result: String,
    pub(crate) server_side_encryption: String,
    pub(crate) server_side_encryption_key_id: String,
}

impl OutputParser for PutObjectOutput {
    fn parse_by_ref<B>(request: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        let hash_crc64ecma = get_header_value_from_str::<u64>(response.headers(), HEADER_HASH_CRC64ECMA, 0)?;
        if let Some(ref rc) = request.request_context {
            if let Some(calc_hash_crc64ecma) = rc.crc64 {
                if calc_hash_crc64ecma != hash_crc64ecma {
                    return Err(TosError::client_error(format!("expect crc64 {hash_crc64ecma}, actual crc64 {calc_hash_crc64ecma}")));
                }
            }
        }

        let mut result = Self::default();
        if get_map_value_str(&request.header, HEADER_CALLBACK) != "" { // callback
            let buf = read_response(response)?;
            if request_info.status_code == 203 {
                if let Ok(error_response) = parse_json_by_buf::<ErrorResponse>(buf.as_slice()) {
                    return Err(TosError::server_error_with_code(error_response.code, error_response.ec, error_response.key, error_response.message,
                                                                error_response.host_id, error_response.resource, request_info));
                }
            }
            result.callback_result = parse_response_string_by_buf(buf)?;
        }
        result.etag = get_header_value(response.headers(), HEADER_ETAG);
        result.version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        result.ssec_algorithm = get_header_value(response.headers(), HEADER_SSEC_ALGORITHM);
        result.ssec_key_md5 = get_header_value(response.headers(), HEADER_SSEC_KEY_MD5);
        result.hash_crc64ecma = hash_crc64ecma;
        result.server_side_encryption = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION);
        result.server_side_encryption_key_id = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID);
        result.request_info = request_info;
        Ok(result)
    }
}

impl PutObjectOutput {
    pub fn etag(&self) -> &str {
        &self.etag
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
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

#[derive(
    Debug,
    HttpBasicHeader,
    AclHeader,
    SseHeader,
    SsecHeader,
    MiscHeader,
    CallbackHeader,
    GenericInput
)]
#[enable_content_length]
#[use_inner]
pub struct PutObjectFromBufferInput {
    pub(crate) inner: PutObjectBasicInput,
    pub(crate) content: Option<Vec<u8>>,
}

impl InputDescriptor for PutObjectFromBufferInput {
    fn operation(&self) -> &str {
        "PutObjectFromBuffer"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.inner.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.inner.key)
    }
}

impl<B> InputTranslator<B> for PutObjectFromBufferInput
where
    B: BuildBufferReader,
{
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.inner.trans(config_holder)?;
        request.operation = self.operation();
        if let Some(content) = &self.content {
            let (body, len) = B::new(content.to_owned())?;
            request.body = Some(body);
            if self.inner.content_length < 0 {
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
            }
        }
        Ok(request)
    }
}

impl PutObjectFromBufferInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input
    }
    pub fn new_with_content(bucket: impl Into<String>, key: impl Into<String>, content: impl AsRef<[u8]>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input.set_content(content);
        input
    }
    pub fn bucket(&self) -> &str {
        &self.inner.bucket
    }
    pub fn key(&self) -> &str {
        &self.inner.key
    }
    pub fn content(&self) -> &Option<impl AsRef<[u8]>> {
        &self.content
    }
    pub fn content_md5(&self) -> &str {
        &self.inner.content_md5
    }
    pub fn content_sha256(&self) -> &str {
        &self.inner.content_sha256
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.inner.meta
    }
    pub fn traffic_limit(&self) -> i64 {
        self.inner.traffic_limit
    }
    pub fn forbid_overwrite(&self) -> bool {
        self.inner.forbid_overwrite
    }
    pub fn if_match(&self) -> &str {
        &self.inner.if_match
    }
    pub fn if_none_match(&self) -> &str {
        &self.inner.if_none_match
    }
    pub fn tagging(&self) -> &str {
        &self.inner.tagging
    }
    pub fn object_expires(&self) -> i64 {
        self.inner.object_expires
    }
    pub fn rate_limiter(&self) -> &Option<Arc<RateLimiter>> {
        &self.inner.rate_limiter
    }
    pub fn notification_custom_parameters(&mut self) -> &str {
        &self.inner.notification_custom_parameters
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.inner.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.inner.key = key.into();
    }
    pub fn set_content(&mut self, content: impl AsRef<[u8]>) {
        self.content = Some(content.as_ref().to_owned());
    }
    pub fn set_content_nocopy(&mut self, content: impl Into<Vec<u8>>) {
        self.content = Some(content.into());
    }
    pub fn set_content_md5(&mut self, content_md5: impl Into<String>) {
        self.inner.content_md5 = content_md5.into();
    }
    pub fn set_content_sha256(&mut self, content_sha256: impl Into<String>) {
        self.inner.content_sha256 = content_sha256.into();
    }
    pub fn set_meta(&mut self, meta: impl Into<HashMap<String, String>>) {
        self.inner.meta = meta.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.inner.traffic_limit = traffic_limit;
    }
    pub fn set_forbid_overwrite(&mut self, forbid_overwrite: bool) {
        self.inner.forbid_overwrite = forbid_overwrite;
    }
    pub fn set_if_match(&mut self, if_match: impl Into<String>) {
        self.inner.if_match = if_match.into();
    }
    pub fn set_if_none_match(&mut self, if_none_match: impl Into<String>) {
        self.inner.if_none_match = if_none_match.into();
    }
    pub fn set_tagging(&mut self, tagging: impl Into<String>) {
        self.inner.tagging = tagging.into();
    }
    pub fn set_object_expires(&mut self, object_expires: i64) {
        self.inner.object_expires = object_expires;
    }
    pub fn set_rate_limiter(&mut self, rate_limiter: impl Into<Arc<RateLimiter>>) {
        self.inner.rate_limiter = Some(rate_limiter.into());
    }
    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.inner.notification_custom_parameters = notification_custom_parameters.into();
    }
}

impl Default for PutObjectFromBufferInput {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            content: None,
        }
    }
}

impl DataTransferListener for PutObjectFromBufferInput {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>> {
        &self.inner.data_transfer_listener
    }

    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>) {
        self.inner.data_transfer_listener = Some(listener.into());
    }
}

#[derive(
    Debug,
    HttpBasicHeader,
    AclHeader,
    SseHeader,
    SsecHeader,
    MiscHeader,
    CallbackHeader,
    GenericInput
)]
#[enable_content_length]
#[use_inner]
pub struct PutObjectFromFileInput {
    pub(crate) inner: PutObjectBasicInput,
    pub(crate) file_path: String,
}

impl InputDescriptor for PutObjectFromFileInput {
    fn operation(&self) -> &str {
        "PutObjectFromFile"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.inner.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.inner.key)
    }
}

impl<B> InputTranslator<B> for PutObjectFromFileInput
where
    B: BuildFileReader,
{
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.inner.trans(config_holder)?;
        request.operation = self.operation();
        if self.file_path != "" {
            let (body, len) = B::new(&self.file_path)?;
            request.body = Some(body);
            if let Some(l) = len {
                if self.inner.content_length < 0 {
                    request.header.insert(HEADER_CONTENT_LENGTH, l.to_string());
                }
            }
        }
        Ok(request)
    }
}

impl DataTransferListener for PutObjectFromFileInput {
    fn data_transfer_listener(&self) -> &Option<Sender<DataTransferStatus>> {
        &self.inner.data_transfer_listener
    }

    fn set_data_transfer_listener(&mut self, listener: impl Into<Sender<DataTransferStatus>>) {
        self.inner.data_transfer_listener = Some(listener.into());
    }
}

impl PutObjectFromFileInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
        input
    }
    pub fn new_with_file_path(bucket: impl Into<String>, key: impl Into<String>, file_path: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.inner.bucket = bucket.into();
        input.inner.key = key.into();
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
    pub fn content_md5(&self) -> &str {
        &self.inner.content_md5
    }
    pub fn content_sha256(&self) -> &str {
        &self.inner.content_sha256
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.inner.meta
    }
    pub fn traffic_limit(&self) -> i64 {
        self.inner.traffic_limit
    }
    pub fn forbid_overwrite(&self) -> bool {
        self.inner.forbid_overwrite
    }
    pub fn if_match(&self) -> &str {
        &self.inner.if_match
    }
    pub fn if_none_match(&self) -> &str {
        &self.inner.if_none_match
    }
    pub fn tagging(&self) -> &str {
        &self.inner.tagging
    }
    pub fn object_expires(&self) -> i64 {
        self.inner.object_expires
    }
    pub fn rate_limiter(&self) -> &Option<Arc<RateLimiter>> {
        &self.inner.rate_limiter
    }
    pub fn notification_custom_parameters(&mut self) -> &str {
        &self.inner.notification_custom_parameters
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
    pub fn set_content_md5(&mut self, content_md5: impl Into<String>) {
        self.inner.content_md5 = content_md5.into();
    }
    pub fn set_content_sha256(&mut self, content_sha256: impl Into<String>) {
        self.inner.content_sha256 = content_sha256.into();
    }
    pub fn set_meta(&mut self, meta: impl Into<HashMap<String, String>>) {
        self.inner.meta = meta.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.inner.traffic_limit = traffic_limit;
    }
    pub fn set_forbid_overwrite(&mut self, forbid_overwrite: bool) {
        self.inner.forbid_overwrite = forbid_overwrite;
    }
    pub fn set_if_match(&mut self, if_match: impl Into<String>) {
        self.inner.if_match = if_match.into();
    }
    pub fn set_if_none_match(&mut self, if_none_match: impl Into<String>) {
        self.inner.if_none_match = if_none_match.into();
    }
    pub fn set_tagging(&mut self, tagging: impl Into<String>) {
        self.inner.tagging = tagging.into();
    }
    pub fn set_object_expires(&mut self, object_expires: i64) {
        self.inner.object_expires = object_expires;
    }
    pub fn set_rate_limiter(&mut self, rate_limiter: impl Into<Arc<RateLimiter>>) {
        self.inner.rate_limiter = Some(rate_limiter.into());
    }
    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.inner.notification_custom_parameters = notification_custom_parameters.into();
    }
}

impl Default for PutObjectFromFileInput {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            file_path: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, AclHeader, Serialize, GenericInput)]
pub struct PutObjectACLInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(skip)]
    pub(crate) key: String,
    #[serde(skip)]
    pub(crate) version_id: String,
    #[serde(skip)]
    pub(crate) acl: Option<ACLType>,
    #[serde(skip)]
    pub(crate) grant_full_control: String,
    #[serde(skip)]
    pub(crate) grant_read: String,
    #[serde(skip)]
    pub(crate) grant_read_acp: String,
    #[serde(skip)]
    pub(crate) grant_write: String,
    #[serde(skip)]
    pub(crate) grant_write_acp: String,
    #[serde(rename = "Owner")]
    pub(crate) owner: Owner,
    #[serde(rename = "Grants")]
    pub(crate) grants: Vec<Grant>,
    #[serde(rename = "BucketOwnerEntrusted")]
    #[serde(skip_serializing_if = "<&bool as std::ops::Not>::not")]
    pub(crate) bucket_owner_entrusted: bool,
    #[serde(rename = "IsDefault")]
    #[serde(skip_serializing_if = "<&bool as std::ops::Not>::not")]
    pub(crate) is_default: bool,
}

impl InputDescriptor for PutObjectACLInput {
    fn operation(&self) -> &str {
        "PutObjectACL"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for PutObjectACLInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodPut;
        if self.acl.is_some() && self.grants.len() > 0 {
            return Err(TosError::client_error("both acl and grants are set for put object acl"));
        }

        if self.acl.is_some() {
            set_acl_header(&mut request.header, self);
        } else if self.grants.len() == 0 {
            return Err(TosError::client_error("neither acl nor grants is set for put object acl"));
        } else if self.owner.id == "" {
            return Err(TosError::client_error("empty owner id for put object acl"));
        } else {
            match serde_json::to_string(self) {
                Err(e) => return Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
                Ok(json) => {
                    let (body, len) = B::new(json.into_bytes())?;
                    request.body = Some(body);
                    request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                }
            }
        }

        let mut query = HashMap::with_capacity(2);
        query.insert("acl", "".to_string());
        map_insert(&mut query, QUERY_VERSION_ID, &self.version_id);

        request.query = Some(query);
        Ok(request)
    }
}

impl PutObjectACLInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input
    }

    pub fn new_with_acl(bucket: impl Into<String>, key: impl Into<String>, acl: impl Into<ACLType>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.acl = Some(acl.into());
        input
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.version_id = version_id.into();
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn owner(&self) -> &Owner {
        &self.owner
    }
    pub fn grants(&self) -> &Vec<Grant> {
        &self.grants
    }
    pub fn bucket_owner_entrusted(&self) -> bool {
        self.bucket_owner_entrusted
    }
    pub fn is_default(&self) -> bool {
        self.is_default
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }
    pub fn set_owner(&mut self, owner: impl Into<Owner>) {
        self.owner = owner.into();
    }
    pub fn set_grants(&mut self, grants: impl Into<Vec<Grant>>) {
        self.grants = grants.into();
    }
    pub fn set_bucket_owner_entrusted(&mut self, bucket_owner_entrusted: bool) {
        self.bucket_owner_entrusted = bucket_owner_entrusted;
    }

    pub fn set_is_default(&mut self, is_default: bool) {
        self.is_default = is_default;
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutObjectACLOutput {
    pub(crate) request_info: RequestInfo,
}

impl OutputParser for PutObjectACLOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, _: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        Ok(Self { request_info })
    }
}

#[derive(Debug, Clone, PartialEq, Default, HttpBasicHeader, GenericInput)]
pub struct SetObjectMetaInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) version_id: String,
    pub(crate) content_length: i64,
    pub(crate) cache_control: String,
    pub(crate) content_disposition: String,
    pub(crate) content_encoding: String,
    pub(crate) content_language: String,
    pub(crate) content_type: String,
    pub(crate) expires: Option<DateTime<Utc>>,
    pub(crate) meta: HashMap<String, String>,
    pub(crate) metadata_directive: Option<MetadataDirectiveType>,
}

impl InputDescriptor for SetObjectMetaInput {
    fn operation(&self) -> &str {
        "SetObjectMeta"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for SetObjectMetaInput {
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodPost;
        let header = &mut request.header;
        set_http_basic_header(header, config_holder.disable_encoding_meta, self);
        request.meta = trans_meta(&self.meta, config_holder.disable_encoding_meta);
        if let Some(metadata_directive) = &self.metadata_directive {
            header.insert(HEADER_METADATA_DIRECTIVE, metadata_directive.as_str().to_string());
        }
        let mut query = HashMap::with_capacity(2);
        query.insert("metadata", "".to_string());
        map_insert(&mut query, QUERY_VERSION_ID, &self.version_id);
        request.query = Some(query);
        Ok(request)
    }
}

impl SetObjectMetaInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.content_length = -1;
        input
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.version_id = version_id.into();
        input.content_length = -1;
        input
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.meta
    }
    pub fn metadata_directive(&self) -> &Option<MetadataDirectiveType> {
        &self.metadata_directive
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }
    pub fn set_meta(&mut self, meta: impl Into<HashMap<String, String>>) {
        self.meta = meta.into();
    }
    pub fn set_metadata_directive(&mut self, metadata_directive: impl Into<MetadataDirectiveType>) {
        self.metadata_directive = Some(metadata_directive.into());
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct SetObjectMetaOutput {
    pub(crate) request_info: RequestInfo,
}

impl OutputParser for SetObjectMetaOutput {
    fn parse_by_ref<B>(_: &HttpRequest<B>, _: &mut HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError> {
        Ok(Self { request_info })
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct PutObjectTaggingInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(skip)]
    pub(crate) key: String,
    #[serde(skip)]
    pub(crate) version_id: String,
    #[serde(rename = "TagSet")]
    pub(crate) tag_set: TagSet,
}

impl PutObjectTaggingInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.version_id = version_id.into();
        input
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn tag_set(&self) -> &TagSet {
        &self.tag_set
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }

    pub fn set_tag_set(&mut self, tag_set: impl Into<TagSet>) {
        self.tag_set = tag_set.into();
    }
}

impl InputDescriptor for PutObjectTaggingInput {
    fn operation(&self) -> &str {
        "PutObjectTagging"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for PutObjectTaggingInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_key()?;
                request.method = HttpMethodPut;
                let mut query = HashMap::with_capacity(2);
                query.insert("tagging", "".to_string());
                map_insert(&mut query, QUERY_VERSION_ID, &self.version_id);
                request.query = Some(query);
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutObjectTaggingOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) version_id: String,
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct GetObjectTaggingInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) version_id: String,
}

impl GetObjectTaggingInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.version_id = version_id.into();
        input
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }
}

impl InputDescriptor for GetObjectTaggingInput {
    fn operation(&self) -> &str {
        "GetObjectTagging"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for GetObjectTaggingInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(2);
        query.insert("tagging", "".to_string());
        if self.version_id != "" {
            query.insert(QUERY_VERSION_ID, self.version_id.clone());
        }
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetObjectTaggingOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(skip)]
    pub(crate) version_id: String,
    #[serde(rename = "TagSet")]
    pub(crate) tag_set: TagSet,
}

impl GetObjectTaggingOutput {
    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn tag_set(&self) -> &TagSet {
        &self.tag_set
    }
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct DeleteObjectTaggingInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) version_id: String,
}

impl DeleteObjectTaggingInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.version_id = version_id.into();
        input
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }
}

impl InputDescriptor for DeleteObjectTaggingInput {
    fn operation(&self) -> &str {
        "DeleteObjectTagging"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for DeleteObjectTaggingInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodDelete;
        let mut query = HashMap::with_capacity(2);
        query.insert("tagging", "".to_string());
        if self.version_id != "" {
            query.insert(QUERY_VERSION_ID, self.version_id.clone());
        }
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct DeleteObjectTaggingOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) version_id: String,
}

#[derive(
    Debug,
    Clone,
    HttpBasicHeader,
    AclHeader,
    SseHeader,
    SsecHeader,
    MiscHeader,
    ObjectLock,
    IfMatch,
    Serialize,
    GenericInput,
)]
pub struct FetchObjectInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Object")]
    pub(crate) key: String,
    #[serde(rename = "URL")]
    pub(crate) url: String,
    #[serde(rename = "IgnoreSameKey")]
    #[serde(skip_serializing_if = "<&bool as std::ops::Not>::not")]
    pub(crate) ignore_same_key: bool,
    #[serde(rename = "ContentMD5")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) content_md5: String,
    #[serde(skip)]
    pub(crate) content_length: i64,
    #[serde(skip)]
    pub(crate) cache_control: String,
    #[serde(skip)]
    pub(crate) content_disposition: String,
    #[serde(skip)]
    pub(crate) content_encoding: String,
    #[serde(skip)]
    pub(crate) content_language: String,
    #[serde(skip)]
    pub(crate) content_type: String,
    #[serde(skip)]
    pub(crate) expires: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub(crate) acl: Option<ACLType>,
    #[serde(skip)]
    pub(crate) grant_full_control: String,
    #[serde(skip)]
    pub(crate) grant_read: String,
    #[serde(skip)]
    pub(crate) grant_read_acp: String,
    #[serde(skip)]
    pub(crate) grant_write: String,
    #[serde(skip)]
    pub(crate) grant_write_acp: String,
    #[serde(skip)]
    pub(crate) ssec_algorithm: String,
    #[serde(skip)]
    pub(crate) ssec_key: String,
    #[serde(skip)]
    pub(crate) ssec_key_md5: String,
    #[serde(skip)]
    pub(crate) server_side_encryption: String,
    #[serde(skip)]
    pub(crate) server_side_encryption_key_id: String,
    #[serde(skip)]
    pub(crate) meta: HashMap<String, String>,
    #[serde(skip)]
    pub(crate) website_redirect_location: String,
    #[serde(skip)]
    pub(crate) storage_class: Option<StorageClassType>,
    #[serde(skip)]
    pub(crate) traffic_limit: i64,
    #[serde(skip)]
    pub(crate) forbid_overwrite: bool,
    #[serde(skip)]
    pub(crate) if_match: String,
    #[serde(skip)]
    pub(crate) if_none_match: String,
    #[serde(skip)]
    pub(crate) tagging: String,
    #[serde(skip)]
    pub(crate) notification_custom_parameters: String,
    #[serde(skip)]
    pub(crate) detect_mime_type: bool,
    #[serde(skip)]
    pub(crate) object_expires: i64,
    #[serde(skip)]
    pub(crate) object_lock_mode: Option<ObjectLockModeType>,
    #[serde(skip)]
    pub(crate) object_lock_retain_util_date: Option<DateTime<Utc>>,
}

impl Default for FetchObjectInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            key: "".to_string(),
            url: "".to_string(),
            ignore_same_key: false,
            content_md5: "".to_string(),
            content_length: -1,
            cache_control: "".to_string(),
            content_disposition: "".to_string(),
            content_encoding: "".to_string(),
            content_language: "".to_string(),
            content_type: "".to_string(),
            expires: None,
            acl: None,
            grant_full_control: "".to_string(),
            grant_read: "".to_string(),
            grant_read_acp: "".to_string(),
            grant_write: "".to_string(),
            grant_write_acp: "".to_string(),
            ssec_algorithm: "".to_string(),
            ssec_key: "".to_string(),
            ssec_key_md5: "".to_string(),
            server_side_encryption: "".to_string(),
            server_side_encryption_key_id: "".to_string(),
            meta: Default::default(),
            website_redirect_location: "".to_string(),
            storage_class: None,
            traffic_limit: 0,
            forbid_overwrite: false,
            if_match: "".to_string(),
            if_none_match: "".to_string(),
            tagging: "".to_string(),
            notification_custom_parameters: "".to_string(),
            detect_mime_type: false,
            object_expires: -1,
            object_lock_mode: None,
            object_lock_retain_util_date: None,
        }
    }
}

impl FetchObjectInput
{
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input
    }
    pub fn new_with_url(bucket: impl Into<String>, key: impl Into<String>, url: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.url = url.into();
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn ignore_same_key(&self) -> bool {
        self.ignore_same_key
    }

    pub fn content_md5(&self) -> &str {
        &self.content_md5
    }
    pub fn meta(&self) -> &HashMap<String, String> {
        &self.meta
    }
    pub fn traffic_limit(&self) -> i64 {
        self.traffic_limit
    }
    pub fn forbid_overwrite(&self) -> bool {
        self.forbid_overwrite
    }
    pub fn tagging(&self) -> &str {
        &self.tagging
    }

    pub fn notification_custom_parameters(&self) -> &str {
        &self.notification_custom_parameters
    }

    pub fn detect_mime_type(&self) -> bool {
        self.detect_mime_type
    }

    pub fn object_expires(&self) -> i64 {
        self.object_expires
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_url(&mut self, url: impl Into<String>) {
        self.url = url.into();
    }

    pub fn set_ignore_same_key(&mut self, ignore_same_key: bool) {
        self.ignore_same_key = ignore_same_key;
    }

    pub fn set_content_md5(&mut self, content_md5: impl Into<String>) {
        self.content_md5 = content_md5.into();
    }
    pub fn set_meta(&mut self, meta: impl Into<HashMap<String, String>>) {
        self.meta = meta.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.traffic_limit = traffic_limit;
    }
    pub fn set_forbid_overwrite(&mut self, forbid_overwrite: bool) {
        self.forbid_overwrite = forbid_overwrite;
    }
    pub fn set_tagging(&mut self, tagging: impl Into<String>) {
        self.tagging = tagging.into();
    }

    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.notification_custom_parameters = notification_custom_parameters.into();
    }

    pub fn set_detect_mime_type(&mut self, detect_mime_type: bool) {
        self.detect_mime_type = detect_mime_type;
    }

    pub fn set_object_expires(&mut self, object_expires: i64) {
        self.object_expires = object_expires;
    }
}

impl InputDescriptor for FetchObjectInput {
    fn operation(&self) -> &str {
        "FetchObject"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for FetchObjectInput
where
    B: BuildBufferReader,
{
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.url == "" {
            return Err(TosError::client_error("empty url"));
        }

        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_key()?;
                request.method = HttpMethodPost;
                let header = &mut request.header;
                header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                set_http_basic_header_for_fetch(header, config_holder.disable_encoding_meta, self);
                set_acl_header(header, self);
                set_sse_header(header, self)?;
                set_ssec_header(header, &self.server_side_encryption, self)?;
                set_if_match_header(header, self);
                set_object_lock_header(header, self);
                request.meta = trans_meta(&self.meta, config_holder.disable_encoding_meta);
                set_misc_header_for_fetch(header, self);
                if self.forbid_overwrite {
                    header.insert(HEADER_FORBID_OVERWRITE, self.forbid_overwrite.to_string());
                }
                if self.traffic_limit > 0 {
                    header.insert(HEADER_TRAFFIC_LIMIT, self.traffic_limit.to_string());
                }
                if self.tagging != "" {
                    header.insert(HEADER_TAGGING, self.tagging.clone());
                }

                if self.object_expires >= 0 {
                    header.insert(HEADER_OBJECT_EXPIRES, self.object_expires.to_string());
                }
                if self.notification_custom_parameters != "" {
                    header.insert(HEADER_NOTIFICATION_CUSTOM_PARAMETERS, self.notification_custom_parameters.to_string());
                }
                if self.detect_mime_type {
                    header.insert(HEADER_FETCH_DETECT_MIME_TYPE, "true".to_string());
                }
                request.query = Some(HashMap::from([("fetch", "".to_string())]));
                Ok(request)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct FetchObjectOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) etag: String,
    pub(crate) version_id: String,
    pub(crate) ssec_algorithm: String,
    pub(crate) ssec_key_md5: String,
    pub(crate) server_side_encryption: String,
    pub(crate) server_side_encryption_key_id: String,
    pub(crate) source_content_type: String,
    pub(crate) source_content_length: i64,
    pub(crate) md5: String,
}

impl FetchObjectOutput {
    pub fn etag(&self) -> &str {
        &self.etag
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
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

    pub fn source_content_type(&self) -> &str {
        &self.source_content_type
    }

    pub fn source_content_length(&self) -> i64 {
        self.source_content_length
    }

    pub fn md5(&self) -> &str {
        &self.md5
    }
}

#[derive(
    Debug,
    Clone,
    HttpBasicHeader,
    AclHeader,
    SseHeader,
    SsecHeader,
    MiscHeader,
    ObjectLock,
    IfMatch,
    Serialize,
    GenericInput
)]
pub struct PutFetchTaskInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(rename = "Object")]
    pub(crate) key: String,
    #[serde(rename = "URL")]
    pub(crate) url: String,
    #[serde(rename = "IgnoreSameKey")]
    #[serde(skip_serializing_if = "<&bool as std::ops::Not>::not")]
    pub(crate) ignore_same_key: bool,
    #[serde(rename = "ContentMD5")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) content_md5: String,
    #[serde(rename = "CallbackUrl")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) callback_url: String,
    #[serde(rename = "CallbackHost")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) callback_host: String,
    #[serde(rename = "CallbackBodyType")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) callback_body_type: String,
    #[serde(rename = "CallbackBody")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) callback_body: String,
    #[serde(skip)]
    pub(crate) content_length: i64,
    #[serde(skip)]
    pub(crate) cache_control: String,
    #[serde(skip)]
    pub(crate) content_disposition: String,
    #[serde(skip)]
    pub(crate) content_encoding: String,
    #[serde(skip)]
    pub(crate) content_language: String,
    #[serde(skip)]
    pub(crate) content_type: String,
    #[serde(skip)]
    pub(crate) expires: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub(crate) acl: Option<ACLType>,
    #[serde(skip)]
    pub(crate) grant_full_control: String,
    #[serde(skip)]
    pub(crate) grant_read: String,
    #[serde(skip)]
    pub(crate) grant_read_acp: String,
    #[serde(skip)]
    pub(crate) grant_write: String,
    #[serde(skip)]
    pub(crate) grant_write_acp: String,
    #[serde(skip)]
    pub(crate) ssec_algorithm: String,
    #[serde(skip)]
    pub(crate) ssec_key: String,
    #[serde(skip)]
    pub(crate) ssec_key_md5: String,
    #[serde(skip)]
    pub(crate) server_side_encryption: String,
    #[serde(skip)]
    pub(crate) server_side_encryption_key_id: String,
    #[serde(skip)]
    pub(crate) meta: HashMap<String, String>,
    #[serde(skip)]
    pub(crate) website_redirect_location: String,
    #[serde(skip)]
    pub(crate) storage_class: Option<StorageClassType>,
    #[serde(skip)]
    pub(crate) traffic_limit: i64,
    #[serde(skip)]
    pub(crate) forbid_overwrite: bool,
    #[serde(skip)]
    pub(crate) if_match: String,
    #[serde(skip)]
    pub(crate) if_none_match: String,
    #[serde(skip)]
    pub(crate) tagging: String,
    #[serde(skip)]
    pub(crate) notification_custom_parameters: String,
    #[serde(skip)]
    pub(crate) detect_mime_type: bool,
    #[serde(skip)]
    pub(crate) object_expires: i64,
    #[serde(skip)]
    pub(crate) object_lock_mode: Option<ObjectLockModeType>,
    #[serde(skip)]
    pub(crate) object_lock_retain_util_date: Option<DateTime<Utc>>,
}

impl Default for PutFetchTaskInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            key: "".to_string(),
            url: "".to_string(),
            ignore_same_key: false,
            content_md5: "".to_string(),
            callback_url: "".to_string(),
            callback_host: "".to_string(),
            callback_body_type: "".to_string(),
            callback_body: "".to_string(),
            content_length: -1,
            cache_control: "".to_string(),
            content_disposition: "".to_string(),
            content_encoding: "".to_string(),
            content_language: "".to_string(),
            content_type: "".to_string(),
            expires: None,
            acl: None,
            grant_full_control: "".to_string(),
            grant_read: "".to_string(),
            grant_read_acp: "".to_string(),
            grant_write: "".to_string(),
            grant_write_acp: "".to_string(),
            ssec_algorithm: "".to_string(),
            ssec_key: "".to_string(),
            ssec_key_md5: "".to_string(),
            server_side_encryption: "".to_string(),
            server_side_encryption_key_id: "".to_string(),
            meta: Default::default(),
            website_redirect_location: "".to_string(),
            storage_class: None,
            traffic_limit: 0,
            forbid_overwrite: false,
            if_match: "".to_string(),
            if_none_match: "".to_string(),
            tagging: "".to_string(),
            notification_custom_parameters: "".to_string(),
            detect_mime_type: false,
            object_expires: -1,
            object_lock_mode: None,
            object_lock_retain_util_date: None,
        }
    }
}

impl PutFetchTaskInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input
    }
    pub fn new_with_url(bucket: impl Into<String>, key: impl Into<String>, url: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.url = url.into();
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn ignore_same_key(&self) -> bool {
        self.ignore_same_key
    }

    pub fn content_md5(&self) -> &str {
        &self.content_md5
    }
    pub fn callback_url(&self) -> &str {
        &self.callback_url
    }

    pub fn callback_host(&self) -> &str {
        &self.callback_host
    }

    pub fn callback_body_type(&self) -> &str {
        &self.callback_body_type
    }

    pub fn callback_body(&self) -> &str {
        &self.callback_body
    }

    pub fn meta(&self) -> &HashMap<String, String> {
        &self.meta
    }
    pub fn traffic_limit(&self) -> i64 {
        self.traffic_limit
    }
    pub fn forbid_overwrite(&self) -> bool {
        self.forbid_overwrite
    }
    pub fn tagging(&self) -> &str {
        &self.tagging
    }

    pub fn notification_custom_parameters(&self) -> &str {
        &self.notification_custom_parameters
    }

    pub fn detect_mime_type(&self) -> bool {
        self.detect_mime_type
    }

    pub fn object_expires(&self) -> i64 {
        self.object_expires
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_url(&mut self, url: impl Into<String>) {
        self.url = url.into();
    }

    pub fn set_ignore_same_key(&mut self, ignore_same_key: bool) {
        self.ignore_same_key = ignore_same_key;
    }
    pub fn set_content_md5(&mut self, content_md5: impl Into<String>) {
        self.content_md5 = content_md5.into();
    }

    pub fn set_callback_url(&mut self, callback_url: impl Into<String>) {
        self.callback_url = callback_url.into();
    }

    pub fn set_callback_host(&mut self, callback_host: impl Into<String>) {
        self.callback_host = callback_host.into();
    }

    pub fn set_callback_body_type(&mut self, callback_body_type: impl Into<String>) {
        self.callback_body_type = callback_body_type.into();
    }

    pub fn set_callback_body(&mut self, callback_body: impl Into<String>) {
        self.callback_body = callback_body.into();
    }
    pub fn set_meta(&mut self, meta: impl Into<HashMap<String, String>>) {
        self.meta = meta.into();
    }
    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.traffic_limit = traffic_limit;
    }
    pub fn set_forbid_overwrite(&mut self, forbid_overwrite: bool) {
        self.forbid_overwrite = forbid_overwrite;
    }
    pub fn set_tagging(&mut self, tagging: impl Into<String>) {
        self.tagging = tagging.into();
    }

    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.notification_custom_parameters = notification_custom_parameters.into();
    }

    pub fn set_detect_mime_type(&mut self, detect_mime_type: bool) {
        self.detect_mime_type = detect_mime_type;
    }

    pub fn set_object_expires(&mut self, object_expires: i64) {
        self.object_expires = object_expires;
    }
}

impl InputDescriptor for PutFetchTaskInput {
    fn operation(&self) -> &str {
        "PutFetchTask"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for PutFetchTaskInput
where
    B: BuildBufferReader,
{
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.url == "" {
            return Err(TosError::client_error("empty url"));
        }

        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_key()?;
                request.method = HttpMethodPost;
                let header = &mut request.header;
                header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                set_http_basic_header_for_fetch(header, config_holder.disable_encoding_meta, self);
                set_acl_header(header, self);
                set_sse_header(header, self)?;
                set_ssec_header(header, &self.server_side_encryption, self)?;
                set_if_match_header(header, self);
                set_object_lock_header(header, self);
                request.meta = trans_meta(&self.meta, config_holder.disable_encoding_meta);
                set_misc_header_for_fetch(header, self);
                if self.forbid_overwrite {
                    header.insert(HEADER_FORBID_OVERWRITE, self.forbid_overwrite.to_string());
                }
                if self.traffic_limit > 0 {
                    header.insert(HEADER_TRAFFIC_LIMIT, self.traffic_limit.to_string());
                }
                if self.tagging != "" {
                    header.insert(HEADER_TAGGING, self.tagging.clone());
                }

                if self.object_expires >= 0 {
                    header.insert(HEADER_OBJECT_EXPIRES, self.object_expires.to_string());
                }
                if self.notification_custom_parameters != "" {
                    header.insert(HEADER_NOTIFICATION_CUSTOM_PARAMETERS, self.notification_custom_parameters.to_string());
                }
                if self.detect_mime_type {
                    header.insert(HEADER_FETCH_DETECT_MIME_TYPE, "true".to_string());
                }
                request.query = Some(HashMap::from([("fetchTask", "".to_string())]));
                Ok(request)
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct PutFetchTaskOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "TaskId")]
    pub(crate) task_id: String,
}
impl PutFetchTaskOutput {
    pub fn task_id(&self) -> &str {
        &self.task_id
    }
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct GetFetchTaskInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) task_id: String,
}

impl GetFetchTaskInput {
    pub fn new(bucket: impl Into<String>, task_id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            task_id: task_id.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_task_id(&mut self, task_id: impl Into<String>) {
        self.task_id = task_id.into();
    }
}

impl InputDescriptor for GetFetchTaskInput {
    fn operation(&self) -> &str {
        "GetFetchTask"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
}

impl<B> InputTranslator<B> for GetFetchTaskInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.task_id == "" {
            return Err(TosError::client_error("empty task id"));
        }

        let mut request = self.trans_bucket()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(2);
        query.insert("fetchTask", "".to_string());
        map_insert(&mut query, QUERY_TASK_ID, &self.task_id);
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetFetchTaskOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "State")]
    #[serde(default)]
    pub(crate) state: String,
    #[serde(rename = "Err")]
    #[serde(default)]
    pub(crate) err: String,
    #[serde(rename = "Task")]
    pub(crate) task: Option<FetchTask>,
}
impl GetFetchTaskOutput {
    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn err(&self) -> &str {
        &self.err
    }

    pub fn task(&self) -> &Option<FetchTask> {
        &self.task
    }
}
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct FetchTask {
    #[serde(rename = "URL")]
    #[serde(default)]
    pub(crate) url: String,
    #[serde(rename = "IgnoreSameKey")]
    #[serde(default)]
    pub(crate) ignore_same_key: bool,
    #[serde(rename = "ContentMD5")]
    #[serde(default)]
    pub(crate) content_md5: String,
    #[serde(rename = "Bucket")]
    #[serde(default)]
    pub(crate) bucket: String,
    #[serde(rename = "Key")]
    #[serde(default)]
    pub(crate) key: String,
    #[serde(rename = "CallbackUrl")]
    #[serde(default)]
    pub(crate) callback_url: String,
    #[serde(rename = "CallbackHost")]
    #[serde(default)]
    pub(crate) callback_host: String,
    #[serde(rename = "CallbackBodyType")]
    #[serde(default)]
    pub(crate) callback_body_type: String,
    #[serde(rename = "CallbackBody")]
    #[serde(default)]
    pub(crate) callback_body: String,
    #[serde(rename = "StorageClass")]
    pub(crate) storage_class: Option<StorageClassType>,
    #[serde(rename = "Acl")]
    pub(crate) acl: Option<ACLType>,
    #[serde(rename = "GrantFullControl")]
    #[serde(default)]
    pub(crate) grant_fullcontrol: String,
    #[serde(rename = "GrantRead")]
    #[serde(default)]
    pub(crate) grant_read: String,
    #[serde(rename = "GrantReadAcp")]
    #[serde(default)]
    pub(crate) grant_read_acp: String,
    #[serde(rename = "GrantWrite")]
    #[serde(default)]
    pub(crate) grant_write: String,
    #[serde(rename = "GrantWriteAcp")]
    #[serde(default)]
    pub(crate) grant_write_acp: String,
    #[serde(rename = "SSECAlgorithm")]
    #[serde(default)]
    pub(crate) ssec_algorithm: String,
    #[serde(rename = "SSECKey")]
    #[serde(default)]
    pub(crate) ssec_key: String,
    #[serde(rename = "SSECKeyMd5")]
    #[serde(default)]
    pub(crate) ssec_key_md5: String,
    #[serde(rename = "UserMeta")]
    #[serde(default)]
    pub(crate) user_meta: Vec<UserMeta>,
    #[serde(skip)]
    pub(crate) meta: HashMap<String, String>,
}

impl FetchTask {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn ignore_same_key(&self) -> bool {
        self.ignore_same_key
    }

    pub fn content_md5(&self) -> &str {
        &self.content_md5
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn callback_body(&self) -> &str {
        &self.callback_body
    }

    pub fn callback_body_type(&self) -> &str {
        &self.callback_body_type
    }

    pub fn callback_host(&self) -> &str {
        &self.callback_host
    }

    pub fn callback_url(&self) -> &str {
        &self.callback_url
    }
    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
    }

    pub fn acl(&self) -> &Option<ACLType> {
        &self.acl
    }

    pub fn grant_fullcontrol(&self) -> &str {
        &self.grant_fullcontrol
    }

    pub fn grant_read(&self) -> &str {
        &self.grant_read
    }

    pub fn grant_read_acp(&self) -> &str {
        &self.grant_read_acp
    }

    pub fn grant_write(&self) -> &str {
        &self.grant_write
    }

    pub fn grant_write_acp(&self) -> &str {
        &self.grant_write_acp
    }

    pub fn ssec_algorithm(&self) -> &str {
        &self.ssec_algorithm
    }

    pub fn ssec_key(&self) -> &str {
        &self.ssec_key
    }

    pub fn ssec_key_md5(&self) -> &str {
        &self.ssec_key_md5
    }

    pub fn meta(&self) -> &HashMap<String, String> {
        &self.meta
    }
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct RenameObjectInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) new_key: String,
    pub(crate) recursive_mkdir: bool,
    pub(crate) forbid_overwrite: bool,
    pub(crate) notification_custom_parameters: String,
}

impl RenameObjectInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>, new_key: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            new_key: new_key.into(),
            recursive_mkdir: false,
            forbid_overwrite: false,
            notification_custom_parameters: "".to_string(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn new_key(&self) -> &str {
        &self.new_key
    }

    pub fn recursive_mkdir(&self) -> bool {
        self.recursive_mkdir
    }

    pub fn forbid_overwrite(&self) -> bool {
        self.forbid_overwrite
    }

    pub fn notification_custom_parameters(&self) -> &str {
        &self.notification_custom_parameters
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_new_key(&mut self, new_key: impl Into<String>) {
        self.new_key = new_key.into();
    }

    pub fn set_recursive_mkdir(&mut self, recursive_mkdir: bool) {
        self.recursive_mkdir = recursive_mkdir;
    }

    pub fn set_forbid_overwrite(&mut self, forbid_overwrite: bool) {
        self.forbid_overwrite = forbid_overwrite;
    }

    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.notification_custom_parameters = notification_custom_parameters.into();
    }
}

impl InputDescriptor for RenameObjectInput {
    fn operation(&self) -> &str {
        "RenameObject"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for RenameObjectInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.new_key == "" {
            return Err(TosError::client_error("empty new key"));
        }
        let mut request = self.trans_key()?;
        request.method = HttpMethodPut;
        if self.recursive_mkdir {
            request.header.insert(HEADER_RECURSIVE_MKDIR, self.recursive_mkdir.to_string());
        }
        if self.forbid_overwrite {
            request.header.insert(HEADER_FORBID_OVERWRITE, self.forbid_overwrite.to_string());
        }
        map_insert(&mut request.header, HEADER_NOTIFICATION_CUSTOM_PARAMETERS, &self.notification_custom_parameters);
        let mut query = HashMap::with_capacity(2);
        query.insert("rename", "".to_string());
        query.insert("name", self.new_key.to_string());
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct RenameObjectOutput {
    pub(crate) request_info: RequestInfo,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, GenericInput)]
pub struct RestoreObjectInput {
    #[serde(skip)]
    pub(crate) generic_input: GenericInput,
    #[serde(skip)]
    pub(crate) bucket: String,
    #[serde(skip)]
    pub(crate) key: String,
    #[serde(skip)]
    pub(crate) version_id: String,
    #[serde(rename = "Days")]
    #[serde(skip_serializing_if = "is_non_positive")]
    pub(crate) days: isize,
    #[serde(rename = "RestoreJobParameters")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) restore_job_parameters: Option<RestoreJobParameters>,
    #[serde(skip)]
    pub(crate) notification_custom_parameters: String,
}
impl RestoreObjectInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            version_id: "".to_string(),
            days: 0,
            restore_job_parameters: None,
            notification_custom_parameters: "".to_string(),
        }
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            version_id: version_id.into(),
            days: 0,
            restore_job_parameters: None,
            notification_custom_parameters: "".to_string(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn days(&self) -> isize {
        self.days
    }

    pub fn restore_job_parameters(&self) -> &Option<RestoreJobParameters> {
        &self.restore_job_parameters
    }

    pub fn notification_custom_parameters(&self) -> &str {
        &self.notification_custom_parameters
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }

    pub fn set_days(&mut self, days: isize) {
        self.days = days;
    }

    pub fn set_restore_job_parameters(&mut self, restore_job_parameters: impl Into<RestoreJobParameters>) {
        self.restore_job_parameters = Some(restore_job_parameters.into());
    }

    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.notification_custom_parameters = notification_custom_parameters.into();
    }
}

impl InputDescriptor for RestoreObjectInput {
    fn operation(&self) -> &str {
        "RestoreObject"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for RestoreObjectInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        match serde_json::to_string(self) {
            Err(e) => Err(TosError::client_error_with_cause("trans json error", GenericError::JsonError(e.to_string()))),
            Ok(json) => {
                let mut request = self.trans_key()?;
                request.method = HttpMethodPost;
                map_insert(&mut request.header, HEADER_NOTIFICATION_CUSTOM_PARAMETERS, &self.notification_custom_parameters);
                let mut query = HashMap::with_capacity(2);
                query.insert("restore", "".to_string());
                map_insert(&mut query, QUERY_VERSION_ID, &self.version_id);
                request.query = Some(query);
                request.header.insert(HEADER_CONTENT_MD5, base64_md5(&json));
                let (body, len) = B::new(json.into_bytes())?;
                request.body = Some(body);
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
                Ok(request)
            }
        }
    }
}


#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct RestoreJobParameters {
    #[serde(rename = "Tier")]
    #[serde(default)]
    pub(crate) tier: TierType,
}

impl RestoreJobParameters {
    pub fn new(tier: impl Into<TierType>) -> Self {
        Self {
            tier: tier.into(),
        }
    }

    pub fn tier(&self) -> &TierType {
        &self.tier
    }

    pub fn set_tier(&mut self, tier: impl Into<TierType>) {
        self.tier = tier.into();
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct RestoreObjectOutput {
    pub(crate) request_info: RequestInfo,
}

#[derive(
    Debug,
    Clone,
    HttpBasicHeader,
    AclHeader,
    GenericInput
)]
pub struct PutSymlinkInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) symlink_target_key: String,
    pub(crate) symlink_target_bucket: String,

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

    pub(crate) storage_class: Option<StorageClassType>,
    pub(crate) meta: HashMap<String, String>,
    pub(crate) forbid_overwrite: bool,
    pub(crate) tagging: String,
}

impl PutSymlinkInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>, symlink_target_key: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            symlink_target_key: symlink_target_key.into(),
            symlink_target_bucket: "".to_string(),
            content_length: -1,
            cache_control: "".to_string(),
            content_disposition: "".to_string(),
            content_encoding: "".to_string(),
            content_language: "".to_string(),
            content_type: "".to_string(),
            expires: None,
            acl: None,
            grant_full_control: "".to_string(),
            grant_read: "".to_string(),
            grant_read_acp: "".to_string(),
            grant_write: "".to_string(),
            grant_write_acp: "".to_string(),
            storage_class: None,
            meta: Default::default(),
            forbid_overwrite: false,
            tagging: "".to_string(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn symlink_target_key(&self) -> &str {
        &self.symlink_target_key
    }

    pub fn symlink_target_bucket(&self) -> &str {
        &self.symlink_target_bucket
    }

    pub fn storage_class(&self) -> &Option<StorageClassType> {
        &self.storage_class
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

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_symlink_target_key(&mut self, symlink_target_key: impl Into<String>) {
        self.symlink_target_key = symlink_target_key.into();
    }

    pub fn set_symlink_target_bucket(&mut self, symlink_target_bucket: impl Into<String>) {
        self.symlink_target_bucket = symlink_target_bucket.into();
    }

    pub fn set_storage_class(&mut self, storage_class: impl Into<StorageClassType>) {
        self.storage_class = Some(storage_class.into());
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
}

impl InputDescriptor for PutSymlinkInput {
    fn operation(&self) -> &str {
        "PutSymlink"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for PutSymlinkInput {
    fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.symlink_target_key == "" {
            return Err(TosError::client_error("empty symlink target key"));
        }

        let mut request = self.trans_key()?;
        request.method = HttpMethodPut;
        let header = &mut request.header;
        set_http_basic_header(header, config_holder.disable_encoding_meta, self);
        set_acl_header(header, self);
        request.meta = trans_meta(&self.meta, config_holder.disable_encoding_meta);
        if let Some(storage_class) = &self.storage_class {
            header.insert(HEADER_STORAGE_CLASS, storage_class.as_str().to_string());
        }
        if self.forbid_overwrite {
            header.insert(HEADER_FORBID_OVERWRITE, self.forbid_overwrite.to_string());
        }
        map_insert(header, HEADER_TAGGING, &self.tagging);
        map_insert(header, HEADER_SYMLINK_TARGET, url_encode_with_safe(&self.symlink_target_key, "/").as_str());
        map_insert(header, HEADER_SYMLINK_BUCKET, &self.symlink_target_bucket);
        let mut query = HashMap::with_capacity(2);
        query.insert("symlink", "".to_string());
        request.query = Some(query);
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct PutSymlinkOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) version_id: String,
}

impl PutSymlinkOutput {
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
}

#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct GetSymlinkInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) version_id: String,
}

impl GetSymlinkInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            version_id: "".to_string(),
        }
    }
    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            version_id: version_id.into(),
        }
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version_id(&self) -> &str {
        &self.version_id
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }
}

impl InputDescriptor for GetSymlinkInput {
    fn operation(&self) -> &str {
        "GetSymlink"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for GetSymlinkInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(2);
        query.insert("symlink", "".to_string());
        if self.version_id != "" {
            query.insert(QUERY_VERSION_ID, self.version_id.to_string());
        }
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct GetSymlinkOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) version_id: String,
    pub(crate) symlink_target_key: String,
    pub(crate) symlink_target_bucket: String,
    pub(crate) etag: String,
    pub(crate) last_modified: Option<DateTime<Utc>>,
}

impl GetSymlinkOutput {
    pub fn request_info(&self) -> &RequestInfo {
        &self.request_info
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn symlink_target_key(&self) -> &str {
        &self.symlink_target_key
    }

    pub fn symlink_target_bucket(&self) -> &str {
        &self.symlink_target_bucket
    }
    pub fn etag(&self) -> &str {
        &self.etag
    }
    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct GetFileStatusInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
}

impl GetFileStatusInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
        }
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }
    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }
}

impl InputDescriptor for GetFileStatusInput {
    fn operation(&self) -> &str {
        "GetFileStatus"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for GetFileStatusInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodGet;
        let mut query = HashMap::with_capacity(2);
        query.insert("stat", "".to_string());
        request.query = Some(query);
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo, Deserialize)]
pub struct GetFileStatusOutput {
    #[serde(skip)]
    pub(crate) request_info: RequestInfo,
    #[serde(rename = "Key")]
    #[serde(default)]
    pub(crate) key: String,
    #[serde(rename = "Size")]
    #[serde(default)]
    pub(crate) size: i64,
    #[serde(rename = "LastModified")]
    #[serde(default)]
    pub(crate) last_modified_string: Option<String>,
    #[serde(skip)]
    pub(crate) last_modified: Option<DateTime<Utc>>,
    #[serde(rename = "CRC32")]
    #[serde(default)]
    pub(crate) crc32: String,
    #[serde(rename = "CRC64")]
    #[serde(default)]
    pub(crate) crc64: String,
    #[serde(rename = "ETag")]
    #[serde(default)]
    pub(crate) etag: String,
    #[serde(rename = "Type")]
    #[serde(default)]
    pub(crate) object_type: String,
}

impl GetFileStatusOutput {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn size(&self) -> i64 {
        self.size
    }

    pub fn last_modified(&self) -> Option<DateTime<Utc>> {
        self.last_modified
    }

    pub fn crc32(&self) -> &str {
        &self.crc32
    }

    pub fn crc64(&self) -> &str {
        &self.crc64
    }

    pub fn etag(&self) -> &str {
        &self.etag
    }

    pub fn object_type(&self) -> &str {
        &self.object_type
    }
}
#[derive(Debug, Clone, GenericInput)]
#[handle_content]
pub(crate) struct ModifyObjectInput<B> {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) offset: i64,
    pub(crate) content: Arc<RefCell<Option<B>>>,

    pub(crate) content_length: i64,
    pub(crate) traffic_limit: i64,
    pub(crate) async_data_transfer_listener: Option<async_channel::Sender<DataTransferStatus>>,
    pub(crate) notification_custom_parameters: String,
    pub(crate) if_match: String,

    pub(crate) pre_hash_crc64ecma: u64,
}

unsafe impl<B> Sync for ModifyObjectInput<B> {}

unsafe impl<B> Send for ModifyObjectInput<B> {}

impl<B> Default for ModifyObjectInput<B> {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            key: "".to_string(),
            offset: 0,
            content: Arc::new(RefCell::new(None)),
            content_length: -1,
            traffic_limit: 0,
            async_data_transfer_listener: None,
            notification_custom_parameters: "".to_string(),
            if_match: "".to_string(),
            pre_hash_crc64ecma: 0,
        }
    }
}

impl<B> ModifyObjectInput<B>
{
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input
    }

    pub fn new_with_content(bucket: impl Into<String>, key: impl Into<String>, content: impl Into<B>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.set_content(content);
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn content(&self) -> Ref<Option<B>> {
        self.content.borrow()
    }

    pub fn offset(&self) -> i64 {
        self.offset
    }

    pub fn content_length(&self) -> i64 {
        self.content_length
    }

    pub fn traffic_limit(&self) -> i64 {
        self.traffic_limit
    }

    pub fn notification_custom_parameters(&self) -> &str {
        &self.notification_custom_parameters
    }
    pub fn if_match(&self) -> &str {
        &self.if_match
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_offset(&mut self, offset: i64) {
        self.offset = offset;
    }

    pub fn set_content(&mut self, content: impl Into<B>)
    {
        self.content = Arc::new(RefCell::new(Some(content.into())));
    }

    pub fn set_content_length(&mut self, content_length: i64) {
        self.content_length = content_length;
    }

    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.traffic_limit = traffic_limit;
    }

    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.notification_custom_parameters = notification_custom_parameters.into();
    }

    pub fn set_if_match(&mut self, if_match: impl Into<String>) {
        self.if_match = if_match.into();
    }
}

impl<B> InputDescriptor for ModifyObjectInput<B> {
    fn operation(&self) -> &str {
        "ModifyObject"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for ModifyObjectInput<B> {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.offset < 0
        {
            return Err(TosError::client_error("invalid offset for modify object"));
        }
        let mut request = self.trans_key()?;
        request.method = HttpMethodPost;

        if let Some(ref adts) = self.async_data_transfer_listener {
            if request.request_context.is_some() {
                request.request_context.as_mut().unwrap().async_data_transfer_listener = Some(adts.clone());
            } else {
                let mut rc = RequestContext::default();
                rc.async_data_transfer_listener = Some(adts.clone());
                request.request_context = Some(rc);
            }
        }

        if self.pre_hash_crc64ecma > 0 {
            if request.request_context.is_some() {
                request.request_context.as_mut().unwrap().init_crc64 = Some(self.pre_hash_crc64ecma);
            } else {
                let mut rc = RequestContext::default();
                rc.init_crc64 = Some(self.pre_hash_crc64ecma);
                request.request_context = Some(rc);
            }
        }

        let header = &mut request.header;
        if self.content_length >= 0 {
            header.insert(HEADER_CONTENT_LENGTH, self.content_length.to_string());
        }
        if self.traffic_limit > 0 {
            header.insert(HEADER_TRAFFIC_LIMIT, self.traffic_limit.to_string());
        }
        map_insert(header, HEADER_NOTIFICATION_CUSTOM_PARAMETERS, &self.notification_custom_parameters);
        map_insert(header, HEADER_X_IF_MATCH, &self.if_match);
        let mut query = HashMap::with_capacity(2);
        query.insert("modify", "".to_string());
        query.insert(QUERY_OFFSET, self.offset.to_string());
        request.query = Some(query);
        request.body = self.content.take();
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub(crate) struct ModifyObjectOutput {
    pub(crate) request_info: RequestInfo,
    pub(crate) next_modify_offset: i64,
    pub(crate) hash_crc64ecma: u64,
}

impl ModifyObjectOutput {
    pub fn next_modify_offset(&self) -> i64 {
        self.next_modify_offset
    }

    pub fn hash_crc64ecma(&self) -> u64 {
        self.hash_crc64ecma
    }
}

#[derive(Debug, Clone, GenericInput)]
pub(crate) struct ModifyObjectFromBufferInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) offset: i64,
    pub(crate) content: Option<Vec<u8>>,

    pub(crate) content_length: i64,
    pub(crate) traffic_limit: i64,
    pub(crate) async_data_transfer_listener: Option<async_channel::Sender<DataTransferStatus>>,
    pub(crate) notification_custom_parameters: String,
    pub(crate) if_match: String,

    pub(crate) pre_hash_crc64ecma: u64,
}

impl Default for ModifyObjectFromBufferInput {
    fn default() -> Self {
        Self {
            generic_input: Default::default(),
            bucket: "".to_string(),
            key: "".to_string(),
            offset: 0,
            content: None,
            content_length: -1,
            traffic_limit: 0,
            async_data_transfer_listener: None,
            notification_custom_parameters: "".to_string(),
            if_match: "".to_string(),
            pre_hash_crc64ecma: 0,
        }
    }
}

impl ModifyObjectFromBufferInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input
    }
    pub fn new_with_offset(bucket: impl Into<String>, key: impl Into<String>, offset: i64) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.offset = offset;
        input
    }
    pub fn new_with_content(bucket: impl Into<String>, key: impl Into<String>, content: impl AsRef<[u8]>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.set_content(content);
        input
    }
    pub fn new_with_offset_content(bucket: impl Into<String>, key: impl Into<String>, offset: i64, content: impl AsRef<[u8]>) -> Self {
        let mut input = Self::default();
        input.bucket = bucket.into();
        input.key = key.into();
        input.offset = offset;
        input.set_content(content);
        input
    }
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn content(&self) -> &Option<impl AsRef<[u8]>> {
        &self.content
    }
    pub fn offset(&self) -> i64 {
        self.offset
    }

    pub fn content_length(&self) -> i64 {
        self.content_length
    }

    pub fn traffic_limit(&self) -> i64 {
        self.traffic_limit
    }

    pub fn notification_custom_parameters(&self) -> &str {
        &self.notification_custom_parameters
    }
    pub fn if_match(&self) -> &str {
        &self.if_match
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_offset(&mut self, offset: i64) {
        self.offset = offset;
    }

    pub fn set_content(&mut self, content: impl AsRef<[u8]>) {
        self.content = Some(content.as_ref().to_owned());
    }
    pub fn set_content_nocopy(&mut self, content: impl Into<Vec<u8>>) {
        self.content = Some(content.into());
    }
    pub fn set_content_length(&mut self, content_length: i64) {
        self.content_length = content_length;
    }

    pub fn set_traffic_limit(&mut self, traffic_limit: i64) {
        self.traffic_limit = traffic_limit;
    }

    pub fn set_notification_custom_parameters(&mut self, notification_custom_parameters: impl Into<String>) {
        self.notification_custom_parameters = notification_custom_parameters.into();
    }
    pub fn set_if_match(&mut self, if_match: impl Into<String>) {
        self.if_match = if_match.into();
    }
}

impl InputDescriptor for ModifyObjectFromBufferInput {
    fn operation(&self) -> &str {
        "ModifyObjectFromBuffer"
    }
    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }
    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for ModifyObjectFromBufferInput
where
    B: BuildBufferReader,
{
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        if self.offset < 0
        {
            return Err(TosError::client_error("invalid offset for modify object"));
        }
        let mut request = self.trans_key()?;
        request.method = HttpMethodPost;

        if let Some(ref adts) = self.async_data_transfer_listener {
            if request.request_context.is_some() {
                request.request_context.as_mut().unwrap().async_data_transfer_listener = Some(adts.clone());
            } else {
                let mut rc = RequestContext::default();
                rc.async_data_transfer_listener = Some(adts.clone());
                request.request_context = Some(rc);
            }
        }

        if self.pre_hash_crc64ecma > 0 {
            if request.request_context.is_some() {
                request.request_context.as_mut().unwrap().init_crc64 = Some(self.pre_hash_crc64ecma);
            } else {
                let mut rc = RequestContext::default();
                rc.init_crc64 = Some(self.pre_hash_crc64ecma);
                request.request_context = Some(rc);
            }
        }

        let header = &mut request.header;
        if self.content_length >= 0 {
            header.insert(HEADER_CONTENT_LENGTH, self.content_length.to_string());
        }
        if self.traffic_limit > 0 {
            header.insert(HEADER_TRAFFIC_LIMIT, self.traffic_limit.to_string());
        }
        map_insert(header, HEADER_NOTIFICATION_CUSTOM_PARAMETERS, &self.notification_custom_parameters);
        map_insert(header, HEADER_X_IF_MATCH, &self.if_match);
        let mut query = HashMap::with_capacity(2);
        query.insert("modify", "".to_string());
        query.insert(QUERY_OFFSET, self.offset.to_string());
        request.query = Some(query);
        if let Some(content) = &self.content {
            let (body, len) = B::new(content.to_owned())?;
            request.body = Some(body);
            if self.content_length < 0 {
                request.header.insert(HEADER_CONTENT_LENGTH, len.to_string());
            }
        }
        Ok(request)
    }
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct DoesObjectExistInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) version_id: String,
    // todo xsj
    pub(crate) is_only_in_tos: bool,
}

impl DoesObjectExistInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            version_id: "".to_string(),
            is_only_in_tos: false,
        }
    }

    pub fn new_with_version_id(bucket: impl Into<String>, key: impl Into<String>, version_id: impl Into<String>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            version_id: version_id.into(),
            is_only_in_tos: false,
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub(crate) fn is_only_in_tos(&self) -> bool {
        self.is_only_in_tos
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_version_id(&mut self, version_id: impl Into<String>) {
        self.version_id = version_id.into();
    }

    pub(crate) fn set_is_only_in_tos(&mut self, is_only_in_tos: bool) {
        self.is_only_in_tos = is_only_in_tos;
    }
}
#[derive(Debug, Clone, PartialEq, Default, GenericInput)]
pub struct SetObjectTimeInput {
    pub(crate) generic_input: GenericInput,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) modify_timestamp: DateTime<Utc>,
}

impl SetObjectTimeInput {
    pub fn new(bucket: impl Into<String>, key: impl Into<String>, modify_timestamp: impl Into<DateTime<Utc>>) -> Self {
        Self {
            generic_input: Default::default(),
            bucket: bucket.into(),
            key: key.into(),
            modify_timestamp: modify_timestamp.into(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn modify_timestamp(&self) -> DateTime<Utc> {
        self.modify_timestamp
    }

    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = bucket.into();
    }

    pub fn set_key(&mut self, key: impl Into<String>) {
        self.key = key.into();
    }

    pub fn set_modify_timestamp(&mut self, modify_timestamp: impl Into<DateTime<Utc>>) {
        self.modify_timestamp = modify_timestamp.into();
    }
}

impl InputDescriptor for SetObjectTimeInput {
    fn operation(&self) -> &str {
        "SetObjectTime"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Ok(&self.bucket)
    }

    fn key(&self) -> Result<&str, TosError> {
        Ok(&self.key)
    }
}

impl<B> InputTranslator<B> for SetObjectTimeInput {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_key()?;
        request.method = HttpMethodPost;
        let seconds = self.modify_timestamp.timestamp();
        let ns = self.modify_timestamp.timestamp_subsec_nanos();
        request.header.insert(HEADER_MODIFY_TIMESTAMP, seconds.to_string());
        request.header.insert(HEADER_MODIFY_TIMESTAMP_NS, ns.to_string());

        let mut query = HashMap::with_capacity(1);
        query.insert("time", "".to_string());
        request.query = Some(query);
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Default, RequestInfo)]
pub struct SetObjectTimeOutput {
    pub(crate) request_info: RequestInfo,
}