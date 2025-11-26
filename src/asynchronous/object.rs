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
use crate::constant::{HEADER_COPY_SOURCE_VERSION_ID, HEADER_LAST_MODIFIED, HEADER_NEXT_APPEND_OFFSET, HEADER_NEXT_MODIFY_OFFSET, HEADER_RANGE, HEADER_SYMLINK_BUCKET, HEADER_SYMLINK_TARGET, QUERY_PROCESS};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::asynchronous::common::DataTransferListener;
use crate::asynchronous::http::HttpResponse;
use crate::asynchronous::internal::{parse_json, read_response, OutputParser};
use crate::asynchronous::reader::StreamAdapter;
use crate::common::{DataTransferStatus, Meta, RequestInfo, TempCopyResult, TempFetchResult};
use crate::constant::{HEADER_CALLBACK, HEADER_CONTENT_RANGE, HEADER_DELETE_MARKER, HEADER_ETAG, HEADER_HASH_CRC64ECMA, HEADER_PREFIX_META, HEADER_SERVER_SIDE_ENCRYPTION, HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID, HEADER_SSEC_ALGORITHM, HEADER_SSEC_KEY_MD5, HEADER_VERSION_ID, TRUE};
use crate::error::{ErrorResponse, GenericError, TosError};
use crate::http::HttpRequest;
use crate::internal::{get_header_value, get_header_value_from_str, get_header_value_str, get_map_value_str, parse_date_time_iso8601, parse_date_time_rfc1123, parse_json_by_buf, parse_response_string_by_buf};
use crate::object::{AppendObjectFromBufferInput, AppendObjectInput, AppendObjectOutput, CopyObjectInput, CopyObjectOutput, DeleteMultiObjectsInput, DeleteMultiObjectsOutput, DeleteObjectInput, DeleteObjectOutput, DeleteObjectTaggingInput, DeleteObjectTaggingOutput, DoesObjectExistInput, FetchObjectInput, FetchObjectOutput, GetFetchTaskInput, GetFetchTaskOutput, GetFileStatusInput, GetFileStatusOutput, GetObjectACLInput, GetObjectACLOutput, GetObjectInput, GetObjectOutput, GetObjectTaggingInput, GetObjectTaggingOutput, GetSymlinkInput, GetSymlinkOutput, HeadObjectInput, HeadObjectOutput, ListObjectVersionsInput, ListObjectVersionsOutput, ListObjectsType2Input, ListObjectsType2Output, ModifyObjectFromBufferInput, ModifyObjectInput, ModifyObjectOutput, PutFetchTaskInput, PutFetchTaskOutput, PutObjectACLInput, PutObjectACLOutput, PutObjectFromBufferInput, PutObjectInput, PutObjectOutput, PutObjectTaggingInput, PutObjectTaggingOutput, PutSymlinkInput, PutSymlinkOutput, RenameObjectInput, RenameObjectOutput, RestoreObjectInput, RestoreObjectOutput, SetObjectMetaInput, SetObjectMetaOutput, SetObjectTimeInput, SetObjectTimeOutput};
use crate::reader::MultifunctionalReader;
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use futures_util::StreamExt;

