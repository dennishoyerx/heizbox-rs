use heizbox_core::error::NetworkError;

// ── Request / response types ──────────────────────────────────────────────────

pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

// ── Client ────────────────────────────────────────────────────────────────────

/// Minimal blocking HTTP client stub.
/// In production this wraps `esp_idf_svc::http::client::EspHttpConnection`.
pub struct HttpClient {
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// POST JSON body to `path`. Returns the response status and body.
    pub async fn post_json(&self, path: &str, json: &str) -> Result<HttpResponse, NetworkError> {
        let _url = format!("{}{}", self.base_url, path);
        log::info!("HTTP POST {_url}  body_len={}", json.len());
        // Real implementation uses EspHttpConnection; stub returns 200 OK.
        Ok(HttpResponse {
            status: 200,
            body: Vec::new(),
        })
    }

    /// GET request. Returns the response body bytes.
    pub async fn get(&self, path: &str) -> Result<HttpResponse, NetworkError> {
        let _url = format!("{}{}", self.base_url, path);
        log::info!("HTTP GET {_url}");
        Ok(HttpResponse {
            status: 200,
            body: Vec::new(),
        })
    }
}
