mod http;

use crate::http::{Response, Status};
use anyhow::anyhow;
use http::{Method, Request};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt};
use tokio::{
	io::{AsyncBufRead, BufStream},
	net::TcpListener,
};
use tracing::info;

async fn parse_request(mut stream: impl AsyncBufRead + Unpin) -> anyhow::Result<Request> {
	let mut buffer = String::new();
	stream.read_line(&mut buffer).await?;
	let mut parts = buffer.split_whitespace();

	let method = parts
		.next()
		.ok_or(anyhow!("Missing method"))
		.and_then(Method::try_from)?;

	let path = parts
		.next()
		.ok_or(anyhow!("Missing path"))
		.map(&str::to_string)?;

	let mut headers = HashMap::new();
	loop {
		buffer.clear();
		stream.read_line(&mut buffer).await?;

		if buffer.is_empty() || buffer == "\n" || buffer == "\r\n" {
			break;
		}

		let mut comps = buffer.split(":");
		let key = comps.next().ok_or(anyhow::anyhow!("missing header name"))?;
		let value = comps
			.next()
			.ok_or(anyhow::anyhow!("missing header value"))?
			.trim();

		headers.insert(key.to_string(), value.to_string());
	}

	buffer.clear();
	stream.read_to_string(&mut buffer).await?;

	Ok(Request {
		method,
		path,
		headers,
		body: buffer,
	})
}

static DEFAULT_PORT: &str = "2345";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt().init();

	let port: u16 = std::env::args()
		.nth(1)
		.unwrap_or_else(|| DEFAULT_PORT.to_string())
		.parse()?;

	let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;

	info!("Listening on {}", listener.local_addr()?);

	loop {
		let (stream, addr) = listener.accept().await?;
		let mut stream = BufStream::new(stream);

		tokio::spawn(async move {
			info!(?addr, "new connection");

			match parse_request(&mut stream).await {
				Ok(req) => {
					info!(?req, "incoming request");
					let resp = Response::from_json(Status::Ok, serde_json::to_value(req)?);
					resp.write(&mut stream).await?;
				}
				Err(err) => info!(?err, "Failed to parse request"),
			}

			anyhow::Ok(())
		});
	}
}
