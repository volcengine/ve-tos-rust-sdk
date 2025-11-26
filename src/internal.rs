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

use crate::common::{GenericInputTrait, Meta, RequestInfo};
use crate::config::ConfigHolder;
use crate::constant::{ACCOUNT_REGEX, ALGORITHM_AES256, ALGORITHM_KMS, ALLOWED_IN_URL, ALL_UPLOAD_OPERATIONS, AUTO_RECOGNIZE_CONTENT_TYPE_OPERATIONS, BASE_DELAY_MS, BUCKET_REGEX, CHINESE_REGEX, DEFAULT_READ_BUFFER_SIZE, GET_OBJECT_TO_FILE_OPERATION, HEADER_ACL, HEADER_CACHE_CONTROL, HEADER_CALLBACK, HEADER_CALLBACK_VAR, HEADER_CONTENT_DISPOSITION, HEADER_CONTENT_ENCODING, HEADER_CONTENT_LANGUAGE, HEADER_CONTENT_LENGTH, HEADER_CONTENT_TYPE, HEADER_COPY_SOURCE, HEADER_COPY_SOURCE_IF_MATCH, HEADER_COPY_SOURCE_IF_MODIFIED_SINCE, HEADER_COPY_SOURCE_IF_NONE_MATCH, HEADER_COPY_SOURCE_IF_UNMODIFIED_SINCE, HEADER_COPY_SOURCE_SSEC_ALGORITHM, HEADER_COPY_SOURCE_SSEC_KEY, HEADER_COPY_SOURCE_SSEC_KEY_MD5, HEADER_EXPIRES, HEADER_FETCH_CACHE_CONTROL, HEADER_FETCH_CONTENT_DISPOSITION, HEADER_FETCH_CONTENT_ENCODING, HEADER_FETCH_CONTENT_LANGUAGE, HEADER_FETCH_CONTENT_TYPE, HEADER_FETCH_EXPIRES, HEADER_FETCH_WEBSITE_REDIRECT_LOCATION, HEADER_GRANT_FULL_CONTROL, HEADER_GRANT_READ, HEADER_GRANT_READ_ACP, HEADER_GRANT_WRITE, HEADER_GRANT_WRITE_ACP, HEADER_IF_MATCH, HEADER_IF_MODIFIED_SINCE, HEADER_IF_NONE_MATCH, HEADER_IF_UNMODIFIED_SINCE, HEADER_OBJECT_LOCK_MODE, HEADER_OBJECT_LOCK_RETAIN_UNTIL_DATE, HEADER_PREFIX_META, HEADER_RETRY_AFTER, HEADER_RETRY_AFTER_LOWER, HEADER_SERVER_SIDE_ENCRYPTION, HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID, HEADER_SSEC_ALGORITHM, HEADER_SSEC_KEY, HEADER_SSEC_KEY_MD5, HEADER_STORAGE_CLASS, HEADER_WEBSITE_REDIRECT_LOCATION, ISO8601_DATE_FORMAT, ISO8601_DATE_FORMAT_TRUNCATED, MAX_DELAY_MS, MAX_READ_BUFFER_SIZE_FOR_JSON, MIME_TYPES, NO_IDEMPOTENT_OPERATIONS, QUERY_DELIMITER, QUERY_DOC_DST_TYPE, QUERY_DOC_PAGE, QUERY_DOC_SRC_TYPE, QUERY_ENCODING_TYPE, QUERY_PART_NUMBER, QUERY_PREFIX, QUERY_PROCESS, QUERY_RESPONSE_CACHE_CONTROL, QUERY_RESPONSE_CONTENT_DISPOSITION, QUERY_RESPONSE_CONTENT_ENCODING, QUERY_RESPONSE_CONTENT_LANGUAGE, QUERY_RESPONSE_CONTENT_TYPE, QUERY_RESPONSE_EXPIRES, QUERY_SAVE_BUCKET, QUERY_SAVE_OBJECT, QUERY_UPLOAD_ID, RFC1123_DATE_FORMAT, STREAM_UPLOAD_OPERATIONS};
use crate::enumeration::{ACLType, DocPreviewDstType, DocPreviewSrcType, ObjectLockModeType, StorageClassType};
use crate::error::TosError::{TosClientError, TosServerError};
use crate::error::{GenericError, TosError};
use crate::http::{HttpRequest, HttpResponse};
use crate::reader::{read_at_most, InternalReader};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use md5::Md5;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Certificate, Identity};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use std::sync::Arc;
use std::{thread, time};
use urlencoding::encode;

pub(crate) fn url_encode_chinese(input: &str) -> String {
    let mut temp = String::with_capacity(input.len() * 2);
    let mut buf;
    for item in input.chars() {
        buf = item.to_string();
        if CHINESE_REGEX.is_match(buf.as_str()) {
            temp.push_str(encode(buf.as_str()).as_ref());
        } else {
            temp.push(item);
        }
    }
    temp
}

pub(crate) fn url_encode(input: &str) -> String {
    url_encode_with_safe(input, "")
}

pub(crate) fn url_encode_with_safe(input: &str, safe: &str) -> String {
    if input == "" {
        return "".to_string();
    }

    let mut temp = String::with_capacity(input.len() * 2);
    for i in input.as_bytes() {
        if let Some(_) = ALLOWED_IN_URL.find(*i as char) {
            temp.push(*i as char);
        } else {
            if safe != "" {
                if let Some(_) = safe.find(*i as char) {
                    temp.push(*i as char);
                    continue;
                }
            }
            temp.push('%');
            temp.push_str(format!("{:X}", i).as_str());
        }
    }

    return temp;
}

