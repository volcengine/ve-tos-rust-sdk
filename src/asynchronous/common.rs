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
use crate::common::DataTransferStatus;

pub trait DataTransferListener {
    fn async_data_transfer_listener(&self) -> &Option<async_channel::Sender<DataTransferStatus>>;
    fn set_async_data_transfer_listener(&mut self, listener: impl Into<async_channel::Sender<DataTransferStatus>>);
}