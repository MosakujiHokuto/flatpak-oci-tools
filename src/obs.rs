use quick_xml::de::from_str;
use reqwest::blocking::Client;

mod binaries;
mod container;
pub use binaries::*;
pub use container::*;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub struct ObsApi {
    api: String,
    username: String,
    password: String,
    client: Client,
}

impl ObsApi {
    pub fn new(
        api: &str,
        username: &str,
        password: &str,
    ) -> Result<ObsApi, Box<dyn std::error::Error>> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/xml"),
        );

        let client = Client::builder()
            .user_agent(APP_USER_AGENT)
            .default_headers(headers)
            .build()?;
        Ok(ObsApi {
            api: api.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            client,
        })
    }

    fn do_get_request(&self, path: &str) -> reqwest::blocking::RequestBuilder {
        self.client
            .get(format!("{api}/{path}", api = self.api, path = path))
            .basic_auth(&self.username, Some(&self.password))
    }

    pub fn get<T>(&self, path: &str) -> Result<T, Box<dyn std::error::Error>>
    where T: serde::de::DeserializeOwned
    {
	let res = self.do_get_request(path)
	    .send()?
	    .text()?;

	let ret: T = from_str(&res)?;
	Ok(ret)
    }
}
