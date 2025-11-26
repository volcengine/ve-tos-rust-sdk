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
use crate::common::{DataTransferStatus, DataTransferType, RateLimiter};
use crate::constant::DEFAULT_READ_BUFFER_SIZE;
use crate::error::{GenericError, TosError};
use crate::http::HttpRequest;
use crate::internal::combine_crc64;
use bytes::Bytes;
use crc64fast::Digest;
use std::fs::File;
use std::future::Future;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;

pub(crate) fn read_at_most(reader: &mut dyn Read, buf: &mut Vec<u8>, most: usize) -> Result<usize, TosError> {
    if most == 0 {
        return Ok(0);
    }

    let mut temp_buf = [0u8; DEFAULT_READ_BUFFER_SIZE];
    let mut read_total = 0usize;
    loop {
        match reader.read(&mut temp_buf) {
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(TosError::client_error_with_cause("io read error", GenericError::IoError(e.to_string()))),
            Ok(mut read_once) => {
                if read_once == 0 {
                    return Ok(read_total);
                }
                if read_total + read_once > most {
                    read_once = most - read_total;
                }
                buf.extend_from_slice(&temp_buf[0..read_once]);
                read_total += read_once;
                if read_total >= most {
                    return Ok(read_total);
                }
            }
        }
    }
}

pub(crate) trait BuildFileReader: Sized {
    fn new(input: &str) -> Result<(Self, Option<usize>), TosError>;
    fn new_with_offset(input: &str, offset: i64) -> Result<(Self, Option<usize>), TosError>;
}


impl BuildFileReader for InternalReader<File> {
    fn new(input: &str) -> Result<(Self, Option<usize>), TosError> {
        match File::open(input) {
            Ok(fd) => {
                if let Ok(x) = fd.metadata() {
                    let len = x.len() as usize;
                    return Ok((Self::sized(fd, len), Some(len)));
                }
                Ok((Self::new(fd), None))
            }
            Err(e) => Err(TosError::client_error_with_cause("open file error", GenericError::IoError(e.to_string()))),
        }
    }

    fn new_with_offset(input: &str, offset: i64) -> Result<(Self, Option<usize>), TosError> {
        let (mut fd, len) = <Self as BuildFileReader>::new(input)?;
        if offset > 0 {
            if let Err(e) = fd.seek(SeekFrom::Start(offset as u64)) {
                return Err(TosError::client_error_with_cause("seek file error", GenericError::IoError(e.to_string())));
            }
        }
        Ok((fd, len))
    }
}

pub(crate) trait BuildBufferReader: Sized {
    fn new(input: Vec<u8>) -> Result<(Self, usize), TosError>;
}

impl BuildBufferReader for InternalReader<Cursor<Vec<u8>>> {
    fn new(input: Vec<u8>) -> Result<(Self, usize), TosError> {
        let len = input.len();
        Ok(
            (Self::sized(Cursor::new(input), len), len)
        )
    }
}

pub(crate) struct InternalReader<B> {
    pub(crate) b: B,
    pub(crate) operation: String,
    pub(crate) bucket: String,
    pub(crate) key: String,
    pub(crate) total_size: Option<usize>,
    pub(crate) read_size: usize,
    pub(crate) rate_limiter: Option<Arc<RateLimiter>>,
    pub(crate) data_transfer_listener: Option<Sender<DataTransferStatus>>,
    pub(crate) async_data_transfer_listener: Option<async_channel::Sender<DataTransferStatus>>,
    pub(crate) retry_count: isize,
    pub(crate) first_read: bool,
    pub(crate) succeed_send: bool,
    pub(crate) current_future: Option<Pin<Box<dyn Future<Output=()> + Send>>>,
    pub(crate) current_result: Option<Poll<Option<Result<Bytes, crate::error::CommonError>>>>,
}

impl<B> InternalReader<B> {
    pub(crate) fn new(b: B) -> Self {
        Self {
            b,
            operation: "".to_string(),
            bucket: "".to_string(),
            key: "".to_string(),
            total_size: None,
            read_size: 0,
            rate_limiter: None,
            data_transfer_listener: None,
            async_data_transfer_listener: None,
            retry_count: 0,
            first_read: true,
            succeed_send: false,
            current_future: None,
            current_result: None,
        }
    }

