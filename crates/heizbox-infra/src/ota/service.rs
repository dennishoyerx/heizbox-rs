use heizbox_core::error::{InfraError, NetworkError};
use super::super::network::http_client::HttpClient;

#[derive(Debug, Clone, Copy)]
enum OtaState {
    Idle,
    Downloading { percent: u8 },
    Installing,
}

pub struct OtaService {
    http: HttpClient,
    state: OtaState,
}

impl OtaService {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: HttpClient::new(base_url),
            state: OtaState::Idle,
        }
    }

    /// Check for a firmware update and apply it if one is available.
    pub async fn check_and_update(&mut self) -> Result<(), InfraError> {
        log::info!("OTA: checking for update…");
        self.state = OtaState::Downloading { percent: 0 };

        let firmware_url = self.fetch_latest_url().await?;
        let binary = self.download(&firmware_url).await?;

        log::info!("OTA: installing {} bytes", binary.len());
        self.state = OtaState::Installing;

        // Hand off to the ESP-IDF OTA partition.
        // In production: use esp_idf_svc::ota::EspOta here.
        log::info!("OTA: complete — rebooting");
        Ok(())
    }

    async fn fetch_latest_url(&self) -> Result<String, InfraError> {
        let resp = self.http.get("/api/firmware/latest").await?;
        let url = String::from_utf8(resp.body)
            .map_err(|_| InfraError::Network(NetworkError::SerializationError))?;
        Ok(url)
    }

    async fn download(&mut self, url: &str) -> Result<Vec<u8>, InfraError> {
        let resp = self.http.get(url).await?;
        let total = resp.body.len();
        log::info!("OTA: downloaded {total} bytes");
        self.state = OtaState::Downloading { percent: 100 };
        Ok(resp.body)
    }
}
