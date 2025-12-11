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
use crate::asynchronous::credential::CredentialsProvider;
use crate::asynchronous::object::ObjectAPI;
use crate::asynchronous::tos::{AsyncRuntime, TosClientImpl};
use crate::credential::Credentials;
use crate::error::TosError;
use crate::internal::check_bucket_and_key;
use crate::object::{ListObjectsType2Input, ListObjectsType2Output};
use crate::paginator::{NextPageInput, Paginator};
use crate::tos::ConfigAware;
use arc_swap::ArcSwap;
use async_channel::Receiver;
use async_trait::async_trait;
use std::sync::Arc;

pub trait PaginatorAPI {
    fn new_list_objects_type2_paginator(self: &Arc<Self>, input: &ListObjectsType2Input, recursive_for_delimiter: bool) -> Result<impl ListObjectsType2Paginator, TosError>;
}

impl<P, C, S> PaginatorAPI for TosClientImpl<P, C, S>
where
    P: CredentialsProvider<C> + Send + Sync + 'static,
    C: Credentials + Clone + Send + Sync + 'static,
    S: AsyncRuntime + Send + Sync + 'static,
{
    fn new_list_objects_type2_paginator(self: &Arc<Self>, input: &ListObjectsType2Input, recursive_for_delimiter: bool) -> Result<impl ListObjectsType2Paginator, TosError>
    {
        let _ = check_bucket_and_key(input, self.is_custom_domain())?;
        let mut _input = input.clone();
        let _client = self.clone();
        let (sender, receiver) = async_channel::bounded(3);
        let _ = self.async_runtime.spawn(async move {
            let mut need_break = false;
            if _input.delimiter == "/" {
                let mut prefixes = Vec::with_capacity(16);
                let mut last_page_end = false;
                loop {
                    if last_page_end {
                        if prefixes.is_empty() {
                            let _ = sender.send((true, Err(TosError::client_error("invalid status error")))).await;
                            break;
                        }
                        let prefix = prefixes.remove(0);
                        _input.prefix = prefix;
                        _input.start_after = "".to_string();
                        _input.continuation_token = "".to_string();
                        last_page_end = false;
                    }
                    let result = _client.list_objects_type2(&_input).await;
                    if let Ok(ref o) = result {
                        if o.is_truncated() {
                            _input.continuation_token = o.next_continuation_token.clone();
                        } else {
                            last_page_end = true;
                        }

                        if recursive_for_delimiter {
                            for cp in o.common_prefixes() {
                                prefixes.push(cp.prefix().to_string());
                            }
                        }
                        need_break = last_page_end && prefixes.is_empty();
                    } else {
                        need_break = true;
                    }

                    if let Err(_) = sender.send((need_break, result)).await {
                        need_break = true;
                    }
                    if need_break {
                        break;
                    }
                }
            } else {
                loop {
                    let result = _client.list_objects_type2(&_input).await;
                    if let Ok(ref o) = result {
                        if o.is_truncated() {
                            _input.continuation_token = o.next_continuation_token.clone();
                        } else {
                            need_break = true;
                        }
                    } else {
                        need_break = true;
                    }

                    if let Err(_) = sender.send((need_break, result)).await {
                        need_break = true
                    }
                    if need_break {
                        break;
                    }
                }
            }
            sender.close();
        });

        Ok(PaginatorImpl {
            is_end: ArcSwap::new(Arc::new(false)),
            last_err: ArcSwap::new(Arc::new(None)),
            current_prefix: ArcSwap::new(Arc::new(input.prefix.clone())),
            receiver,
        })
    }
}


#[async_trait]
pub trait ListObjectsType2Paginator: Paginator {
    async fn next_page(&self, input: &NextPageInput) -> Result<ListObjectsType2Output, TosError>;
}


pub(crate) struct PaginatorImpl
{
    is_end: ArcSwap<bool>,
    last_err: ArcSwap<Option<TosError>>,
    current_prefix: ArcSwap<String>,
    receiver: Receiver<(bool, Result<ListObjectsType2Output, TosError>)>,
}

impl Paginator for PaginatorImpl {
    fn has_next(&self) -> bool {
        if self.last_err.load().is_some() {
            return false;
        }
        !*self.is_end.load().as_ref()
    }

    fn current_prefix(&self) -> Arc<String> {
        self.current_prefix.load().clone()
    }

    fn close(&self) {
        self.receiver.close();
    }
}

#[async_trait]
impl ListObjectsType2Paginator for PaginatorImpl
{
    async fn next_page(&self, _: &NextPageInput) -> Result<ListObjectsType2Output, TosError> {
        if let Some(e) = self.last_err.load().as_ref() {
            return Err(e.clone());
        }
        if *self.is_end.load().as_ref() {
            return Err(TosError::client_error("no next page error"));
        }

        match self.receiver.recv().await {
            Err(_) => {
                self.is_end.store(Arc::new(true));
                Err(TosError::client_error("no next page error"))
            }
            Ok((is_end, result)) => {
                match result {
                    Err(e) => {
                        self.last_err.store(Arc::new(Some(e.clone())));
                        Err(e)
                    }
                    Ok(output) => {
                        self.current_prefix.store(Arc::new(output.prefix().to_string()));
                        if is_end {
                            self.is_end.store(Arc::new(true));
                        }
                        Ok(output)
                    }
                }
            }
        }
    }
}