use std::convert::Infallible;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, HeaderMap};
use hyper::header::HeaderValue;
use hyper::server::conn::AddrStream;
use std::env;

fn check_for_header(headers: &HeaderMap<HeaderValue>, header_key: &str) -> Result<String, ()> {
  if let Some(client_ip) = headers.get(header_key) {
    if let Ok(client_ip_str) = client_ip.to_str() {
      return Ok(client_ip_str.into());
    }
  }
  Err(())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  pretty_env_logger::init();

  let port: u16 = env::var("PORT")?.parse()?;

  let make_svc = make_service_fn(|socket: &AddrStream| {
    let remote_address = socket.remote_addr();
    async move {
      Ok::<_, Infallible>(service_fn(move |request: Request<Body>| async move {
        let headers = request.headers();
        if let Ok(client_ip) = check_for_header(headers, "x-forwarded-for") {
          return Ok::<_, Infallible>(
            Response::new(Body::from(client_ip))
          );
        }
        Ok::<_, Infallible>(
          Response::new(Body::from(remote_address.ip().to_string()))
        )
      }))
    }
  });

  let addr = ([0, 0, 0, 0], port).into();

  let server = Server::bind(&addr).serve(make_svc);

  println!("Listening on http://{}", addr);

  server.await?;

  Ok(())
}