pub(crate) fn base64(data: impl AsRef<[u8]>) -> String {
    BASE64_STANDARD.encode(data)
}

pub(crate) fn base64_md5(input: impl AsRef<[u8]>) -> String {
    BASE64_STANDARD.encode(Md5::digest(input))
}

pub(crate) fn hex_md5(input: impl AsRef<[u8]>) -> String {
    hex::encode(Md5::digest(input))
}

pub(crate) fn hex_sha256(input: impl Into<String>) -> String {
    hex::encode(Sha256::digest(input.into()))
}

pub(crate) fn hex(input: impl AsRef<[u8]>) -> String {
    hex::encode(input)
}

pub(crate) fn hmac_sha256(input: impl AsRef<[u8]>, sign_key: impl AsRef<[u8]>) -> Result<impl AsRef<[u8]>, TosError> {
    match Hmac::<Sha256>::new_from_slice(sign_key.as_ref()) {
        Ok(mut m) => {
            m.update(input.as_ref());
            Ok(m.finalize().into_bytes())
        }
        Err(e) => {
            Err(TosError::client_error_with_cause("new hmac sha256 error", GenericError::DefaultError(e.to_string())))
        }
    }
}

pub(crate) fn parse_date_time_rfc1123(input: &str) -> Result<Option<DateTime<Utc>>, TosError> {
    if input == "" {
        return Ok(None);
    }
    match DateTime::parse_from_rfc2822(input) {
        Ok(dt) => Ok(Some(dt.to_utc())),
        Err(e) => Err(TosError::client_error_with_cause(format!("parse date time error with input {}", input), GenericError::DateTimeParseError(e)))
    }
}

pub(crate) fn parse_date_time_iso8601(input: &str) -> Result<Option<DateTime<Utc>>, TosError> {
    if input == "" {
        return Ok(None);
    }
    match DateTime::parse_from_rfc3339(input) {
        Ok(dt) => Ok(Some(dt.to_utc())),
        Err(e) => Err(TosError::client_error_with_cause(format!("parse date time error with input {}", input), GenericError::DateTimeParseError(e)))
    }
}

pub(crate) fn map_insert<'a>(header: &mut HashMap<&'a str, String>, key: &'a str, value: &str) {
    if value == "" {
        return;
    }
    header.insert(key, value.to_string());
}

pub(crate) fn map_insert_string<'a>(header: &mut HashMap<&'a str, String>, key: &'a str, value: String) {
    if value == "" {
        return;
    }
    header.insert(key, value);
}

pub(crate) fn set_acl_header<I: AclHeader>(header: &mut HashMap<&str, String>, input: &I) {
    if let Some(x) = input.acl() {
        header.insert(HEADER_ACL, x.as_str().to_string());
    }
    map_insert(header, HEADER_GRANT_FULL_CONTROL, input.grant_full_control());
    map_insert(header, HEADER_GRANT_READ, input.grant_read());
    map_insert(header, HEADER_GRANT_READ_ACP, input.grant_read_acp());
    map_insert(header, HEADER_GRANT_WRITE, input.grant_write());
    map_insert(header, HEADER_GRANT_WRITE_ACP, input.grant_write_acp());
}

pub(crate) fn set_copy_source_header<I: CopySourceHeader>(header: &mut HashMap<&str, String>, input: &I) -> Result<(), TosError> {
    if input.src_key() == "" {
        return TosError::client_error_result("invalid source object name");
    }

    let bucket = input.src_bucket().trim();
    if bucket == "" {
        return TosError::client_error_result("invalid source bucket name");
    }
    let mut copy_source = String::with_capacity(bucket.len() + input.src_key().len() * 2 + input.src_version_id().len() + 16);
    copy_source = copy_source + bucket + "/" + url_encode_with_safe(input.src_key(), "/").as_str();
    if input.src_version_id() != ""
    {
        copy_source = copy_source + "?versionId=" + input.src_version_id();
    }

    map_insert_string(header, HEADER_COPY_SOURCE, copy_source);
    Ok(())
}

pub(crate) fn set_copy_source_if_condition_header<I: CopySourceIfConditionHeader>(header: &mut HashMap<&str, String>, input: &I) {
    map_insert(header, HEADER_COPY_SOURCE_IF_MATCH, input.copy_source_if_match());

    if let Some(copy_source_if_modified_since) = input.copy_source_if_modified_since() {
        header.insert(HEADER_COPY_SOURCE_IF_MODIFIED_SINCE, copy_source_if_modified_since.format(RFC1123_DATE_FORMAT).to_string());
    }

    map_insert(header, HEADER_COPY_SOURCE_IF_NONE_MATCH, input.copy_source_if_none_match());

    if let Some(copy_source_if_unmodified_since) = input.copy_source_if_unmodified_since() {
        header.insert(HEADER_COPY_SOURCE_IF_UNMODIFIED_SINCE, copy_source_if_unmodified_since.format(RFC1123_DATE_FORMAT).to_string());
    }
}