    pub(crate) fn sized(b: B, len: usize) -> Self {
        Self {
            b,
            operation: "".to_string(),
            bucket: "".to_string(),
            key: "".to_string(),
            total_size: Some(len),
            read_size: 0,
            rate_limiter: None,
            data_transfer_listener: None,
            async_data_transfer_listener: None,
            retry_count: 0,
            first_read: true,
            succeed_send: false,
            current_future: None,
            current_result: None,
        }
    }

    pub(crate) fn set_rate_limiter(&mut self, rate_limiter: Arc<RateLimiter>) {
        self.rate_limiter = Some(rate_limiter);
    }
    pub(crate) fn set_data_transfer_listener(&mut self, data_transfer_listener: Sender<DataTransferStatus>) {
        self.data_transfer_listener = Some(data_transfer_listener);
    }

    pub(crate) fn set_async_data_transfer_listener(&mut self, async_data_transfer_listener: async_channel::Sender<DataTransferStatus>) {
        self.async_data_transfer_listener = Some(async_data_transfer_listener);
    }

    pub(crate) fn send_data_transfer_status(&self, data_transfer_type: DataTransferType, rw_once_bytes: i64) {
        if let Some(ref dts) = self.data_transfer_listener {
            let mut total_bytes = -1;
            if let Some(ts) = self.total_size {
                total_bytes = ts as i64;
            }
            let _ = dts.send(DataTransferStatus::new(data_transfer_type, self.retry_count)
                .set_operation(&self.operation)
                .set_bucket(&self.bucket)
                .set_key(&self.key)
                .set_consumed_bytes(self.read_size as i64)
                .set_total_bytes(total_bytes).set_rw_once_bytes(rw_once_bytes));
        }
    }
    pub(crate) fn async_send_data_transfer_status(&mut self, data_transfer_type: DataTransferType, rw_once_bytes: i64, cx: &mut Context<'_>) -> bool {
        if let Some(ctx) = self.prepare_async_send_context(data_transfer_type, rw_once_bytes) {
            let mut future = Box::pin(async move {
                let _ = ctx.sender.send(ctx.status).await;
            });

            if let Poll::Pending = future.as_mut().poll(cx) {
                self.current_future = Some(future);
                return false;
            }
        }
        true
    }
    pub(crate) fn prepare_async_send_context(&self, data_transfer_type: DataTransferType, rw_once_bytes: i64) -> Option<AsyncSendContext> {
        if let Some(ref adts) = self.async_data_transfer_listener {
            let mut total_bytes = -1;
            if let Some(ts) = self.total_size {
                total_bytes = ts as i64;
            }
            return Some(AsyncSendContext {
                sender: adts.clone(),
                status: DataTransferStatus::new(data_transfer_type, self.retry_count)
                    .set_operation(&self.operation)
                    .set_bucket(&self.bucket)
                    .set_key(&self.key)
                    .set_consumed_bytes(self.read_size as i64)
                    .set_total_bytes(total_bytes).set_rw_once_bytes(rw_once_bytes),
            });
        }
        None
    }
}

impl<B> Read for InternalReader<B>
where
    B: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        if let Some(ref rl) = self.rate_limiter {
            let mut acquire_count = 0;
            loop {
                let (ok, dur) = rl.acquire(buf.len() as i64);
                if ok {
                    break;
                }
                thread::sleep(dur.unwrap());
                acquire_count += 1;
                if acquire_count > 30 {
                    self.send_data_transfer_status(DataTransferType::DataTransferFailed, -1);
                    return Err(Error::new(ErrorKind::Other, "exceeded max acquire times"));
                }
            }
        }

        if self.first_read {
            self.send_data_transfer_status(DataTransferType::DataTransferStarted, -1);
            self.first_read = false;
        }

        match self.b.read(buf) {
            Err(e) => {
                self.send_data_transfer_status(DataTransferType::DataTransferFailed, -1);
                Err(e)
            }
            Ok(read_once) => {
                self.read_size += read_once;
                if read_once > 0 {
                    self.send_data_transfer_status(DataTransferType::DataTransferRW, read_once as i64);
                    if let Some(total_size) = self.total_size {
                        if self.read_size == total_size {
                            if !self.succeed_send {
                                self.succeed_send = true;
                                self.send_data_transfer_status(DataTransferType::DataTransferSucceed, -1);
                            }
                        }
                    }
                } else if read_once == 0 {
                    if let Some(total_size) = self.total_size {
                        if self.read_size < total_size {
                            self.send_data_transfer_status(DataTransferType::DataTransferFailed, -1);
                            return Err(Error::new(ErrorKind::UnexpectedEof,
                                                  format!("premature end, expected {}, actual {}", total_size, self.read_size)));
                        }
                    }
                    if !self.succeed_send {
                        self.succeed_send = true;
                        self.send_data_transfer_status(DataTransferType::DataTransferSucceed, -1);
                    }
                }
                Ok(read_once)
            }
        }
    }
}

