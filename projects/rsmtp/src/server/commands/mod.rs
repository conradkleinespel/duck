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

use super::super::common::mailbox::Mailbox;

/// The MAIL command.
pub mod mail;

/// The HELO command.
pub mod helo;

/// The EHLO command.
pub mod ehlo;

/// The RCPT command.
pub mod rcpt;

/// Allows commands to get access to information about the state of the
/// current transaction.
pub trait HeloSeen {
    /// Returns the state object for the current connection.
    fn helo_seen(&mut self) -> bool;

    /// Sets if we have HELO or not.
    fn set_helo_seen(&mut self, helo_seen: bool);
}

/// Methods needed by the MAIL/RCPT command to read the current state.
pub trait HeloHandler {
    /// Handles the domain passed to the HELO/EHLO command.
    fn handle_domain(&mut self, domain: &str) -> Result<(), ()>;
}

/// Methods needed by the MAIL command to read the current state.
pub trait MailHandler {
    /// Handles the email address passed to the MAIL command.
    ///
    /// This will be `None` when the argument to MAIL is `<>`. This can happen
    /// when a server receives a delivery failure notification.
    fn handle_sender_address(&mut self, mailbox: Option<Mailbox>) -> Result<(), ()>;
}

/// Methods needed by the RCPT command to read the current state.
pub trait RcptHandler {
    /// Handles the email address passed to the RCPT command.
    fn handle_receiver_address(&mut self, mailbox: Mailbox) -> Result<(), ()>;
}
