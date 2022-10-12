mod request;
mod response;

use std::collections::{HashMap, HashSet};
// because the compile error, I change the clap's version to 4.0.12
use clap::Parser;
use rand::{Rng, SeedableRng};
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{self, Duration};

/// Contains information parsed from the command-line invocation of balancebeam. The Clap macros
/// provide a fancy way to automatically construct a command-line argument parser.
#[derive(Parser, Debug)]
#[command(about = "Fun with load balancing")]
struct CmdOptions {
    // in clap 4.0.12, we use #[arg(...)] rather than #[clap(...)]
    /// IP/port to bind to
    #[arg(short, long, default_value = "0.0.0.0:1100")]
    bind: String,

    /// Upstream host to forward requests to
    #[arg(short, long)]
    upstream: Vec<String>,

    /// Perform active health checks on this interval (in seconds)
    #[arg(long, default_value = "10")]
    active_health_check_interval: usize,

    /// Path to send request to for active health checks
    #[arg(long, default_value = "/")]
    active_health_check_path: String,

    /// Maximum number of requests to accept per IP per minute (0 = unlimited)
    #[arg(long, default_value = "0")]
    max_requests_per_minute: usize,
}

/// Contains information about the state of balancebeam (e.g. what servers we are currently proxying
/// to, what servers have failed, rate limiting counts, etc.)
///
/// You should add fields to this struct in later milestones.
struct ProxyState {
    /// How frequently we check whether upstream servers are alive (Milestone 4)
    active_health_check_interval: usize,

    /// Where we should send requests when doing active health checks (Milestone 4)
    active_health_check_path: String,

    /// Maximum number of requests an individual IP can make in a minute (Milestone 5)
    max_requests_per_minute: usize,

    /// Addresses of servers that we are proxying to
    upstream_addresses: Vec<String>,

    // we only use RwLock on this field rather than the whole struct
    // keep lock as small as possible
    /// Addresses of servers that are dead
    dead_upstream: RwLock<HashSet<String>>,

    /// Track the { IP: requests count } relations
    requests_cnt: RwLock<HashMap<String, usize>>,
}

#[tokio::main]
async fn main() {
    // Initialize the logging library. You can print log messages using the `log` macros:
    // https://docs.rs/log/0.4.8/log/ You are welcome to continue using print! statements; this
    // just looks a little prettier.
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::init();

    // Parse the command line arguments passed to this program
    let options = CmdOptions::parse();
    if options.upstream.len() < 1 {
        log::error!("At least one upstream server must be specified using the --upstream option.");
        std::process::exit(1);
    }

    // Start listening for connections
    let mut listener = match TcpListener::bind(&options.bind).await {
        Ok(listener) => listener,
        Err(err) => {
            log::error!("Could not bind to {}: {}", options.bind, err);
            std::process::exit(1);
        }
    };
    log::info!("Listening for requests on {}", options.bind);

    // Handle incoming connections
    let state = ProxyState {
        upstream_addresses: options.upstream,
        active_health_check_interval: options.active_health_check_interval,
        active_health_check_path: options.active_health_check_path,
        max_requests_per_minute: options.max_requests_per_minute,
        dead_upstream: RwLock::new(HashSet::new()),
        requests_cnt: RwLock::new(HashMap::new()),
    };

    // Milestone 4
    let arc_state = Arc::new(state);
    let health_check_state = arc_state.clone();
    // spawn a new thread to do health_check
    tokio::spawn(async move {
        health_check(&health_check_state).await;
    });

    // Milestone 5
    let rate_limit_state = arc_state.clone();
    // If it is zero, rate limiting should be disabled.
    if rate_limit_state.max_requests_per_minute != 0 {
        let mut rate_interval = time::interval(Duration::from_secs(60));
        rate_interval.tick().await; // similar to what we did in the Milestone 4
        tokio::spawn(async move {
            rate_interval.tick().await;
            loop {
                // every 60 seconds, we reset the counter, i.e. state.
                rate_limit_state.requests_cnt.write().await.clear();
                log::debug!("[---------] Reset the requests_cnt HashMap")
            }
        });
    }

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                // Handle the connection!
                let current_state = arc_state.clone();
                // Spawns a new asynchronous task, returning a JoinHandle for it.
                tokio::spawn(async move {
                    handle_connection(stream, &current_state).await;
                });
            }
            Err(_) => return,
        }
    }
}

