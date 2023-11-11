// Copyright 2014 The Rustastic SMTP Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The `common` module contains things that can be used by both an SMTP server and an SMTP client.

pub mod mailbox;
pub mod stream;
pub mod utils;

pub static MIN_ALLOWED_MESSAGE_SIZE: usize = 65536;

pub static MIN_ALLOWED_LINE_SIZE: usize = 1000;

pub static MIN_ALLOWED_RECIPIENTS: usize = 100;