#[async_trait]
pub trait ObjectAPI {
    async fn copy_object(&self, input: &CopyObjectInput) -> Result<CopyObjectOutput, TosError>;
    async fn delete_object(&self, input: &DeleteObjectInput) -> Result<DeleteObjectOutput, TosError>;
    async fn delete_multi_objects(&self, input: &DeleteMultiObjectsInput) -> Result<DeleteMultiObjectsOutput, TosError>;
    async fn put_object<B>(&self, input: &PutObjectInput<B>) -> Result<PutObjectOutput, TosError>
    where
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Sync + Unpin + 'static;
    async fn put_object_from_buffer(&self, input: &PutObjectFromBufferInput) -> Result<PutObjectOutput, TosError>;
    #[cfg(feature = "tokio-runtime")]
    async fn put_object_from_file(&self, input: &crate::object::PutObjectFromFileInput) -> Result<PutObjectOutput, TosError>;
    async fn append_object<B>(&self, input: &AppendObjectInput<B>) -> Result<AppendObjectOutput, TosError>
    where
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Sync + Unpin + 'static;
    async fn append_object_from_buffer(&self, input: &AppendObjectFromBufferInput) -> Result<AppendObjectOutput, TosError>;
    async fn get_object(&self, input: &GetObjectInput) -> Result<GetObjectOutput, TosError>;
    #[cfg(feature = "tokio-runtime")]
    async fn get_object_to_file(&self, input: &crate::object::GetObjectToFileInput) -> Result<crate::object::GetObjectToFileOutput, TosError>;
    async fn get_object_acl(&self, input: &GetObjectACLInput) -> Result<GetObjectACLOutput, TosError>;
    async fn head_object(&self, input: &HeadObjectInput) -> Result<HeadObjectOutput, TosError>;
    async fn list_objects_type2(&self, input: &ListObjectsType2Input) -> Result<ListObjectsType2Output, TosError>;
    async fn list_object_versions(&self, input: &ListObjectVersionsInput) -> Result<ListObjectVersionsOutput, TosError>;
    async fn put_object_acl(&self, input: &PutObjectACLInput) -> Result<PutObjectACLOutput, TosError>;
    async fn set_object_meta(&self, input: &SetObjectMetaInput) -> Result<SetObjectMetaOutput, TosError>;
    async fn fetch_object(&self, input: &FetchObjectInput) -> Result<FetchObjectOutput, TosError>;
    async fn put_fetch_task(&self, input: &PutFetchTaskInput) -> Result<PutFetchTaskOutput, TosError>;
    async fn get_fetch_task(&self, input: &GetFetchTaskInput) -> Result<GetFetchTaskOutput, TosError>;
    async fn put_object_tagging(&self, input: &PutObjectTaggingInput) -> Result<PutObjectTaggingOutput, TosError>;
    async fn get_object_tagging(&self, input: &GetObjectTaggingInput) -> Result<GetObjectTaggingOutput, TosError>;
    async fn delete_object_tagging(&self, input: &DeleteObjectTaggingInput) -> Result<DeleteObjectTaggingOutput, TosError>;
    async fn rename_object(&self, input: &RenameObjectInput) -> Result<RenameObjectOutput, TosError>;
    async fn restore_object(&self, input: &RestoreObjectInput) -> Result<RestoreObjectOutput, TosError>;
    async fn put_symlink(&self, input: &PutSymlinkInput) -> Result<PutSymlinkOutput, TosError>;
    async fn get_symlink(&self, input: &GetSymlinkInput) -> Result<GetSymlinkOutput, TosError>;
    async fn get_file_status(&self, input: &GetFileStatusInput) -> Result<GetFileStatusOutput, TosError>;
    async fn does_object_exist(&self, input: &DoesObjectExistInput) -> Result<bool, TosError>;
    async fn set_object_time(&self, input: &SetObjectTimeInput) -> Result<SetObjectTimeOutput, TosError>;
}

#[async_trait]
pub trait ObjectContent {
    type Content: ?Sized;

    fn content(&mut self) -> Option<&mut Self::Content>;

    async fn read_all(&mut self) -> Result<Vec<u8>, TosError>;
}

#[async_trait]
impl OutputParser for DeleteObjectOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        let delete_marker = get_header_value_str(response.headers(), HEADER_DELETE_MARKER) == TRUE;
        Ok(Self {
            request_info,
            delete_marker,
            version_id,
        })
    }
}

#[async_trait]
impl OutputParser for HeadObjectOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, meta: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Self::parse_by_header(response.headers(), request_info, meta)
    }
}

#[async_trait]
impl OutputParser for ListObjectsType2Output {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
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


#[async_trait]
impl OutputParser for GetObjectOutput {
    async fn parse<B>(request: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, meta: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let head_object_output = HeadObjectOutput::parse_by_header(response.headers(), request_info, meta)?;
        let content_range = get_header_value(response.headers(), HEADER_CONTENT_RANGE);
        let content = Box::new(StreamAdapter::new(response.bytes_stream())) as Box<dyn Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Unpin>;
        let mut target_crc64 = None;
        if request.enable_crc && !request.header.contains_key(HEADER_RANGE) &&
            (request.query.is_none() || !request.query.as_ref().unwrap().contains_key(QUERY_PROCESS)) {
            target_crc64 = Some(head_object_output.hash_crc64ecma);
        }
        let mut crc64 = None;
        if target_crc64.is_some() {
            crc64 = Some(Arc::new(AtomicU64::new(0)));
        }
        let mut reader = MultifunctionalReader::with_target_crc64(content, crc64, head_object_output.content_length,
                                                                  &request, target_crc64);

        if let Some(ref rc) = request.request_context {
            if let Some(ref rl) = rc.rate_limiter {
                reader.set_rate_limiter(rl.clone());
            }
            if let Some(ref adts) = rc.async_data_transfer_listener {
                reader.set_async_data_transfer_listener(adts.clone());
                reader.inner.operation = request.operation.to_string();
                reader.inner.bucket = request.bucket.to_string();
                reader.inner.key = request.key.to_string();
                reader.inner.retry_count = request.retry_count;
            }
        }

        Ok(Self {
            content_range,
            content: None,
            async_content: Some(reader),
            head_object_output,
        })
    }
}

impl Stream for GetObjectOutput {
    type Item = Result<Bytes, crate::error::CommonError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.async_content.as_mut() {
            None => Poll::Ready(None),
            Some(reader) => {
                reader.poll_next_unpin(cx)
            }
        }
    }
}
#[async_trait]
impl ObjectContent for GetObjectOutput {
    type Content = (dyn Stream<Item=Result<Bytes, crate::error::CommonError>> + Unpin);