// Milestone 4
async fn health_check(state: &Arc<ProxyState>) {
    let mut interval = time::interval(Duration::from_secs(
        state.active_health_check_interval as u64,
    ));
    // Note: according to the docs, the 1st tick will hit immediately
    interval.tick().await;
    loop {
        interval.tick().await;

        // run health check
        for upstream_ip in state.upstream_addresses.iter().as_ref() {
            let request = http::Request::builder()
                .method(http::Method::GET)
                .uri(&state.active_health_check_path)
                .header("Host", upstream_ip)
                .body(Vec::new())
                .unwrap();

            // If a failed upstream returns HTTP 200, put it back in the rotation of upstream servers.
            // If an online upstream returns a non-200 status code, mark that server as failed.
            let mut tcp_stream = TcpStream::connect(upstream_ip)
                .await
                .expect("Error when try to form a TcpStream");
            request::write_to_stream(&request, &mut tcp_stream)
                .await
                .expect("Failed to send request to upstream");

            match response::read_from_stream(&mut tcp_stream, &http::Method::GET).await {
                Ok(response) => {
                    if response.status() == http::StatusCode::OK {
                        if state.dead_upstream.write().await.remove(upstream_ip) {
                            log::debug!("[---------] Live {}", upstream_ip);
                        }
                    } else {
                        {
                            state
                                .dead_upstream
                                .write()
                                .await
                                .insert(upstream_ip.to_string());
                        }
                        log::debug!("[---------] Fail {}", upstream_ip);
                        log::debug!(
                            "[---------] Fail list {:?}",
                            state.dead_upstream.read().await
                        );
                    }
                    // the lock drop here
                }
                Err(error) => {
                    log::error!("Error reading response from server: {:?}", error);
                }
            };
        }
    }
}

async fn connect_to_upstream(state: &Arc<ProxyState>) -> Result<TcpStream, std::io::Error> {
    loop {
        // just in case that the state.upstream_addresses are empty at first
        if state.upstream_addresses.len() == state.dead_upstream.read().await.len() {
            log::error!("No upstream available");
            // errors can be created from strings
            return Err(Error::new(ErrorKind::Other, "No upstream available"));
        }

        log::debug!(
            "[---------] All upstreams ips: {:?}",
            state.upstream_addresses
        );

        // in every loop, we only check the available upstream ips
        let mut upstream_available = Vec::new();
        log::debug!(
            "[---------] Ready to find available upstreams, current fail list: {:?}",
            state.dead_upstream.read().await
        );
        for upstream_ip in state.upstream_addresses.iter().as_ref() {
            // Note: the server may contains same Ip address
            if state.dead_upstream.read().await.get(upstream_ip).is_none() {
                upstream_available.push(upstream_ip.to_string());
            }
        }
        log::debug!("[---------] Current available: {:?}", upstream_available);

        let mut rng = rand::rngs::StdRng::from_entropy();
        // let upstream_idx = rng.gen_range(0, state.upstream_addresses.len());
        // let upstream_ip = &state.upstream_addresses[upstream_idx];
        let upstream_idx = rng.gen_range(0, upstream_available.len());
        let upstream_ip = &upstream_available[upstream_idx];

        // TODO: implement failover (milestone 3)
        match TcpStream::connect(upstream_ip).await {
            Ok(tcp_stream) => {
                log::debug!("[---------] Use {}", upstream_ip);
                return Ok(tcp_stream);
            }
            Err(err) => {
                {
                    state
                        .dead_upstream
                        .write()
                        .await
                        .insert(upstream_ip.to_string());
                }

                log::debug!("Connect {} failed, try a new one", upstream_ip);

                // Clients should only receive an error if all upstreams are dead.
                if state.upstream_addresses.len() == state.dead_upstream.read().await.len() {
                    log::error!("All servers are down");
                    return Err(err);
                } else {
                    continue;
                }
            }
        }
    }
}

