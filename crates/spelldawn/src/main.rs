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

use protos::spelldawn::spelldawn_server::SpelldawnServer;
use server::GameService;
use tonic::transport::Server;
use tracing::warn;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fmt_layer = fmt::Layer::default().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(fmt_layer).init();

    let address = "127.0.0.1:50052".parse().expect("valid address");
    let service = tonic_web::config()
        .allow_origins(vec!["127.0.0.1"])
        .enable(SpelldawnServer::new(GameService {}));

    warn!("Server listening on {}", address);
    Server::builder().accept_http1(true).add_service(service).serve(address).await?;
    Ok(())
}
