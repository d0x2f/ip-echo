use hyper::header::HeaderValue;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Request, Response, Server};
use std::convert::Infallible;
use std::env;

// Headers to check in order of priority
const HEADERS: &[&str] = &[
  "x-forwarded-for",
  "x-client-ip",
  "x-real-ip",
  "cf-connecting-ip",
  "fastly-client-ip",
  "true-client-ip",
  "x-cluster-client-ip",
];

fn check_for_headers(headers: &HeaderMap<HeaderValue>) -> Result<String, ()> {
  for header in HEADERS {
    if let Some(client_ip) = headers.get(*header) {
      if let Ok(client_ip_str) = client_ip.to_str() {
        return Ok(client_ip_str.into());
      }
    }
  }
  Err(())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  pretty_env_logger::init();

  let port: u16 = match env::var("PORT") {
    Ok(port_string) => port_string
      .parse()
      .expect("Unable to parse PORT environment variable"),
    Err(_) => 8080,
  };

  let make_svc = make_service_fn(|socket: &AddrStream| {
    let remote_address = socket.remote_addr();
    async move {
      Ok::<_, Infallible>(service_fn(move |request: Request<Body>| async move {
        Ok::<_, Infallible>(match check_for_headers(request.headers()) {
          Ok(client_ip) => Response::new(Body::from(client_ip)),
          _ => Response::new(Body::from(remote_address.ip().to_string())),
        })
      }))
    }
  });

  let addr = ([0, 0, 0, 0], port).into();

  let server = Server::bind(&addr).serve(make_svc);

  println!("Listening on http://{}", addr);

  server.await?;

  Ok(())
}
