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
use crate::asynchronous::common::DataTransferListener;
use crate::asynchronous::http::HttpResponse;
use crate::asynchronous::internal::{AsyncInputTranslator, OutputParser};
use crate::asynchronous::reader::StreamAdapter;
use crate::asynchronous::tos::{AsyncRuntime, TosClientImpl};
use crate::common::{DataTransferStatus, Meta, RequestInfo, RequestInfoTrait};
use crate::config::ConfigHolder;
use crate::constant::{HEADER_CONTENT_LENGTH, HEADER_CONTENT_RANGE, HEADER_RANGE, QUERY_PROCESS, UUID_NODE};
use crate::credential::{Credentials, CredentialsProvider};
use crate::error::{GenericError, TosError};
use crate::http::HttpRequest;
use crate::internal::{get_header_value, InputDescriptor, InputTranslator, MockAsyncInputTranslator};
use crate::multipart::UploadPartFromFileInput;
use crate::object::{GetObjectToFileInput, GetObjectToFileOutput, HeadObjectOutput, PutObjectFromFileInput};
use crate::reader::MultifunctionalReader;
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use futures_util::StreamExt;
use std::io::{Error, ErrorKind, SeekFrom};
use std::path::Path;
use std::pin::Pin;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

pub(crate) struct FileReader {
    pub(crate) b: ReaderStream<File>,
    pub(crate) total_size: Option<usize>,
    pub(crate) read_size: usize,
}

impl FileReader {
    async fn open(input: &str) -> Result<(File, Option<usize>), TosError> {
        match File::open(input).await {
            Ok(fd) => {
                if let Ok(x) = fd.metadata().await {
                    let len = x.len() as usize;
                    return Ok((fd, Some(len)));
                }
                Ok((fd, None))
            }
            Err(e) => Err(TosError::client_error_with_cause("open file error", GenericError::IoError(e.to_string()))),
        }
    }
}

impl Stream for FileReader
{
    type Item = Result<Bytes, crate::error::CommonError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.b.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(opt) => {
                match opt {
                    None => {
                        if let Some(total_size) = self.total_size {
                            if self.read_size < total_size {
                                return Poll::Ready(Some(Err(Error::new(ErrorKind::UnexpectedEof, format!("premature end, expected {}, actual {}", total_size, self.read_size)))));
                            }
                        }
                        Poll::Ready(None)
                    }
                    Some(result) => {
                        match result {
                            Err(e) => Poll::Ready(Some(Err(Error::new(ErrorKind::Other, e.to_string())))),
                            Ok(x) => {
                                self.read_size += x.len();
                                Poll::Ready(Some(Ok(x)))
                            }
                        }
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.b.size_hint()
    }
}

#[async_trait]
pub(crate) trait BuildFileStream: Sized {
    async fn new(input: &str) -> Result<(Self, Option<usize>), TosError>;
    async fn new_with_offset(input: &str, offset: i64) -> Result<(Self, Option<usize>), TosError>;
}
#[async_trait]
impl BuildFileStream for FileReader {
    async fn new(input: &str) -> Result<(Self, Option<usize>), TosError> {
        let (fd, len) = Self::open(input).await?;
        Ok((Self { b: ReaderStream::new(fd), total_size: len, read_size: 0 }, len))
    }

    async fn new_with_offset(input: &str, offset: i64) -> Result<(Self, Option<usize>), TosError> {
        let (mut fd, len) = Self::open(input).await?;
        if offset > 0 {
            if let Err(e) = fd.seek(SeekFrom::Start(offset as u64)).await {
                return Err(TosError::client_error_with_cause("seek file error", GenericError::IoError(e.to_string())));
            }
        }
        Ok((Self { b: ReaderStream::new(fd), total_size: len, read_size: 0 }, len))
    }
}

impl<P, C, S> TosClientImpl<P, C, S>
where
    P: CredentialsProvider<C> + Send + Sync + 'static,
    C: Credentials + Send + Sync + 'static,
    S: AsyncRuntime + Send + Sync + 'static,
{
    pub(crate) async fn do_request_af<F, K, B>(&self, input: &F) -> Result<K, TosError>
    where
        F: AsyncInputTranslator<B>,
        K: OutputParser + RequestInfoTrait + Send,
        B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Send + Unpin + 'static,
    {
        self.do_request_common::<MockAsyncInputTranslator, F, K, B>(None, Some(input)).await
    }
}


#[async_trait]
impl<B> AsyncInputTranslator<B> for PutObjectFromFileInput
where
    B: BuildFileStream + Send,
{
    async fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.inner.trans(config_holder)?;
        request.operation = self.operation();
        if self.file_path != "" {
            let (body, len) = B::new(&self.file_path).await?;
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

#[async_trait]
impl<B> AsyncInputTranslator<B> for UploadPartFromFileInput
where
    B: BuildFileStream + Send,
{
    async fn trans(&self, config_holder: Arc<ConfigHolder>) -> Result<HttpRequest<B>, TosError> {
        let mut request = self.inner.trans(config_holder)?;
        request.operation = self.operation();
        if self.offset < 0 {
            return Err(TosError::client_error("invalid offset for upload part"));
        }
        if self.part_size >= 0 {
            request.header.insert(HEADER_CONTENT_LENGTH, self.part_size.to_string());
        }
        if self.file_path != "" {
            let (body, len) = B::new_with_offset(&self.file_path, self.offset).await?;
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

#[async_trait]
impl OutputParser for GetObjectToFileOutput {
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

        let file_path = &request.request_context.as_ref().unwrap().file_path;
        let path = Path::new(file_path);
        match path.parent() {
            None => return Err(TosError::client_error(format!("cannot get parent for path {}", file_path))),
            Some(p) => {
                if !p.exists() {
                    if let Err(e) = fs::create_dir_all(p).await {
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
        match File::options().write(true).truncate(true).create(true).open(temp_file_path.clone()).await {
            Err(e) => {
                return Err(TosError::client_error_with_cause("open file to write error", GenericError::IoError(e.to_string())))
            }
            Ok(mut fd) => {
                loop {
                    match reader.next().await {
                        None => break,
                        Some(result) => {
                            match result {
                                Err(re) => {
                                    let _ = fs::remove_file(temp_file_path).await;
                                    return Err(TosError::client_error_with_cause("read content to write error", GenericError::IoError(re.to_string())));
                                }
                                Ok(data) => {
                                    if let Err(we) = fd.write_all(data.as_ref()).await {
                                        let _ = fs::remove_file(temp_file_path).await;
                                        return Err(TosError::client_error_with_cause("write data to file error", GenericError::IoError(we.to_string())));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Err(re) = fs::rename(temp_file_path.clone(), final_file_path).await {
            let _ = fs::remove_file(temp_file_path).await;
            return Err(TosError::client_error_with_cause("rename file error", GenericError::IoError(re.to_string())));
        }

        Ok(Self {
            content_range,
            head_object_output,
        })
    }
}

impl DataTransferListener for PutObjectFromFileInput {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.inner.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.inner.async_data_transfer_listener = Some(listener.into());
    }
}

impl DataTransferListener for GetObjectToFileInput {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.inner.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.inner.async_data_transfer_listener = Some(listener.into());
    }
}

impl DataTransferListener for UploadPartFromFileInput {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>> {
        &self.inner.async_data_transfer_listener
    }

    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>) {
        self.inner.async_data_transfer_listener = Some(listener.into());
    }
}