use anyhow::anyhow;
use itertools::Itertools;
use mime::Mime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::Cursor;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) enum Method {
	Get,
	Post,
}

impl TryFrom<&str> for Method {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		match value {
			"GET" => Ok(Self::Get),
			"POST" => Ok(Self::Post),
			_ => Err(anyhow!("Invalid HTTP method: {value}")),
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Request {
	pub(crate) method: Method,
	pub(crate) path: String,
	pub(crate) headers: HashMap<String, String>,
	pub(crate) body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Response<S: AsyncRead + Unpin> {
	pub(crate) status: Status,
	pub(crate) headers: HashMap<String, String>,
	pub(crate) body: S,
}

impl<S: AsyncRead + Unpin> Response<S> {
	pub(crate) fn status_and_headers(&self) -> String {
		let headers = self
			.headers
			.iter()
			.map(|(k, v)| format!("{k}: {v}"))
			.collect_vec()
			.join("\r\n");
		format!("HTTP/1.1 {}\r\n{headers}\r\n\r\n", self.status)
	}

	pub(crate) async fn write<O: AsyncWrite + Unpin>(mut self, stream: &mut O) -> anyhow::Result<()> {
		stream
			.write_all(self.status_and_headers().as_bytes())
			.await?;
		tokio::io::copy(&mut self.body, stream).await?;
		Ok(())
	}
}

impl Response<Cursor<Vec<u8>>> {
	pub fn from_string(status: Status, mime: Mime, data: String) -> Self {
		let bytes = data.to_string().into_bytes();

		let headers = HashMap::from([
			("Content-Type".to_string(), mime.to_string()),
			("Content-Length".to_string(), bytes.len().to_string()),
		]);

		Self {
			status,
			headers,
			body: Cursor::new(bytes),
		}
	}

	pub fn from_json(status: Status, data: Value) -> Self {
		Self::from_string(status, mime::APPLICATION_JSON, data.to_string())
	}
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub(crate) enum Status {
	Ok,
	NotFound,
}

impl Display for Status {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Status::Ok => "200 OK",
				Status::NotFound => "404 Not Found",
			}
		)
	}
}

impl TryFrom<u16> for Status {
	type Error = anyhow::Error;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			200 => Ok(Self::Ok),
			404 => Ok(Self::NotFound),
			_ => Err(anyhow!("Invalid status code: {value}")),
		}
	}
}