pub(crate) fn set_if_condition_header<I: IfConditionHeader>(header: &mut HashMap<&str, String>, input: &I) {
    map_insert(header, HEADER_IF_MATCH, input.if_match());

    if let Some(if_modified_since) = input.if_modified_since() {
        header.insert(HEADER_IF_MODIFIED_SINCE, if_modified_since.format(RFC1123_DATE_FORMAT).to_string());
    }

    map_insert(header, HEADER_IF_NONE_MATCH, input.if_none_match());

    if let Some(if_unmodified_since) = input.if_unmodified_since() {
        header.insert(HEADER_IF_UNMODIFIED_SINCE, if_unmodified_since.format(RFC1123_DATE_FORMAT).to_string());
    }
}

pub(crate) fn set_http_basic_header<I: HttpBasicHeader>(header: &mut HashMap<&str, String>, disable_encoding_meta: bool, input: &I) {
    if input.content_length() >= 0 {
        header.insert(HEADER_CONTENT_LENGTH, input.content_length().to_string());
    }
    map_insert(header, HEADER_CACHE_CONTROL, input.cache_control());
    map_insert(header, HEADER_CONTENT_TYPE, input.content_type());
    map_insert(header, HEADER_CONTENT_LANGUAGE, input.content_language());
    map_insert(header, HEADER_CONTENT_ENCODING, input.content_encoding());
    if input.content_disposition() != "" {
        if disable_encoding_meta {
            header.insert(HEADER_CONTENT_DISPOSITION, input.content_disposition().to_string());
        } else {
            header.insert(HEADER_CONTENT_DISPOSITION, url_encode_chinese(input.content_disposition()));
        }
    }
    if let Some(expires) = input.expires() {
        header.insert(HEADER_EXPIRES, expires.format(RFC1123_DATE_FORMAT).to_string());
    }
}

pub(crate) fn set_http_basic_header_for_fetch<I: HttpBasicHeader>(header: &mut HashMap<&str, String>, disable_encoding_meta: bool, input: &I) {
    map_insert(header, HEADER_FETCH_CACHE_CONTROL, input.cache_control());
    map_insert(header, HEADER_FETCH_CONTENT_TYPE, input.content_type());
    map_insert(header, HEADER_FETCH_CONTENT_LANGUAGE, input.content_language());
    map_insert(header, HEADER_FETCH_CONTENT_ENCODING, input.content_encoding());
    if input.content_disposition() != "" {
        if disable_encoding_meta {
            header.insert(HEADER_FETCH_CONTENT_DISPOSITION, input.content_disposition().to_string());
        } else {
            header.insert(HEADER_FETCH_CONTENT_DISPOSITION, url_encode_chinese(input.content_disposition()));
        }
    }
    if let Some(expires) = input.expires() {
        header.insert(HEADER_FETCH_EXPIRES, expires.format(RFC1123_DATE_FORMAT).to_string());
    }
}

pub(crate) fn set_sse_header<I: SseHeader>(header: &mut HashMap<&str, String>, input: &I) -> Result<(), TosError> {
    if input.server_side_encryption() != "" {
        if input.server_side_encryption() != ALGORITHM_AES256 && input.server_side_encryption() != ALGORITHM_KMS {
            return TosError::client_error_result("invalid encryption-decryption algorithm");
        }

        header.insert(HEADER_SERVER_SIDE_ENCRYPTION, input.server_side_encryption().to_string());
        map_insert(header, HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID, input.server_side_encryption_key_id());
    }

    Ok(())
}

pub(crate) fn set_ssec_header<I: SsecHeader>(header: &mut HashMap<&str, String>, server_side_encryption: &str, input: &I) -> Result<(), TosError> {
    if input.ssec_algorithm() != "" {
        if input.ssec_algorithm() != ALGORITHM_AES256 {
            return TosError::client_error_result("invalid encryption-decryption algorithm");
        }

        if server_side_encryption != "" {
            return TosError::client_error_result("both ssec and server side encryption are set");
        }

        if input.ssec_key() == "" || input.ssec_key_md5() == "" {
            return TosError::client_error_result("empty ssec key or ssec key md5");
        }
        header.insert(HEADER_SSEC_ALGORITHM, input.ssec_algorithm().to_string());
        header.insert(HEADER_SSEC_KEY, input.ssec_key().to_string());
        header.insert(HEADER_SSEC_KEY_MD5, input.ssec_key_md5().to_string());
    }

    Ok(())
}

pub(crate) fn set_copy_source_ssec_header<I: CopySourceSSecHeader>(header: &mut HashMap<&str, String>, input: &I) -> Result<(), TosError> {
    if input.copy_source_ssec_algorithm() != "" {
        if input.copy_source_ssec_algorithm() != ALGORITHM_AES256 {
            return TosError::client_error_result("invalid copy source encryption-decryption algorithm");
        }

        if input.copy_source_ssec_key() == "" || input.copy_source_ssec_key_md5() == "" {
            return TosError::client_error_result("empty copy source ssec key or ssec key md5");
        }
        header.insert(HEADER_COPY_SOURCE_SSEC_ALGORITHM, input.copy_source_ssec_algorithm().to_string());
        header.insert(HEADER_COPY_SOURCE_SSEC_KEY, input.copy_source_ssec_key().to_string());
        header.insert(HEADER_COPY_SOURCE_SSEC_KEY_MD5, input.copy_source_ssec_key_md5().to_string());
    }

    Ok(())
}

