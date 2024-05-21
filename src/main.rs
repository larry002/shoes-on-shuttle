mod address;
mod async_stream;
mod client_proxy_selector;
mod config;
mod copy_bidirectional;
mod copy_bidirectional_message;
mod copy_multidirectional_message;
mod http_handler;
mod line_reader;
mod option_util;
mod port_forward_handler;
mod quic_server;
mod quic_stream;
mod resolver;
mod rustls_util;
mod salt_checker;
mod shadowsocks;
mod snell_handler;
mod snell_udp_stream;
mod socket_util;
mod socks_handler;
mod tcp_client_connector;
mod tcp_handler;
mod tcp_handler_util;
mod tcp_server;
mod thread_util;
mod timed_salt_checker;
mod tls_handler;
mod trojan_handler;
mod udp_direct_message_stream;
mod util;
mod vless_handler;
mod vmess;
mod websocket;

use log::debug;
use tokio::task::JoinHandle;

use crate::address::NetLocation;
use crate::config::{BindLocation, ServerConfig, Transport};
use crate::quic_server::start_quic_server;
use crate::tcp_server::start_tcp_server;
use crate::thread_util::set_num_threads;

#[derive(Debug)]
struct ConfigChanged;

async fn start_server(config: ServerConfig) -> std::io::Result<JoinHandle<()>> {
    match config.transport {
        Transport::Tcp => start_tcp_server(config).await,
        Transport::Quic => start_quic_server(config).await,
        Transport::Udp => todo!(),
    }
}

struct ShoesService(pub ServerConfig);

#[shuttle_runtime::main]
async fn shuttle_main() -> Result<ShoesService, shuttle_runtime::Error> {
    let config_paths = ["config.shoes.yaml".to_owned()];

    let num_threads = num_cpus::get().min(4);

    set_num_threads(num_threads);

    let configs = config::load_configs(&config_paths)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    let config = configs.into_iter().next().unwrap();
    debug!("================================================================================");
    debug!("{:#?}", &config);
    debug!("================================================================================");

    Ok(ShoesService(config))
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for ShoesService {
    async fn bind(self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let config = self.0;
        let config = ServerConfig {
            bind_location: BindLocation::Address(NetLocation::from_socket_addr(addr)),
            ..config
        };
        start_server(config)
            .await
            .unwrap()
            .await
            .map_err(shuttle_runtime::CustomError::new)?;
        Ok(())
    }
}