async fn send_response(client_conn: &mut TcpStream, response: &http::Response<Vec<u8>>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();
    log::info!(
        "{} <- {}",
        client_ip,
        response::format_response_line(&response)
    );
    if let Err(error) = response::write_to_stream(&response, client_conn).await {
        log::warn!("Failed to send response to client: {}", error);
        return;
    }
}

async fn handle_connection(mut client_conn: TcpStream, state: &Arc<ProxyState>) {
    let client_ip = client_conn.peer_addr().unwrap().ip().to_string();

    // Open a connection to a random destination server
    let mut upstream_conn = match connect_to_upstream(state).await {
        Ok(stream) => stream,
        Err(_error) => {
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
    };
    let upstream_ip = client_conn.peer_addr().unwrap().ip().to_string();

    // The client may now send us one or more requests. Keep trying to read requests until the
    // client hangs up or we get an error.
    loop {
        // Read a request from the client
        let mut request = match request::read_from_stream(&mut client_conn).await {
            Ok(request) => request,
            // Handle case where client closed connection and is no longer sending requests
            Err(request::Error::IncompleteRequest(0)) => {
                log::debug!("Client finished sending requests. Shutting down connection");
                return;
            }
            // Handle I/O error in reading from the client
            Err(request::Error::ConnectionError(io_err)) => {
                log::info!("Error reading request from client stream: {}", io_err);
                return;
            }
            Err(error) => {
                log::debug!("Error parsing request: {:?}", error);
                let response = response::make_http_error(match error {
                    request::Error::IncompleteRequest(_)
                    | request::Error::MalformedRequest(_)
                    | request::Error::InvalidContentLength
                    | request::Error::ContentLengthMismatch => http::StatusCode::BAD_REQUEST,
                    request::Error::RequestBodyTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
                    request::Error::ConnectionError(_) => http::StatusCode::SERVICE_UNAVAILABLE,
                });
                send_response(&mut client_conn, &response).await;
                continue;
            }
        };
        log::info!(
            "{} -> {}: {}",
            client_ip,
            upstream_ip,
            request::format_request_line(&request)
        );
        // Now, we can increase the client_ip's counter
        if state.max_requests_per_minute != 0 {
            let mut lock = state.requests_cnt.write().await;
            lock.entry(client_ip.clone())
                .and_modify(|counter| *counter += 1)
                .or_insert(1);

            if lock.get(&client_ip).unwrap().clone() > state.max_requests_per_minute {
                let response = response::make_http_error(http::StatusCode::TOO_MANY_REQUESTS);
                send_response(&mut client_conn, &response).await;
                return;
            }

            log::debug!("[---------] The current rate list stat: {:?}", lock);
        }

        // Add X-Forwarded-For header so that the upstream server knows the client's IP address.
        // (We're the ones connecting directly to the upstream server, so without this header, the
        // upstream server will only know our IP, not the client's.)
        request::extend_header_value(&mut request, "x-forwarded-for", &client_ip);

        // Forward the request to the server
        if let Err(error) = request::write_to_stream(&request, &mut upstream_conn).await {
            log::error!(
                "Failed to send request to upstream {}: {}",
                upstream_ip,
                error
            );
            let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
            send_response(&mut client_conn, &response).await;
            return;
        }
        log::debug!("Forwarded request to server");

        // Read the server's response
        let response = match response::read_from_stream(&mut upstream_conn, request.method()).await
        {
            Ok(response) => response,
            Err(error) => {
                log::error!("Error reading response from server: {:?}", error);
                let response = response::make_http_error(http::StatusCode::BAD_GATEWAY);
                send_response(&mut client_conn, &response).await;
                return;
            }
        };
        // Forward the response to the client
        send_response(&mut client_conn, &response).await;
        log::debug!("Forwarded response to client");
    }
}
