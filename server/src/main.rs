mod http;

use std::{collections::HashMap, io};

use anyhow::anyhow;
use http::{Method, Request};
use tokio::{
	io::{AsyncBufRead, BufStream},
	net::{TcpListener, TcpStream},
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

	loop {
		line_buffer.clear();
		stream.read_line(&mut line_buffer).await?;

		if line_buffer.is_empty() || line_buffer == "\n" || line_buffer == "\r\n" {
			break;
		}

		let mut comps = line_buffer.split(":");
		let key = comps.next().ok_or(anyhow::anyhow!("missing header name"))?;
		let value = comps
			.next()
			.ok_or(anyhow::anyhow!("missing header value"))?
			.trim();

		headers.insert(key.to_string(), value.to_string());
	}

	line_buffer.clear();

	Ok(Request {
		method,
		path,
		headers,
		body,
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
				Ok(req) => info!(?req, "incoming request"),
				Err(err) => info!(?err, "Failed to parse request"),
			}
		})
	}

	Ok(())
}