pub(crate) fn trans_meta(input: &HashMap<String, String>, disable_encoding_meta: bool) -> Option<HashMap<String, String>> {
    if input.len() == 0 {
        return None;
    }

    let mut meta = HashMap::<String, String>::with_capacity(input.len());
    let mut temp;
    for (key, value) in input.iter() {
        if key == "" {
            continue;
        }
        if disable_encoding_meta {
            if key.starts_with(HEADER_PREFIX_META) {
                temp = key.to_string();
            } else {
                temp = String::with_capacity(key.len() * 2 + HEADER_PREFIX_META.len());
                temp.push_str(HEADER_PREFIX_META);
                temp.push_str(key.as_str());
            }
        } else {
            if key.starts_with(HEADER_PREFIX_META) {
                temp = url_encode(key);
            } else {
                temp = String::with_capacity(key.len() * 2 + HEADER_PREFIX_META.len());
                temp.push_str(HEADER_PREFIX_META);
                temp.push_str(url_encode(key).as_str());
            }
        }
        meta.insert(temp, url_encode(value));
    }
    Some(meta)
}

pub(crate) fn set_misc_header<I: MiscHeader>(header: &mut HashMap<&str, String>, input: &I) {
    if let Some(storage_class) = input.storage_class() {
        header.insert(HEADER_STORAGE_CLASS, storage_class.as_str().to_string());
    }

    map_insert(header, HEADER_WEBSITE_REDIRECT_LOCATION, input.website_redirect_location());
}
pub(crate) fn set_misc_header_for_fetch<I: MiscHeader>(header: &mut HashMap<&str, String>, input: &I) {
    if let Some(storage_class) = input.storage_class() {
        header.insert(HEADER_STORAGE_CLASS, storage_class.as_str().to_string());
    }

    map_insert(header, HEADER_FETCH_WEBSITE_REDIRECT_LOCATION, input.website_redirect_location());
}
pub(crate) fn set_if_match_header<I: IfMatch>(header: &mut HashMap<&str, String>, input: &I) {
    map_insert(header, HEADER_IF_MATCH, input.if_match());
    map_insert(header, HEADER_IF_NONE_MATCH, input.if_none_match());
}
pub(crate) fn set_object_lock_header<I: ObjectLock>(header: &mut HashMap<&str, String>, input: &I) {
    if let Some(object_lock_mode) = input.object_lock_mode() {
        header.insert(HEADER_OBJECT_LOCK_MODE, object_lock_mode.as_str().to_string());
    }
    if let Some(object_lock_retain_util_date) = input.object_lock_retain_util_date() {
        header.insert(HEADER_OBJECT_LOCK_RETAIN_UNTIL_DATE, object_lock_retain_util_date.format(ISO8601_DATE_FORMAT).to_string());
    }
}

pub(crate) fn set_rewrite_response_query<I: RewriteResponseQuery>(query: &mut HashMap<&str, String>, input: &I) {
    map_insert(query, QUERY_RESPONSE_CACHE_CONTROL, input.response_cache_control());
    if input.response_content_disposition() != "" {
        query.insert(QUERY_RESPONSE_CONTENT_DISPOSITION, url_encode_chinese(input.response_content_disposition()));
    }
    map_insert(query, QUERY_RESPONSE_CONTENT_ENCODING, input.response_content_encoding());
    map_insert(query, QUERY_RESPONSE_CONTENT_LANGUAGE, input.response_content_language());
    map_insert(query, QUERY_RESPONSE_CONTENT_TYPE, input.response_content_type());
    if let Some(response_expires) = input.response_expires() {
        query.insert(QUERY_RESPONSE_EXPIRES, response_expires.format(RFC1123_DATE_FORMAT).to_string());
    }
}

pub(crate) fn set_data_process_query<I: DataProcessQuery>(query: &mut HashMap<&str, String>, input: &I) {
    map_insert(query, QUERY_PROCESS, input.process());
    map_insert(query, QUERY_SAVE_BUCKET, input.save_bucket());
    map_insert(query, QUERY_SAVE_OBJECT, input.save_object());
    if input.doc_page() > 0 {
        query.insert(QUERY_DOC_PAGE, input.doc_page().to_string());
    }
    if let Some(x) = input.src_type() {
        query.insert(QUERY_DOC_SRC_TYPE, x.as_str().to_string());
    }
    if let Some(x) = input.dst_type() {
        query.insert(QUERY_DOC_DST_TYPE, x.as_str().to_string());
    }
}

pub(crate) fn set_callback_header<I: CallbackHeader>(header: &mut HashMap<&str, String>, input: &I) {
    map_insert(header, HEADER_CALLBACK, input.callback());
    map_insert(header, HEADER_CALLBACK_VAR, input.callback_var());
}

pub(crate) fn set_list_common_query<I: ListCommonQuery>(query: &mut HashMap<&str, String>, input: &I) {
    map_insert(query, QUERY_PREFIX, input.prefix());
    map_insert(query, QUERY_DELIMITER, input.delimiter());
    map_insert(query, QUERY_ENCODING_TYPE, input.encoding_type());
}

pub(crate) fn set_multipart_upload_query<I: MultipartUploadQuery>(query: &mut HashMap<&str, String>, input: &I) -> Result<(), TosError> {
    set_upload_id(query, input.upload_id())?;
    if input.part_number() < 0 {
        return TosError::client_error_result("invalid part number");
    }
    query.insert(QUERY_PART_NUMBER, input.part_number().to_string());
    Ok(())
}