    fn content(&mut self) -> Option<&mut Self::Content> {
        match self.async_content.as_mut() {
            None => None,
            Some(x) => Some(x as &mut (dyn Stream<Item=Result<Bytes, crate::error::CommonError>> + Unpin)),
        }
    }

    async fn read_all(&mut self) -> Result<Vec<u8>, TosError> {
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

        match self.async_content.as_mut() {
            None => Err(TosError::client_error("empty content")),
            Some(r) => {
                loop {
                    match r.next().await {
                        None => return Ok(buf),
                        Some(result) => {
                            match result {
                                Err(e) => {
                                    return Err(TosError::client_error_with_cause("read error", GenericError::IoError(e.to_string())));
                                }
                                Ok(x) => {
                                    buf.extend_from_slice(x.as_ref());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


#[async_trait]
impl OutputParser for PutObjectOutput {
    async fn parse<B>(request: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let hash_crc64ecma = get_header_value_from_str::<u64>(response.headers(), HEADER_HASH_CRC64ECMA, 0)?;
        if let Some(ref rc) = request.request_context {
            if let Some(calc_hash_crc64ecma) = rc.crc64 {
                if calc_hash_crc64ecma != hash_crc64ecma {
                    return Err(TosError::client_error(format!("expect crc64 {hash_crc64ecma}, actual crc64 {calc_hash_crc64ecma}")));
                }
            }
        }

        let mut result = Self::default();
        result.etag = get_header_value(response.headers(), HEADER_ETAG);
        result.version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        result.ssec_algorithm = get_header_value(response.headers(), HEADER_SSEC_ALGORITHM);
        result.ssec_key_md5 = get_header_value(response.headers(), HEADER_SSEC_KEY_MD5);
        result.hash_crc64ecma = hash_crc64ecma;
        result.server_side_encryption = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION);
        result.server_side_encryption_key_id = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID);
        if get_map_value_str(&request.header, HEADER_CALLBACK) != "" { // callback
            let buf = read_response(response).await?;
            if request_info.status_code == 203 {
                if let Ok(error_response) = parse_json_by_buf::<ErrorResponse>(buf.as_slice()) {
                    return Err(TosError::server_error_with_code(error_response.code, error_response.ec, error_response.key, error_response.message,
                                                                error_response.host_id, error_response.resource, request_info));
                }
            }
            result.callback_result = parse_response_string_by_buf(buf)?;
        }
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for CopyObjectOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = Self {
            request_info: RequestInfo::default(),
            etag: "".to_owned(),
            last_modified: None,
            copy_source_version_id: get_header_value(response.headers(), HEADER_COPY_SOURCE_VERSION_ID),
            version_id: get_header_value(response.headers(), HEADER_VERSION_ID),
            ssec_algorithm: get_header_value(response.headers(), HEADER_SSEC_ALGORITHM),
            ssec_key_md5: get_header_value(response.headers(), HEADER_SSEC_KEY_MD5),
            server_side_encryption: get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION),
            server_side_encryption_key_id: get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID),
        };

        let temp_result = parse_json::<TempCopyResult>(response).await?;
        if temp_result.etag == "" {
            return Err(TosError::server_error_with_code(temp_result.code, temp_result.ec, temp_result.key, temp_result.message,
                                                        temp_result.host_id, temp_result.resource, request_info));
        }

        result.etag = temp_result.etag;
        result.last_modified = parse_date_time_iso8601(&temp_result.last_modified)?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteMultiObjectsOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}
#[async_trait]
impl OutputParser for GetObjectACLOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        let mut result = parse_json::<Self>(response).await?;
        result.version_id = version_id;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for ListObjectVersionsOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
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

#[async_trait]
impl OutputParser for PutObjectACLOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for SetObjectMetaOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for AppendObjectOutput {
    async fn parse<B>(request: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
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

#[async_trait]
impl OutputParser for ModifyObjectOutput {
    async fn parse<B>(request: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let hash_crc64ecma = get_header_value_from_str::<u64>(response.headers(), HEADER_HASH_CRC64ECMA, 0)?;
        if let Some(ref rc) = request.request_context {
            if let Some(calc_hash_crc64ecma) = rc.crc64 {
                if calc_hash_crc64ecma != hash_crc64ecma {
                    return Err(TosError::client_error(format!("expect crc64 {hash_crc64ecma}, actual crc64 {calc_hash_crc64ecma}")));
                }
            }
        }
        let mut result = Self::default();
        result.next_modify_offset = get_header_value_from_str::<i64>(response.headers(), HEADER_NEXT_MODIFY_OFFSET, 0)?;
        result.hash_crc64ecma = hash_crc64ecma;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for PutObjectTaggingOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = Self::default();
        result.request_info = request_info;
        result.version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for GetObjectTaggingOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        result.version_id = version_id;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for DeleteObjectTaggingOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = Self::default();
        result.request_info = request_info;
        result.version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for FetchObjectOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = Self {
            request_info: RequestInfo::default(),
            etag: "".to_owned(),
            version_id: get_header_value(response.headers(), HEADER_VERSION_ID),
            ssec_algorithm: get_header_value(response.headers(), HEADER_SSEC_ALGORITHM),
            ssec_key_md5: get_header_value(response.headers(), HEADER_SSEC_KEY_MD5),
            server_side_encryption: get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION),
            server_side_encryption_key_id: get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID),
            source_content_type: "".to_string(),
            source_content_length: 0,
            md5: "".to_string(),
        };

        let temp_result = parse_json::<TempFetchResult>(response).await?;
        if temp_result.etag == "" {
            return Err(TosError::server_error_with_code(temp_result.code, temp_result.ec, temp_result.key, temp_result.message,
                                                        temp_result.host_id, temp_result.resource, request_info));
        }
        result.etag = temp_result.etag;
        result.source_content_type = temp_result.source_content_type;
        result.source_content_length = temp_result.source_content_length;
        result.md5 = temp_result.md5;
        result.request_info = request_info;
        Ok(result)
    }
}


#[async_trait]
impl OutputParser for RenameObjectOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for RestoreObjectOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for PutFetchTaskOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for GetFetchTaskOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        if let Some(fetch_task) = &mut result.task {
            if fetch_task.user_meta.len() > 0 {
                fetch_task.meta = HashMap::with_capacity(fetch_task.user_meta.len());
                for item in fetch_task.user_meta.iter() {
                    fetch_task.meta.insert(item.key.clone(), item.value.clone());
                }
            }
        }
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for PutSymlinkOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        Ok(Self {
            request_info,
            version_id,
        })
    }
}

#[async_trait]
impl OutputParser for GetSymlinkOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let symlink_target_key = get_header_value(response.headers(), HEADER_SYMLINK_TARGET);
        let symlink_target_bucket = get_header_value(response.headers(), HEADER_SYMLINK_BUCKET);
        let version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        let etag = get_header_value(response.headers(), HEADER_ETAG);
        let last_modified = parse_date_time_rfc1123(&get_header_value(response.headers(), HEADER_LAST_MODIFIED))?;
        Ok(Self {
            request_info,
            version_id,
            symlink_target_key,
            symlink_target_bucket,
            etag,
            last_modified,
        })
    }
}

#[async_trait]
impl OutputParser for GetFileStatusOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        if let Some(x) = result.last_modified_string.take() {
            result.last_modified = parse_date_time_iso8601(&x)?;
        }
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for SetObjectTimeOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self {
            request_info,
        })
    }
}

impl<B> DataTransferListener for PutObjectInput<B> {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.inner.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.inner.async_data_transfer_listener = Some(listener.into());
    }
}

impl DataTransferListener for PutObjectFromBufferInput {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.inner.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.inner.async_data_transfer_listener = Some(listener.into());
    }
}
impl<B> DataTransferListener for AppendObjectInput<B> {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.inner.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.inner.async_data_transfer_listener = Some(listener.into());
    }
}

impl DataTransferListener for AppendObjectFromBufferInput {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.inner.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.inner.async_data_transfer_listener = Some(listener.into());
    }
}

impl DataTransferListener for GetObjectInput {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.async_data_transfer_listener = Some(listener.into());
    }
}

impl<B> DataTransferListener for ModifyObjectInput<B> {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.async_data_transfer_listener = Some(listener.into());
    }
}


impl DataTransferListener for ModifyObjectFromBufferInput {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.async_data_transfer_listener = Some(listener.into());
    }
}