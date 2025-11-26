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
use crate::asynchronous::common::DataTransferListener;
use crate::asynchronous::http::HttpResponse;
use crate::asynchronous::internal::{parse_json, read_response, OutputParser};
use crate::common::{DataTransferStatus, Meta, RequestInfo, TempCopyResult};
use crate::constant::{HEADER_CALLBACK, HEADER_COPY_SOURCE_VERSION_ID, HEADER_ETAG, HEADER_HASH_CRC64ECMA, HEADER_LOCATION, HEADER_SERVER_SIDE_ENCRYPTION, HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID, HEADER_SSEC_ALGORITHM, HEADER_SSEC_KEY_MD5, HEADER_VERSION_ID, QUERY_PART_NUMBER};
use crate::error::{ErrorResponse, TosError};
use crate::http::HttpRequest;
use crate::internal::{get_header_value, get_header_value_from_str, get_map_value_from_str, get_map_value_str, parse_date_time_iso8601, parse_json_by_buf, parse_response_string_by_buf};
use crate::multipart::{AbortMultipartUploadInput, AbortMultipartUploadOutput, CompleteMultipartUploadInput, CompleteMultipartUploadOutput, CreateMultipartUploadInput, CreateMultipartUploadOutput, ListMultipartUploadsInput, ListMultipartUploadsOutput, ListPartsInput, ListPartsOutput, UploadPartCopyInput, UploadPartCopyOutput, UploadPartFromBufferInput, UploadPartInput, UploadPartOutput};
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;

#[async_trait]
pub trait MultipartAPI {
    async fn create_multipart_upload(&self, input: &CreateMultipartUploadInput) -> Result<CreateMultipartUploadOutput, TosError>;
    async fn upload_part<B>(&self, input: &UploadPartInput<B>) -> Result<UploadPartOutput, TosError>
    where
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Sync + Unpin + 'static;
    async fn upload_part_from_buffer(&self, input: &UploadPartFromBufferInput) -> Result<UploadPartOutput, TosError>;
    #[cfg(feature = "tokio-runtime")]
    async fn upload_part_from_file(&self, input: &crate::multipart::UploadPartFromFileInput) -> Result<UploadPartOutput, TosError>;
    async fn complete_multipart_upload(&self, input: &CompleteMultipartUploadInput) -> Result<CompleteMultipartUploadOutput, TosError>;
    async fn abort_multipart_upload(&self, input: &AbortMultipartUploadInput) -> Result<AbortMultipartUploadOutput, TosError>;
    async fn upload_part_copy(&self, input: &UploadPartCopyInput) -> Result<UploadPartCopyOutput, TosError>;
    async fn list_multipart_uploads(&self, input: &ListMultipartUploadsInput) -> Result<ListMultipartUploadsOutput, TosError>;
    async fn list_parts(&self, input: &ListPartsInput) -> Result<ListPartsOutput, TosError>;
}

#[async_trait]
impl OutputParser for CreateMultipartUploadOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let ssec_algorithm = get_header_value(response.headers(), HEADER_SSEC_ALGORITHM);
        let ssec_key_md5 = get_header_value(response.headers(), HEADER_SSEC_KEY_MD5);
        let server_side_encryption = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION);
        let server_side_encryption_key_id = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID);
        let request_info = request_info;
        let mut result = parse_json::<Self>(response).await?;
        result.ssec_algorithm = ssec_algorithm;
        result.ssec_key_md5 = ssec_key_md5;
        result.server_side_encryption = server_side_encryption;
        result.server_side_encryption_key_id = server_side_encryption_key_id;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for UploadPartOutput {
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

#[async_trait]
impl OutputParser for CompleteMultipartUploadOutput {
    async fn parse<B>(request: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let version_id = get_header_value(response.headers(), HEADER_VERSION_ID);
        let hash_crc64ecma = get_header_value_from_str::<u64>(response.headers(), HEADER_HASH_CRC64ECMA, 0)?;
        let server_side_encryption = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION);
        let server_side_encryption_key_id = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID);
        let mut result;
        if get_map_value_str(&request.header, HEADER_CALLBACK) != "" { // callback
            let etag = get_header_value(response.headers(), HEADER_ETAG);
            let location = get_header_value(response.headers(), HEADER_LOCATION);
            let buf = read_response(response).await?;
            if request_info.status_code == 203 {
                if let Ok(error_response) = parse_json_by_buf::<ErrorResponse>(buf.as_slice()) {
                    return Err(TosError::server_error_with_code(error_response.code, error_response.ec, error_response.key, error_response.message,
                                                                error_response.host_id, error_response.resource, request_info));
                }
            }
            result = Self::default();
            result.bucket = request.bucket.to_string();
            result.key = request.key.to_string();
            result.etag = etag;
            result.location = location;
            result.callback_result = parse_response_string_by_buf(buf)?;
        } else {
            result = parse_json::<Self>(response).await?;
        }

        result.version_id = version_id;
        result.hash_crc64ecma = hash_crc64ecma;
        result.server_side_encryption = server_side_encryption;
        result.server_side_encryption_key_id = server_side_encryption_key_id;
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for AbortMultipartUploadOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, _: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        Ok(Self { request_info })
    }
}

#[async_trait]
impl OutputParser for UploadPartCopyOutput {
    async fn parse<B>(request: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let copy_source_version_id = get_header_value(response.headers(), HEADER_COPY_SOURCE_VERSION_ID);
        let ssec_algorithm = get_header_value(response.headers(), HEADER_SSEC_ALGORITHM);
        let ssec_key_md5 = get_header_value(response.headers(), HEADER_SSEC_KEY_MD5);
        let server_side_encryption = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION);
        let server_side_encryption_key_id = get_header_value(response.headers(), HEADER_SERVER_SIDE_ENCRYPTION_KMS_KEY_ID);
        let temp_result = parse_json::<TempCopyResult>(response).await?;
        if temp_result.etag == "" {
            return Err(TosError::server_error_with_code(temp_result.code, temp_result.ec, temp_result.key, temp_result.message,
                                                        temp_result.host_id, temp_result.resource, request_info));
        }

        let result = Self {
            request_info,
            part_number: get_map_value_from_str(request.query.as_ref().unwrap(), QUERY_PART_NUMBER, 1)?,
            etag: temp_result.etag,
            last_modified: parse_date_time_iso8601(&temp_result.last_modified)?,
            copy_source_version_id,
            ssec_algorithm,
            ssec_key_md5,
            server_side_encryption,
            server_side_encryption_key_id,
        };
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for ListMultipartUploadsOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        for upload in &mut result.uploads {
            if let Some(x) = upload.initiated_string.take() {
                upload.initiated = parse_date_time_iso8601(&x)?;
            }
        }
        result.request_info = request_info;
        Ok(result)
    }
}

#[async_trait]
impl OutputParser for ListPartsOutput {
    async fn parse<B>(_: HttpRequest<'_, B>, response: HttpResponse, request_info: RequestInfo, _: Meta) -> Result<Self, TosError>
    where
        B: Send,
    {
        let mut result = parse_json::<Self>(response).await?;
        for part in &mut result.parts {
            if let Some(x) = part.last_modified_string.take() {
                part.last_modified = parse_date_time_iso8601(&x)?;
            }
        }
        result.request_info = request_info;
        Ok(result)
    }
}

impl<B> DataTransferListener for UploadPartInput<B> {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.inner.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.inner.async_data_transfer_listener = Some(listener.into());
    }
}

impl DataTransferListener for UploadPartFromBufferInput {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.inner.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.inner.async_data_transfer_listener = Some(listener.into());
    }
}