impl<B> Seek for InternalReader<B>
where
    B: Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.b.seek(pos)
    }

    fn rewind(&mut self) -> std::io::Result<()> {
        self.b.rewind()
    }
    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.b.stream_position()
    }
}

pub(crate) struct MultifunctionalReader<B> {
    pub(crate) inner: InternalReader<B>,
    pub(crate) digest: Option<Digest>,
    pub(crate) crc64: Option<Arc<AtomicU64>>,
    pub(crate) init_crc64: Option<u64>,
    pub(crate) target_crc64: Option<u64>,
}

pub(crate) struct AsyncSendContext {
    pub(crate) sender: async_channel::Sender<DataTransferStatus>,
    pub(crate) status: DataTransferStatus,
}

impl<B> MultifunctionalReader<B> {
    pub(crate) fn new<C>(b: B, crc64: Option<Arc<AtomicU64>>, cl: i64, request: &HttpRequest<'_, C>) -> Self {
        Self::with_target_crc64(b, crc64, cl, request, None)
    }

    pub(crate) fn with_target_crc64<C>(b: B, crc64: Option<Arc<AtomicU64>>, cl: i64, request: &HttpRequest<'_, C>, target_crc64: Option<u64>) -> Self {
        let mut digest = None;
        if crc64.is_some() {
            digest = Some(Digest::new());
        }
        let mut total_size = None;
        if cl >= 0 {
            total_size = Some(cl as usize);
        }
        let mut inner = InternalReader::new(b);
        inner.retry_count = request.retry_count;
        inner.total_size = total_size;
        inner.operation = request.operation.to_string();
        inner.bucket = request.bucket.to_string();
        inner.key = request.key.to_string();
        Self {
            inner,
            digest,
            crc64,
            init_crc64: None,
            target_crc64,
        }
    }

    pub(crate) fn set_crc64(&self) -> std::io::Result<()> {
        let result;
        if let Some(init_crc64) = self.init_crc64 {
            result = combine_crc64(init_crc64, self.digest.as_ref().unwrap().sum64(), self.inner.read_size);
        } else {
            result = self.digest.as_ref().unwrap().sum64();
        }

        if let Ok(_) = self.crc64.as_ref().unwrap().compare_exchange(0, result, Ordering::AcqRel, Ordering::Relaxed) {
            if let Some(target_crc64) = self.target_crc64 {
                if target_crc64 != result {
                    return Err(Error::new(ErrorKind::Other, format!("expect crc64 {target_crc64}, actual crc64 {result}")));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn set_rate_limiter(&mut self, rate_limiter: Arc<RateLimiter>) {
        self.inner.set_rate_limiter(rate_limiter);
    }
    pub(crate) fn set_data_transfer_listener(&mut self, data_transfer_listener: Sender<DataTransferStatus>) {
        self.inner.set_data_transfer_listener(data_transfer_listener);
    }

    pub(crate) fn set_async_data_transfer_listener(&mut self, async_data_transfer_listener: async_channel::Sender<DataTransferStatus>) {
        self.inner.set_async_data_transfer_listener(async_data_transfer_listener);
    }
}

impl<B> Read for MultifunctionalReader<B>
where
    B: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read_once = self.inner.read(buf)?;
        if read_once > 0 {
            if self.digest.is_some() {
                self.digest.as_mut().unwrap().write(&buf[..read_once]);
            }
            if let Some(total_size) = self.inner.total_size {
                if self.inner.read_size == total_size {
                    if self.digest.is_some() {
                        self.set_crc64()?;
                    }
                }
            }
        } else {
            if self.digest.is_some() {
                self.set_crc64()?;
            }
        }
        Ok(read_once)
    }
}
