use std::collections::HashMap;

use anyhow::anyhow;
use reqwest::StatusCode;
use tokio::io::AsyncRead;

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

pub(crate) struct Request {
	pub(crate) method: Method,
	pub(crate) path: String,
	pub(crate) headers: HashMap<String, String>,
	pub(crate) body: String,
}

pub(crate) struct Response<S: AsyncRead + Unpin> {
	pub(crate) status: Status,
	pub(crate) headers: HashMap<String, String>,
	pub(crate) body: S,
}

pub(crate) enum Status {
	Ok,
	NotFound,
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