pub(crate) fn set_upload_id(query: &mut HashMap<&str, String>, upload_id: &str) -> Result<(), TosError> {
    if upload_id == "" {
        return TosError::client_error_result("empty upload id");
    }
    query.insert(QUERY_UPLOAD_ID, upload_id.to_string());
    Ok(())
}

pub(crate) trait InputDescriptor: GenericInputTrait {
    fn operation(&self) -> &str;

    fn bucket(&self) -> Result<&str, TosError>;

    fn key(&self) -> Result<&str, TosError> {
        Err(TosError::client_error("invoke InputDescriptor.key"))
    }
    fn is_control_operation(&self) -> bool {
        false
    }
}

pub(crate) trait InputTranslator<B>: InputDescriptor {
    fn trans(&self, _: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        Err(TosError::client_error("invoke InputTranslator.trans"))
    }

    fn trans_mut(&mut self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        self.trans(config_holder)
    }

    fn trans_bucket(&self) -> Result<HttpRequest<B>, TosError> {
        let mut request = HttpRequest::default();
        request.operation = self.operation();
        request.bucket = self.bucket()?.trim();
        Ok(request)
    }

    fn trans_key(&self) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.trans_bucket()?;
        request.key = self.key()?;
        Ok(request)
    }
}

pub(crate) struct AdditionalContext<'a> {
    pub(crate) request_url: Option<String>,
    pub(crate) request_size: i64,
    pub(crate) request_host: &'a str,
    pub(crate) request_date: Option<DateTime<Utc>>,
    pub(crate) request_header: &'a Option<HashMap<String, String>>,
    pub(crate) request_query: &'a Option<HashMap<String, String>>,
    pub(crate) is_control_operation: bool,
}

impl<'a> AdditionalContext<'a> {
    pub(crate) fn new() -> Self {
        Self {
            request_url: None,
            request_size: -1,
            request_host: "",
            request_date: None,
            request_header: &None,
            request_query: &None,
            is_control_operation: false,
        }
    }
}

pub(crate) struct MockAsyncInputTranslator {}

impl GenericInputTrait for MockAsyncInputTranslator {
    fn request_date(&self) -> Option<DateTime<Utc>> {
        None
    }

    fn request_host(&self) -> &str {
        ""
    }

    fn request_header(&self) -> &Option<HashMap<String, String>> {
        &None
    }

    fn request_query(&self) -> &Option<HashMap<String, String>> {
        &None
    }
}

impl InputDescriptor for MockAsyncInputTranslator {
    fn operation(&self) -> &str {
        "MockAsyncInputTranslator"
    }

    fn bucket(&self) -> Result<&str, TosError> {
        Err(TosError::client_error("invoke MockAsyncInputTranslator.bucket"))
    }
}
impl<B> InputTranslator<B> for MockAsyncInputTranslator {}


pub(crate) trait AclHeader {
    fn acl(&self) -> &Option<ACLType>;
    fn grant_full_control(&self) -> &str;
    fn grant_read(&self) -> &str;
    fn grant_read_acp(&self) -> &str;
    fn grant_write(&self) -> &str;
    fn grant_write_acp(&self) -> &str;
}

pub(crate) trait HttpBasicHeader {
    fn content_length(&self) -> i64;
    fn cache_control(&self) -> &str;
    fn content_disposition(&self) -> &str;
    fn content_encoding(&self) -> &str;
    fn content_language(&self) -> &str;
    fn content_type(&self) -> &str;
    fn expires(&self) -> Option<DateTime<Utc>>;
}

pub(crate) trait MiscHeader {
    fn website_redirect_location(&self) -> &str;
    fn storage_class(&self) -> &Option<StorageClassType>;
}

pub(crate) trait SsecHeader {
    fn ssec_algorithm(&self) -> &str;
    fn ssec_key(&self) -> &str;
    fn ssec_key_md5(&self) -> &str;
}

pub(crate) trait SseHeader {
    fn server_side_encryption(&self) -> &str;
    fn server_side_encryption_key_id(&self) -> &str;
}

pub(crate) trait CopySourceHeader {
    fn src_bucket(&self) -> &str;
    fn src_key(&self) -> &str;
    fn src_version_id(&self) -> &str;
}

pub(crate) trait CopySourceSSecHeader {
    fn copy_source_ssec_algorithm(&self) -> &str;
    fn copy_source_ssec_key(&self) -> &str;
    fn copy_source_ssec_key_md5(&self) -> &str;
}

pub(crate) trait CopySourceIfConditionHeader {
    fn copy_source_if_match(&self) -> &str;
    fn copy_source_if_modified_since(&self) -> Option<DateTime<Utc>>;
    fn copy_source_if_none_match(&self) -> &str;
    fn copy_source_if_unmodified_since(&self) -> Option<DateTime<Utc>>;
}

pub(crate) trait IfConditionHeader {
    fn if_match(&self) -> &str;
    fn if_modified_since(&self) -> Option<DateTime<Utc>>;
    fn if_none_match(&self) -> &str;
    fn if_unmodified_since(&self) -> Option<DateTime<Utc>>;
}

pub(crate) trait CallbackHeader {
    fn callback(&self) -> &str;
    fn callback_var(&self) -> &str;
}

pub(crate) trait ListCommonQuery {
    fn prefix(&self) -> &str;
    fn delimiter(&self) -> &str;
    fn encoding_type(&self) -> &str;
}

pub(crate) trait MultipartUploadQuery {
    fn upload_id(&self) -> &str;
    fn part_number(&self) -> isize;
}

