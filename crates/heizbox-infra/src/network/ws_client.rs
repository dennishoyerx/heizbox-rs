use heizbox_core::error::NetworkError;
use heizbox_core::event::DomainEvent;
use super::reconnect::ExponentialBackoff;

// ── Placeholder WebSocket handle ──────────────────────────────────────────────

/// Wraps the underlying WebSocket connection.
/// In production this would wrap `esp_idf_svc::ws::client::EspWebSocketClient`.
struct WsHandle;

impl WsHandle {
    async fn connect(_url: &str) -> Result<Self, NetworkError> {
        Ok(Self)
    }

    async fn send_text(&mut self, _text: &str) -> Result<(), NetworkError> {
        Ok(())
    }

    async fn recv(&mut self) -> Result<Vec<u8>, NetworkError> {
        Ok(Vec::new())
    }
}

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
enum WsState {
    Disconnected,
    Connecting,
    Connected,
}

// ── Client ────────────────────────────────────────────────────────────────────

pub struct WebSocketClient {
    url: String,
    socket: Option<WsHandle>,
    state: WsState,
    backoff: ExponentialBackoff,
}

impl WebSocketClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            socket: None,
            state: WsState::Disconnected,
            backoff: ExponentialBackoff::new(),
        }
    }

    pub async fn ensure_connected(&mut self) -> Result<(), NetworkError> {
        if self.state == WsState::Connected {
            return Ok(());
        }
        if self.state == WsState::Connecting {
            return Err(NetworkError::AlreadyConnecting);
        }

        const MAX_RETRIES: u32 = 10;
        self.state = WsState::Connecting;

        for attempt in 1..=MAX_RETRIES {
            match WsHandle::connect(&self.url).await {
                Ok(ws) => {
                    self.socket = Some(ws);
                    self.state = WsState::Connected;
                    self.backoff.reset();
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("WS connect attempt {attempt}/{MAX_RETRIES} failed: {e}");
                    if attempt == MAX_RETRIES {
                        self.state = WsState::Disconnected;
                        return Err(NetworkError::ReconnectFailed);
                    }
                    let _delay = self.backoff.next_delay_ms();
                    // In production: FreeRTOS vTaskDelay(_delay / portTICK_PERIOD_MS)
                }
            }
        }
        unreachable!()
    }

    /// Serialise and send a `DomainEvent` over the WebSocket.
    pub async fn send_event(&mut self, event: &DomainEvent) -> Result<(), NetworkError> {
        self.ensure_connected().await?;

        let json =
            serde_json::to_string(event).map_err(|_| NetworkError::SerializationError)?;

        self.socket
            .as_mut()
            .ok_or(NetworkError::NotConnected)?
            .send_text(&json)
            .await
    }

    /// Receive raw bytes from the WebSocket.
    pub async fn recv(&mut self) -> Result<Vec<u8>, NetworkError> {
        if self.state != WsState::Connected {
            return Err(NetworkError::NotConnected);
        }
        self.socket
            .as_mut()
            .ok_or(NetworkError::NotConnected)?
            .recv()
            .await
    }

    pub fn is_connected(&self) -> bool {
        self.state == WsState::Connected
    }

    pub fn disconnect(&mut self) {
        self.socket = None;
        self.state = WsState::Disconnected;
    }
}
