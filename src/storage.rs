
use crate::{Message, MessageState};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// The `Storage` trait defines the interface for a message queue storage implementation.
#[async_trait]
pub trait Storage: Send + Sync {
    async fn add(&self, msg: Message) -> Result<(), String>;
    async fn get(&self, count: usize) -> Result<Vec<Message>, String>;
    async fn delete(&self, ids: Vec<String>) -> Result<(), String>;
    async fn purge(&self) -> Result<(), String>;
    async fn retry(&self, ids: Vec<String>) -> Result<(), String>;
    async fn peek(&self, count: usize) -> Result<Vec<Message>, String>;
}

#[derive(Default)]
struct BaseMemoryStorage {
    queue: Vec<Message>,
    processing: HashMap<String, Message>,
}

impl BaseMemoryStorage {
    fn add(&mut self, msg: Message) -> Result<(), String> {
        self.queue.push(msg);
        Ok(())
    }

    fn get(&mut self, count: usize) -> Result<Vec<Message>, String> {
        let count = count.min(self.queue.len());
        let mut messages: Vec<Message> = self.queue.drain(0..count).collect();
        for message in &mut messages {
            message.state = MessageState::Processing;
            self.processing
                .insert(message.id.to_string(), message.clone());
        }
        Ok(messages)
    }

    fn delete(&mut self, ids: Vec<String>) -> Result<(), String> {
        for id in ids {
            self.processing.remove(&id);
        }
        Ok(())
    }

    fn purge(&mut self) -> Result<(), String> {
        self.queue.clear();
        self.processing.clear();
        Ok(())
    }

    fn retry(&mut self, ids: Vec<String>) -> Result<(), String> {
        let mut retried_messages = Vec::new();
        let ids_set: std::collections::HashSet<String> = ids.into_iter().collect();

        self.processing.retain(|id, message| {
            if ids_set.contains(id) {
                message.retry_count += 1;
                message.state = MessageState::Ready;
                retried_messages.push(message.clone());
                false
            } else {
                true
            }
        });

        self.queue.extend(retried_messages);
        Ok(())
    }

    fn peek(&mut self, count: usize) -> Result<Vec<Message>, String> {
        let count = count.min(self.queue.len());
        Ok(self.queue.iter().take(count).cloned().collect())
    }
}

#[derive(Default)]
pub struct MemoryStorage {
    inner: Arc<Mutex<BaseMemoryStorage>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn add(&self, msg: Message) -> Result<(), String> {
        self.inner.lock().await.add(msg)
    }

    async fn get(&self, count: usize) -> Result<Vec<Message>, String> {
        self.inner.lock().await.get(count)
    }

    async fn delete(&self, ids: Vec<String>) -> Result<(), String> {
        self.inner.lock().await.delete(ids)
    }

    async fn purge(&self) -> Result<(), String> {
        self.inner.lock().await.purge()
    }

    async fn retry(&self, ids: Vec<String>) -> Result<(), String> {
        self.inner.lock().await.retry(ids)
    }

    async fn peek(&self, count: usize) -> Result<Vec<Message>, String> {
        self.inner.lock().await.peek(count)
    }
}