pub(crate) trait RewriteResponseQuery {
    fn response_cache_control(&self) -> &str;
    fn response_content_disposition(&self) -> &str;
    fn response_content_encoding(&self) -> &str;
    fn response_content_language(&self) -> &str;
    fn response_content_type(&self) -> &str;
    fn response_expires(&self) -> Option<DateTime<Utc>>;
}

pub(crate) trait DataProcessQuery {
    fn process(&self) -> &str;
    fn doc_page(&self) -> isize;
    fn src_type(&self) -> &Option<DocPreviewSrcType>;
    fn dst_type(&self) -> &Option<DocPreviewDstType>;
    fn save_bucket(&self) -> &str;
    fn save_object(&self) -> &str;
}

pub(crate) trait ObjectLock {
    fn object_lock_mode(&self) -> &Option<ObjectLockModeType>;
    fn object_lock_retain_util_date(&self) -> Option<DateTime<Utc>>;
}

pub(crate) trait IfMatch {
    fn if_match(&self) -> &str;
    fn if_none_match(&self) -> &str;
}

pub(crate) trait OutputParser: Sized {
    fn parse_by_ref<B>(request: &HttpRequest<B>, response: &mut HttpResponse, request_info: RequestInfo, meta: Meta) -> Result<Self, TosError>;
    fn parse<B>(request: HttpRequest<B>, response: HttpResponse, request_info: RequestInfo, meta: Meta) -> Result<Self, TosError> {
        let mut response = response;
        Self::parse_by_ref(&request, &mut response, request_info, meta)
    }
}

pub(crate) fn trans_header_value(value: &HeaderValue) -> String {
    match value.to_str() {
        Ok(v) => v.to_string(),
        Err(_) => "".to_string(),
    }
}

pub(crate) fn get_map_value(map: &HashMap<&str, String>, key: &str) -> String {
    get_map_value_str(map, key).to_string()
}

pub(crate) fn get_map_value_str<'a>(map: &'a HashMap<&str, String>, key: &str) -> &'a str {
    match map.get(key) {
        Some(value) => {
            value
        }
        None => "",
    }
}

pub(crate) fn get_map_value_from_str<F: FromStr>(map: &HashMap<&str, String>, key: &str, def: F) -> Result<F, TosError> {
    match map.get(key) {
        Some(value) => {
            match value.parse::<F>() {
                Ok(x) => Ok(x),
                Err(_) => Err(TosError::client_error(format!("parse value error with input {}", value)))
            }
        }
        None => Ok(def),
    }
}

pub(crate) fn get_header_value(header: &HeaderMap, key: &str) -> String {
    get_header_value_str(header, key).to_string()
}

pub(crate) fn get_header_value_ref<'a>(header: &'a HeaderMap, key: &str) -> &'a str {
    get_header_value_str(header, key)
}


pub(crate) fn get_header_value_url_decoded(header: &HeaderMap, key: &str) -> String {
    match urlencoding::decode(get_header_value_str(header, key)) {
        Ok(x) => x.to_string(),
        Err(_) => "".to_string(),
    }
}

pub(crate) fn get_header_value_str<'a>(header: &'a HeaderMap, key: &str) -> &'a str {
    match header.get(key) {
        Some(value) => {
            match value.to_str() {
                Ok(x) => {
                    x
                }
                Err(_) => {
                    ""
                }
            }
        }
        None => "",
    }
}

pub(crate) fn get_header_value_from_str<F: FromStr>(header: &HeaderMap, key: &str, def: F) -> Result<F, TosError> {
    match header.get(key) {
        Some(value) => {
            if value.is_empty() {
                return Ok(def);
            }

            match value.to_str() {
                Ok(v) => {
                    match v.parse::<F>() {
                        Ok(x) => Ok(x),
                        Err(_) => Err(TosError::client_error(format!("parse value error with input {}", v)))
                    }
                }
                Err(_) => Ok(def),
            }
        }
        None => Ok(def),
    }
}

pub(crate) fn parse_response_string_by_buf(buf: Vec<u8>) -> Result<String, TosError> {
    match String::from_utf8(buf) {
        Ok(x) => Ok(x),
        Err(e) => Err(TosError::client_error_with_cause("trans string error", GenericError::DefaultError(e.to_string()))),
    }
}

pub(crate) fn parse_json_by_buf<T>(buf: &[u8]) -> Result<T, TosError>
where
    T: DeserializeOwned,
{
    match serde_json::from_slice::<T>(buf) {
        Err(e) => Err(TosError::client_error_with_cause("parse json error", GenericError::JsonError(e.to_string()))),
        Ok(item) => Ok(item),
    }
}

pub(crate) fn read_response(response: &mut HttpResponse) -> Result<Vec<u8>, TosError> {
    let mut buf;
    match response.content_length() {
        None => {
            buf = Vec::with_capacity(DEFAULT_READ_BUFFER_SIZE);
            read_at_most(response, &mut buf, MAX_READ_BUFFER_SIZE_FOR_JSON)?;
            Ok(buf)
        }
        Some(x) => {
            buf = Vec::with_capacity(x as usize);
            // wrap with reader length check
            let mut readable = InternalReader::sized(response, x as usize);
            read_at_most(&mut readable, &mut buf, MAX_READ_BUFFER_SIZE_FOR_JSON)?;
            Ok(buf)
        }
    }
}

