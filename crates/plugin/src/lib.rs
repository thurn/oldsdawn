// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(clippy::missing_safety_doc)] // You only live once, that's the motto - Drake

//! Implements a DLL for Unity to call into the Spelldawn API

use std::str;

use anyhow::Result;
use cards::initialize;
use prost::Message;
use protos::spelldawn::{ConnectRequest, GameRequest};
use server::{database, requests};

/// Initialize the plugin. Must be called immediately at application start.
///
/// Should be invoked with a buffer containing a UTF-8 encoded string of the
/// database path the plugin should use, along with its length.
///
/// Returns 0 on success and -1 on error.
#[no_mangle]
pub unsafe extern "C" fn spelldawn_initialize(path: *const u8, path_length: i32) -> i32 {
    initialize_impl(path, path_length).unwrap_or(-1)
}

unsafe fn initialize_impl(path: *const u8, path_length: i32) -> Result<i32> {
    initialize::run();
    let slice = std::slice::from_raw_parts(path, path_length as usize);
    let db_path = str::from_utf8(slice)?;
    // let tmp = tmp_plugin_test::db_test(db_path.to_string());
    database::override_path(db_path.to_string());
    println!("Initialized libspelldawn with database path {}", db_path);
    Ok(0)
}

/// Synchronize the state of an ongoing game, downloading a full description of
/// the game state.
///
/// `request` should be a buffer including the protobuf serialization of a
/// `ConnectRequest` message of `request_length` bytes. `response` should be an
/// empty buffer of `response_length` bytes, this buffer will be populated with
/// a protobuf-serialized `CommandList` describing the current state of the
/// game.
///
/// Returns the number of bytes written to the `response` buffer, or -1 on
/// error.
#[no_mangle]
pub unsafe extern "C" fn spelldawn_connect(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    connect_impl(request, request_length, response, response_length).unwrap_or(-1)
}

unsafe fn connect_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    let request_data = std::slice::from_raw_parts(request, request_length as usize);
    let connect_request = ConnectRequest::decode(request_data)?;
    let command_list = requests::connect(connect_request)?;
    let mut out = std::slice::from_raw_parts_mut(response, response_length as usize);
    command_list.encode(&mut out)?;
    Ok(command_list.encoded_len() as i32)
}

/// Performs a given game action.
///
/// `request` should be a buffer including the protobuf serialization of a
/// `GameRequest` message of `request_length` bytes. `response` should be an
/// empty buffer of `response_length` bytes, this buffer will be populated with
/// a protobuf-serialized `CommandList` describing the result of performing this
/// action.
///
/// Returns the number of bytes written to the `response` buffer, or -1 on
/// error.
#[no_mangle]
pub unsafe extern "C" fn spelldawn_perform_action(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> i32 {
    perform_impl(request, request_length, response, response_length).unwrap_or(-1)
}

unsafe fn perform_impl(
    request: *const u8,
    request_length: i32,
    response: *mut u8,
    response_length: i32,
) -> Result<i32> {
    let request_data = std::slice::from_raw_parts(request, request_length as usize);
    let game_request = GameRequest::decode(request_data)?;
    let command_list = requests::perform_action(game_request)?;
    let mut out = std::slice::from_raw_parts_mut(response, response_length as usize);
    command_list.encode(&mut out)?;
    Ok(command_list.encoded_len() as i32)
}
