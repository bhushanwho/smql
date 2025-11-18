use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, OnceLock};
use tracing::Level;
use uuid::Uuid;

pub mod api;
pub mod storage;

// CONFIG
const DEFAULT_PORT: u16 = 1337;
const DEFAULT_MAX_MESSAGE_SIZE: usize = 65536; // 64KB
const DEFAULT_LOG_LEVEL: &str = "info";

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub max_message_size: usize,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
            log_level: DEFAULT_LOG_LEVEL.to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let mut config = Config::default();

        if let Ok(port_str) = env::var("SMQL_PORT") {
            config.port = port_str.parse().unwrap_or(config.port);
        }

        if let Ok(size_str) = env::var("SMQL_MAX_MESSAGE_SIZE") {
            config.max_message_size = Self::parse_size(&size_str).unwrap_or(config.max_message_size);
        }

        if let Ok(log_level) = env::var("SMQL_LOG_LEVEL") {
            config.log_level = log_level;
        }

        config
    }

    fn parse_size(value: &str) -> Option<usize> {
        if value.is_empty() {
            return None;
        }

        if let Some(kb_str) = value.strip_suffix(['K', 'k']) {
            kb_str
                .parse::<usize>()
                .ok()
                .filter(|&kb| kb > 0)
                .map(|kb| kb * 1024)
        } else {
            value.parse::<usize>().ok().filter(|&bytes| bytes > 0)
        }
    }

    pub fn tracing_level(&self) -> Level {
        match self.log_level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" | "warning" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Returns a reference to the global `Config` instance.
pub fn config() -> &'static Config {
    CONFIG.get_or_init(Config::from_env)
}

// TYPES
/// Represents the state of a message in the queue.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MessageState {
    /// The message is ready to be processed.
    Ready,
    /// The message is currently being processed.
    Processing,
    /// The message has been processed and is done.
    Done,
}

/// Represents a message in the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub body: String,
    pub state: MessageState,
    pub lock_until: Option<i64>,
    pub retry_count: i32,
}

impl Message {
    pub fn new(body: String) -> Message {
        Message {
            id: Uuid::now_v7(),
            body,
            state: MessageState::Ready,
            lock_until: None,
            retry_count: 0,
        }
    }
}

// SERVICES
/// The `MessageService` provides the business logic for interacting with the message queue.
#[derive(Clone)]
pub struct MessageService {
    store: Arc<dyn storage::Storage>,
}

/// Represents the possible errors that can occur in the `MessageService`.
#[derive(Debug)]
pub enum Error {
    /// The message body is larger than the configured maximum size.
    BodyTooLarge,
    /// No message IDs were provided for an operation that requires them.
    NoIds,
    /// An invalid message ID was provided.
    InvalidId(String),
    /// An error occurred in the storage layer.
    Store(String),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Store(s)
    }
}

impl MessageService {
    /// Creates a new `MessageService` with the given storage implementation.
    pub fn new(store: Arc<dyn storage::Storage>) -> MessageService {
        Self { store }
    }
}

impl MessageService {
    pub async fn add(&self, body: String) -> Result<Message, Error> {
        if body.len() > config().max_message_size {
            return Err(Error::BodyTooLarge);
        }

        let msg = Message::new(body);
        self.store.add(msg.clone()).await?;
        Ok(msg)
    }

    pub async fn get(&self, count: usize) -> Result<Vec<Message>, Error> {
        Ok(self.store.get(count).await?)
    }

    pub async fn delete(&self, ids: Vec<String>) -> Result<(), Error> {
        Self::validate_ids(&ids)?;
        Ok(self.store.delete(ids).await?)
    }

    pub async fn purge(&self) -> Result<(), Error> {
        Ok(self.store.purge().await?)
    }

    pub async fn retry(&self, ids: Vec<String>) -> Result<(), Error> {
        Self::validate_ids(&ids)?;
        Ok(self.store.retry(ids).await?)
    }

    pub async fn peek(&self, count: usize) -> Result<Vec<Message>, Error> {
        Ok(self.store.peek(count).await?)
    }

    fn validate_ids(ids: &[String]) -> Result<(), Error> {
        if ids.is_empty() {
            return Err(Error::NoIds);
        }

        for id in ids {
            if Uuid::parse_str(id).is_err() {
                return Err(Error::InvalidId(id.clone()));
            }
        }

        Ok(())
    }

    
}