pub(crate) fn read_response_string(response: &mut HttpResponse) -> Result<String, TosError> {
    let buf = read_response(response)?;
    match String::from_utf8(buf) {
        Ok(x) => Ok(x),
        Err(e) => Err(TosError::client_error_with_cause("trans string error", GenericError::DefaultError(e.to_string()))),
    }
}

pub(crate) fn parse_json<T>(response: &mut HttpResponse) -> Result<T, TosError>
where
    T: DeserializeOwned,
{
    let buf = read_response(response)?;
    match serde_json::from_slice::<T>(buf.as_slice()) {
        Err(e) => Err(TosError::client_error_with_cause("parse json error", GenericError::JsonError(e.to_string()))),
        Ok(item) => Ok(item),
    }
}

pub(crate) fn sleep_for_retry(retry_count: isize, retry_after: isize) {
    let mut delay = BASE_DELAY_MS * 2u64.pow(retry_count as u32);
    if delay > MAX_DELAY_MS {
        delay = MAX_DELAY_MS;
    }
    let retry_after = retry_after as u64 * 1000;
    if retry_after > delay {
        delay = retry_after;
    }
    thread::sleep(time::Duration::from_millis(delay))
}

pub(crate) fn check_bucket_and_key<T>(input: &T, is_custom_domain: bool) -> Result<&str, TosError>
where
    T: InputDescriptor,
{
    if let Ok(key) = input.key() {
        if key == "" {
            return Err(TosError::client_error("invalid object name"));
        }
        let bucket_or_account_id = input.bucket()?;
        if input.is_control_operation() {
            check_account_id(bucket_or_account_id.trim())?;
        } else if !is_custom_domain {
            check_bucket(bucket_or_account_id.trim())?;
        }
    } else if let Ok(bucket_or_account_id) = input.bucket() {
        if input.is_control_operation() {
            check_account_id(bucket_or_account_id.trim())?;
        } else if !is_custom_domain {
            check_bucket(bucket_or_account_id.trim())?;
        }
    }
    Ok(input.operation())
}

pub(crate) fn check_bucket(bucket: &str) -> Result<(), TosError> {
    if bucket.len() < 3 || bucket.len() > 63 {
        return TosError::client_error_result("invalid bucket name, the length must be [3, 63]");
    }

    if bucket.starts_with("-") || bucket.ends_with("-") {
        return TosError::client_error_result("invalid bucket name, the bucket name can be neither starting with '-' nor ending with '-'");
    }

    if !BUCKET_REGEX.is_match(bucket) {
        return TosError::client_error_result("invalid bucket name, the character set is illegal");
    }

    Ok(())
}

pub(crate) fn check_account_id(account_id: &str) -> Result<(), TosError> {
    if !ACCOUNT_REGEX.is_match(account_id) {
        return TosError::client_error_result("invalid account id, the character set is illegal");
    }
    Ok(())
}

pub(crate) fn check_need_retry(e: &TosError, retry_count: isize, max_retry_count: isize, operation: &str) -> (isize, bool) {
    if retry_count >= max_retry_count {
        return (0, false);
    }

    match e {
        TosClientError { cause, .. } => {
            if cause.is_none() {
                return (0, false);
            }
            match cause.as_ref().unwrap() {
                GenericError::HttpRequestError(_) => (0, timeout_retryable(operation)),
                GenericError::IoError(_) => (0, timeout_retryable(operation)),
                _ => (0, false),
            }
        }
        TosServerError { status_code, header, .. } => {
            if *status_code == 408 {
                return (0, timeout_retryable(operation));
            }

            if *status_code == 503 || *status_code == 429 {
                if let Some(x) = header.get(HEADER_RETRY_AFTER) {
                    if let Ok(y) = x.parse::<isize>() {
                        return (y, server_error_retryable(operation));
                    }
                } else if let Some(x) = header.get(HEADER_RETRY_AFTER_LOWER) {
                    if let Ok(y) = x.parse::<isize>() {
                        return (y, server_error_retryable(operation));
                    }
                }
                return (0, server_error_retryable(operation));
            }

            (0, *status_code >= 500 && server_error_retryable(operation))
        }
    }
}

fn timeout_retryable(operation: &str) -> bool {
    !NO_IDEMPOTENT_OPERATIONS.contains_key(operation) && !STREAM_UPLOAD_OPERATIONS.contains_key(operation)
}

fn server_error_retryable(operation: &str) -> bool {
    !STREAM_UPLOAD_OPERATIONS.contains_key(operation)
}

pub(crate) fn truncate_date_to_midnight(date: DateTime<Utc>) -> String {
    let mut date = date.format(ISO8601_DATE_FORMAT_TRUNCATED).to_string();
    date.push_str("T00:00:00Z");
    date
}

pub(crate) fn auto_recognize_content_type<B>(request: &mut HttpRequest<B>, auto_recognize_content_type: bool) {
    if !auto_recognize_content_type || request.key == "" {
        return;
    }

    if request.header.contains_key(HEADER_CONTENT_TYPE) {
        return;
    }

    if !AUTO_RECOGNIZE_CONTENT_TYPE_OPERATIONS.contains_key(request.operation) {
        return;
    }
    if let Some(idx) = request.key.rfind('.') {
        if let Some(ct) = MIME_TYPES.get(request.key[idx + 1..].to_lowercase().as_str()) {
            request.header.insert(HEADER_CONTENT_TYPE, (*ct).to_string());
        }
    }
}

