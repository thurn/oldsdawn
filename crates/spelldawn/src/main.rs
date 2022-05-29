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

//! Spelldawn: An asymmetric trading card game

use protos::spelldawn::spelldawn_server::SpelldawnServer;
use server::requests::GameService;
use tonic::transport::Server;
use tracing::warn;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ai::core::initialize();
    let found_cards = cards::initialize();
    let fmt_layer = fmt::Layer::default().pretty().with_filter(LevelFilter::WARN);
    tracing_subscriber::registry().with(fmt_layer).init();

    let address = "0.0.0.0:50052".parse().expect("valid address");
    let server = SpelldawnServer::new(GameService {
        // To print responses:
        // response_interceptor: Some(|response| eprintln!("{}", Summary::summarize(response)))
        response_interceptor: None,
    })
    .send_gzip()
    .accept_gzip();
    let service = tonic_web::config().enable(server);

    warn!("Discovered {} cards. Server listening on {}.", found_cards, address);
    Server::builder().accept_http1(true).add_service(service).serve(address).await?;
    Ok(())
}
