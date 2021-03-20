use std::convert::Infallible;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use hyper::server::conn::AddrStream;
use std::env;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  pretty_env_logger::init();

  let port: u16 = env::var("PORT")?.parse()?;

  let make_svc = make_service_fn(|socket: &AddrStream| {
    let remote_address = socket.remote_addr();
    async move {
      Ok::<_, Infallible>(service_fn(move |_: Request<Body>| async move {
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