// elapsed_ms greater than 500ms
pub(crate) fn exceed_high_latency_log_threshold(high_latency_log_threshold: isize, elapsed_ms: u128, request_size: i64, operation: &str) -> bool {
    high_latency_log_threshold > 0 && elapsed_ms > 500 && request_size > 0
        && (request_size as isize / 1024 < high_latency_log_threshold)
        && (operation == GET_OBJECT_TO_FILE_OPERATION || ALL_UPLOAD_OPERATIONS.contains_key(operation))
}

pub(crate) fn get_request_url<B>(request: &HttpRequest<B>, config_holder: &ConfigHolder, is_control_operation: bool) -> String {
    let bucket = request.bucket;
    let mut request_url;
    if is_control_operation {
        request_url = config_holder.get_endpoint_with_domain(bucket, request.key, &config_holder.schema_control, &config_holder.domain_control, true, config_holder.is_custom_domain);
    } else {
        request_url = config_holder.get_endpoint(bucket, request.key);
    }

    if let Some(query) = request.query.as_ref() {
        if query.len() > 0 {
            request_url.push('?');
            for (idx, kv) in query.iter().enumerate() {
                request_url.push_str(*kv.0);
                request_url.push('=');
                request_url.push_str(kv.1);
                if idx != query.len() - 1 {
                    request_url.push('&');
                }
            }
        }
    }
    request_url
}

pub(crate) fn load_file_to_buf(file_path: &str) -> Result<Vec<u8>, TosError> {
    match File::open(file_path) {
        Err(ex) => Err(TosError::client_error_with_cause(format!("open file {} error", file_path), GenericError::IoError(ex.to_string()))),
        Ok(mut fd) => {
            let mut buf = Vec::new();
            match fd.read_to_end(&mut buf) {
                Err(ex) => Err(TosError::client_error_with_cause(format!("read file {} error", file_path), GenericError::IoError(ex.to_string()))),
                Ok(_) => Ok(buf)
            }
        }
    }
}

pub(crate) fn build_certificate(ca_crt: &str) -> Result<Certificate, TosError> {
    let buf = load_file_to_buf(ca_crt)?;
    match Certificate::from_der(&buf) {
        Err(ex) => Err(TosError::client_error_with_cause("build certificate error", GenericError::IoError(ex.to_string()))),
        Ok(cert) => Ok(cert)
    }
}

pub(crate) fn build_identity(client_crt: &str, client_key: &str) -> Result<Identity, TosError> {
    let crt = load_file_to_buf(client_crt)?;
    #[cfg(feature = "use-native-tls")]
    {
        let key = crate::internal::load_file_to_buf(client_key)?;
        return match Identity::from_pkcs8_pem(&crt, &key) {
            Err(ex) => Err(TosError::client_error_with_cause("build identity error", GenericError::IoError(ex.to_string()))),
            Ok(identity) => Ok(identity)
        };
    }

    #[cfg(feature = "use-rustls")]
    {
        return match Identity::from_pem(&crt) {
            Err(ex) => Err(TosError::client_error_with_cause("build identity error", GenericError::IoError(ex.to_string()))),
            Ok(identity) => Ok(identity)
        };
    }

    Err(TosError::client_error("must enable use-native-tls or use-rustls feature"))
}

const GF2_DIM: usize = 64;
const ECMA: u64 = 0xC96C5795D7870F42;
fn gf2matrix_times(mat: &[u64; GF2_DIM], mut vec: u64) -> u64 {
    let mut sum = 0u64;
    let mut i = 0;
    while vec != 0 {
        if vec & 1 != 0 {
            sum ^= mat[i];
        }
        vec >>= 1;
        i += 1;
    }
    sum
}

fn gf2matrix_square(square: &mut [u64; GF2_DIM], mat: &[u64; 64]) {
    let mut n = 0;
    while n < GF2_DIM {
        square[n] = gf2matrix_times(mat, mat[n]);
        n += 1;
    }
}

pub(crate) fn combine_crc64(mut crc1: u64, crc2: u64, mut len2: usize) -> u64 {
    // Degenerate case
    if len2 == 0 {
        return crc1;
    }

    let mut even = [0u64; GF2_DIM];
    let mut odd = [0u64; GF2_DIM];

    // Put operator for one zero bit in odd
    odd[0] = ECMA;
    let mut row = 1u64;
    for n in 1..GF2_DIM {
        odd[n] = row;
        row <<= 1;
    }

    // Put operator for two zero bits in even
    gf2matrix_square(&mut even, &odd);

    // Put operator for four zero bits in odd
    gf2matrix_square(&mut odd, &even);

    // Apply len2 zeros to crc1, first square will put the operator for one zero byte, eight zero bits, in even
    loop {
        // Apply zeros operator for this bit of len2
        gf2matrix_square(&mut even, &odd);

        if len2 & 1 != 0 {
            crc1 = gf2matrix_times(&even, crc1)
        }

        len2 >>= 1;

        // If no more bits set, then done
        if len2 == 0 {
            break;
        }

        // Another iteration of the loop with odd and even swapped
        gf2matrix_square(&mut odd, &even);
        if len2 & 1 != 0 {
            crc1 = gf2matrix_times(&odd, crc1);
        }
        len2 >>= 1;

        // If no more bits set, then done
        if len2 == 0 {
            break;
        }
    }

    // Return combined CRC
    crc1 ^= crc2;
    crc1
}
