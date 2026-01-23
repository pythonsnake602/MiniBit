/*
    MiniBit - A Minecraft minigame server network written in Rust.
    Copyright (C) 2026  Cheezer1656 (https://github.com/Cheezer1656/)

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

mod subservers {
    automod::dir!(pub "src/bin/minibit/subservers");
}

use std::error::Error;
use crate::subservers::*;
use clap::{Args, FromArgMatches, arg, command, value_parser};
use figment::providers::{Env, Serialized};
use figment::{
    Figment,
    providers::{Format, Yaml},
};
use minibit_lib::config::NetworkConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::thread;
use std::thread::JoinHandle;
use opentelemetry::global;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use valence::log::tracing_subscriber::layer::SubscriberExt;
use valence::log::tracing_subscriber::Registry;
use minibit_lib::telemetry::TelemetryConfig;

#[macro_export]
macro_rules! subserver {
    ($server:ident, $config:expr) => {
        (
            stringify!($server),
            $server::main as fn(GlobalConfig, ServerConfig),
            $config.$server,
        )
    };
}

#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(default)]
struct ServerConfig {
    enabled: bool,
    path: PathBuf,
    network: NetworkConfig,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
struct ForwardingConfig {
    secret: String,
    mode: u8,
}

impl Default for ForwardingConfig {
    fn default() -> Self {
        ForwardingConfig {
            secret: "".to_string(),
            mode: 1,
        }
    }
}

#[derive(Copy, Clone)]
struct GlobalConfig {
    telemetry: TelemetryConfig,
}

#[rustfmt::skip]
#[derive(Args, Default, Deserialize, Serialize)]
#[serde(default)]
struct Config {
    #[arg(long, default_value = "data")]
    data_path: PathBuf,

    #[clap(skip)] telemetry: TelemetryConfig,

    #[clap(skip)] forwarding: ForwardingConfig,

    #[clap(skip)] lobby: ServerConfig,
    #[clap(skip)] bedwars: ServerConfig,
    #[clap(skip)] bowfight: ServerConfig,
    #[clap(skip)] boxing: ServerConfig,
    #[clap(skip)] bridge: ServerConfig,
    #[clap(skip)] classic: ServerConfig,
    #[clap(skip)] parkour: ServerConfig,
    #[clap(skip)] spaceshooter: ServerConfig,
    #[clap(skip)] sumo: ServerConfig,
    #[clap(skip)] trainchase: ServerConfig,
}

fn enable_trace() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()?;

    // Create a tracer provider with the exporter
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .build();

    // Set it as the global provider
    global::set_tracer_provider(tracer_provider);

    let tracer = global::tracer("bevy");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default().with(telemetry);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

fn main() {
    let cli = command!().arg(
        arg!(-c --config <FILE>)
            .required(false)
            .value_parser(value_parser!(PathBuf)),
    );
    let cli = Config::augment_args(cli);
    let matches = cli.get_matches();

    let config_path = matches.get_one::<PathBuf>("config");

    let derived_matches = Config::from_arg_matches(&matches)
        .map_err(|e| e.exit())
        .unwrap();

    let config = Figment::new()
        .merge(Serialized::defaults(derived_matches))
        .merge(Yaml::file(
            config_path.unwrap_or(&PathBuf::from("config.yml")),
        ))
        .merge(Env::prefixed("MINIBIT_").split("_"))
        .extract::<Config>();

    if let Err(e) = &config {
        eprintln!("Error: {}", e);
    }

    let config = config.unwrap();

    if config.telemetry.tracing {
        println!("Enabling tracing");
        if let Err(e) = enable_trace() {
            eprintln!("Tracing Error: {}", e)
        }
    }

    #[allow(clippy::type_complexity)]
    let subservers: Vec<(&str, fn(GlobalConfig, ServerConfig), ServerConfig)> = vec![
        subserver!(lobby, config),
        subserver!(bedwars, config),
        subserver!(bowfight, config),
        subserver!(boxing, config),
        subserver!(bridge, config),
        subserver!(classic, config),
        subserver!(parkour, config),
        subserver!(spaceshooter, config),
        subserver!(sumo, config),
        subserver!(trainchase, config),
    ];

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    let global_config = GlobalConfig {
        telemetry: config.telemetry,
    };
    
    for (server, run, server_config) in subservers {
        if !server_config.enabled {
            continue;
        }

        let mut cloned_config = server_config.clone();
        cloned_config.path = config.data_path.join(cloned_config.path);
        cloned_config.network.forwarding_secret = config.forwarding.secret.clone();
        cloned_config.network.connection_mode = config.forwarding.mode;
        println!("{}", cloned_config.network.forwarding_secret);


        println!("Starting server {}", server);
        handles.push(thread::spawn(move || {
            run(global_config, cloned_config);
        }));
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}
