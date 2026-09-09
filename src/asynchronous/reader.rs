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
use crate::common::DataTransferType;
use crate::error::TosError;
use crate::reader::{BuildBufferReader, InternalReader, MultiBytes, MultifunctionalReader};
use bytes::Bytes;
use futures_core::Stream;
use futures_util::StreamExt;
use std::future::Future;
use std::io::{Error, ErrorKind};
use std::pin::Pin;
use std::task::{Context, Poll};

pub(crate) async fn read_at_most<R: Stream<Item=Result<Bytes, crate::error::CommonError>> + Unpin + ?Sized>(reader: &mut R, buf: &mut Vec<u8>, most: usize) -> Result<usize, crate::error::CommonError> {
    if most == 0 {
        return Ok(0);
    }
    let mut read_total = 0usize;
    loop {
        match reader.next().await {
            None => return Ok(read_total),
            Some(result) => {
                let x = result?;
                let mut read_once = x.len();
                if read_total + read_once > most {
                    read_once = most - read_total;
                }
                buf.extend_from_slice(x.slice(0..read_once).as_ref());
                read_total += read_once;
                if read_total >= most {
                    return Ok(read_total);
                }
            }
        }
    }
}

impl<B> Stream for InternalReader<B>
where
    B: Stream<Item=reqwest::Result<Bytes>> + Unpin,
{
    type Item = Result<Bytes, crate::error::CommonError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(mut future) = self.current_future.take() {
            if let Poll::Pending = future.as_mut().poll(cx) {
                self.current_future = Some(future);
                return Poll::Pending;
            }
        }

        if let Some(result) = self.current_result.take() {
            return result;
        }

        if self.succeed_send {
            return Poll::Ready(None);
        }

        if self.first_read {
            self.first_read = false;
            if !self.async_send_data_transfer_status(DataTransferType::DataTransferStarted, -1, cx) {
                return Poll::Pending;
            }
        }
        match self.b.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(opt) => {
                match opt {
                    None => {
                        if let Some(total_size) = self.total_size {
                            if self.read_size < total_size {
                                let result = Poll::Ready(Some(Err(Error::new(ErrorKind::Other, format!("premature end, expected {}, actual {}", total_size, self.read_size)))));
                                if !self.async_send_data_transfer_status(DataTransferType::DataTransferFailed, -1, cx) {
                                    self.current_result = Some(result);
                                    return Poll::Pending;
                                }
                                return result;
                            }
                        }

                        if !self.succeed_send {
                            self.succeed_send = true;
                            if !self.async_send_data_transfer_status(DataTransferType::DataTransferSucceed, -1, cx) {
                                self.current_result = Some(Poll::Ready(None));
                                return Poll::Pending;
                            }
                        }
                        Poll::Ready(None)
                    }
                    Some(result) => {
                        match result {
                            Err(e) => {
                                let result = Poll::Ready(Some(Err(Error::new(ErrorKind::Other, e.to_string()))));
                                if !self.async_send_data_transfer_status(DataTransferType::DataTransferFailed, -1, cx) {
                                    self.current_result = Some(result);
                                    return Poll::Pending;
                                }
                                result
                            }
                            Ok(x) => {
                                self.read_size += x.len();
                                if x.len() > 0 {
                                    if !self.async_send_data_transfer_status(DataTransferType::DataTransferRW, x.len() as i64, cx) {
                                        self.current_result = Some(Poll::Ready(Some(Ok(x))));
                                        return Poll::Pending;
                                    }
                                    if let Some(total_size) = self.total_size {
                                        if self.read_size == total_size {
                                            if !self.succeed_send {
                                                self.succeed_send = true;
                                                if !self.async_send_data_transfer_status(DataTransferType::DataTransferSucceed, -1, cx) {
                                                    self.current_result = Some(Poll::Ready(Some(Ok(x))));
                                                    return Poll::Pending;
                                                }
                                            }
                                        }
                                    }
                                }
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

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct StreamVec(Option<Bytes>);

impl Stream for StreamVec {
    type Item = reqwest::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.0.is_none() {
            return Poll::Ready(None);
        }
        Poll::Ready(Some(Ok(self.0.take().unwrap())))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0 {
            None => (0, None),
            Some(v) => (0, Some(v.len()))
        }
    }
}

impl BuildBufferReader for InternalReader<StreamVec> {
    fn new(input: Bytes) -> Result<(Self, usize), TosError> {
        let len = input.len();
        Ok(
            (Self::sized(StreamVec(Some(input)), len), len)
        )
    }
}

impl Stream for MultiBytes {
    type Item = reqwest::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.inner.is_empty() {
            return Poll::Ready(None);
        }
        Poll::Ready(Some(Ok(self.inner.pop_front().unwrap())))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.inner.is_empty() {
            return (0, None);
        }
        (0, Some(self.size))
    }
}

impl<B> MultifunctionalReader<B>
where
    B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Unpin,
{
    #[cfg(feature = "tokio-runtime")]
    fn wait_rate_limit(&mut self, duration: std::time::Duration, b: Bytes, read_end: bool, cx: &mut Context<'_>)
                       -> Poll<Option<Result<Bytes, crate::error::CommonError>>> {
        let mut future = Box::pin(async move {
            tokio::time::sleep(duration).await;
        });
        // [Review Fix] 必须立即 poll sleep future 以向 timer 注册 waker，
        // 否则在 current_thread runtime 中无人唤醒该 task，stream 将永久 Pending。
        match future.as_mut().poll(cx) {
            Poll::Ready(()) => {
                // 定时器已过期（duration ≈ 0），直接重试 rate limit
                return self.poll_ready_bytes(b, read_end, cx);
            }
            Poll::Pending => {
                self.pending_bytes = Some(b);
                self.pending_read_end = read_end;
                self.inner.current_future = Some(future);
                Poll::Pending
            }
        }
    }

    #[cfg(not(feature = "tokio-runtime"))]
    fn wait_rate_limit(&mut self, _: std::time::Duration, _: Bytes, _: bool, _: &mut Context<'_>)
                       -> Poll<Option<Result<Bytes, crate::error::CommonError>>> {
        Poll::Ready(Some(Err(Error::new(
            ErrorKind::Other,
            "async rate limiter requires feature `tokio-runtime`",
        ))))
    }

    fn poll_ready_bytes(&mut self, b: Bytes, read_end: bool, cx: &mut Context<'_>)
                        -> Poll<Option<Result<Bytes, crate::error::CommonError>>> {
        if let Some(ref rl) = self.inner.rate_limiter {
            let (ok, dur) = rl.acquire(b.len() as i64);
            if !ok {
                // [Review Fix #1] 限制最大获取次数，防止因分块大于令牌桶容量导致无限等待
                self.acquire_count += 1;
                if self.acquire_count > 30 {
                    let result = Poll::Ready(Some(Err(Error::new(ErrorKind::Other, "exceeded max acquire times"))));
                    if !self.inner.async_send_data_transfer_status(DataTransferType::DataTransferFailed, -1, cx) {
                        self.inner.current_result = Some(result);
                        return Poll::Pending;
                    }
                    return result;
                }
                return self.wait_rate_limit(dur.unwrap(), b, read_end, cx);
            }
            self.acquire_count = 0;
        }

        if !self.inner.async_send_data_transfer_status(DataTransferType::DataTransferRW, b.len() as i64, cx) {
            self.inner.current_result = Some(Poll::Ready(Some(Ok(b))));
            return Poll::Pending;
        }

        if read_end && !self.inner.succeed_send {
            self.inner.succeed_send = true;
            if !self.inner.async_send_data_transfer_status(DataTransferType::DataTransferSucceed, -1, cx) {
                self.inner.current_result = Some(Poll::Ready(Some(Ok(b))));
                return Poll::Pending;
            }
        }
        Poll::Ready(Some(Ok(b)))
    }
}


impl<B> Stream for MultifunctionalReader<B>
where
    B: Stream<Item=Result<Bytes, crate::error::CommonError>> + Unpin,
{
    type Item = Result<Bytes, crate::error::CommonError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(mut future) = self.inner.current_future.take() {
            if let Poll::Pending = future.as_mut().poll(cx) {
                self.inner.current_future = Some(future);
                return Poll::Pending;
            }
        }

        if let Some(result) = self.inner.current_result.take() {
            return result;
        }

        if let Some(b) = self.pending_bytes.take() {
            let read_end = self.pending_read_end;
            self.pending_read_end = false;
            return self.poll_ready_bytes(b, read_end, cx);
        }

        if self.inner.succeed_send {
            return Poll::Ready(None);
        }

        if self.inner.first_read {
            self.inner.first_read = false;
            if !self.inner.async_send_data_transfer_status(DataTransferType::DataTransferStarted, -1, cx) {
                return Poll::Pending;
            }
        }

        match self.inner.b.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(opt) => {
                match opt {
                    None => {
                        if let Some(total_size) = self.inner.total_size {
                            if self.inner.read_size < total_size {
                                let result = Poll::Ready(Some(Err(Error::new(ErrorKind::Other, format!("premature end, expected {}, actual {}", total_size, self.inner.read_size)))));
                                if !self.inner.async_send_data_transfer_status(DataTransferType::DataTransferFailed, -1, cx) {
                                    self.inner.current_result = Some(result);
                                    return Poll::Pending;
                                }
                                return result;
                            }
                        }

                        if self.digest.is_some() {
                            if let Err(ex) = self.set_crc64() {
                                return Poll::Ready(Some(Err(ex)));
                            }
                        }
                        if !self.inner.succeed_send {
                            self.inner.succeed_send = true;
                            if !self.inner.async_send_data_transfer_status(DataTransferType::DataTransferSucceed, -1, cx) {
                                self.inner.current_result = Some(Poll::Ready(None));
                                return Poll::Pending;
                            }
                        }
                        Poll::Ready(None)
                    }
                    Some(result) => {
                        if result.is_err() {
                            if !self.inner.async_send_data_transfer_status(DataTransferType::DataTransferFailed, -1, cx) {
                                self.inner.current_result = Some(Poll::Ready(Some(result)));
                                return Poll::Pending;
                            }
                            return Poll::Ready(Some(result));
                        }

                        let mut b = result.unwrap();
                        if b.len() > 0 {
                            let mut read_end = false;
                            if let Some(total_size) = self.inner.total_size {
                                if self.inner.read_size + b.len() > total_size {
                                    b.truncate(b.len() - (self.inner.read_size + b.len() - total_size));
                                }
                                self.inner.read_size += b.len();
                                if self.digest.is_some() {
                                    self.digest.as_mut().unwrap().write(b.as_ref());
                                }
                                if self.inner.read_size == total_size {
                                    if self.digest.is_some() {
                                        if let Err(ex) = self.set_crc64() {
                                            return Poll::Ready(Some(Err(ex)));
                                        }
                                    }
                                    read_end = true;
                                }
                            } else {
                                self.inner.read_size += b.len();
                                if self.digest.is_some() {
                                    self.digest.as_mut().unwrap().write(b.as_ref());
                                }
                            }
                            return self.poll_ready_bytes(b, read_end, cx);
                        }
                        Poll::Ready(Some(Ok(b)))
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.b.size_hint()
    }
}

pub(crate) struct StreamAdapter<B> {
    pub(crate) b: B,
}

impl<B> StreamAdapter<B> {
    pub(crate) fn new(b: B) -> Self {
        Self {
            b,
        }
    }
}

impl<B> Stream for StreamAdapter<B>
where
    B: Stream<Item=reqwest::Result<Bytes>> + Unpin,
{
    type Item = Result<Bytes, crate::error::CommonError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.b.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(opt) => {
                match opt {
                    None => Poll::Ready(None),
                    Some(result) => {
                        match result {
                            Err(e) => Poll::Ready(Some(Err(Error::new(ErrorKind::Other, e.to_string())))),
                            Ok(x) => Poll::Ready(Some(Ok(x))),
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
