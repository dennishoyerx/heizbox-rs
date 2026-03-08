//! APP-T11: EventBus backed by a `heapless::spsc::Queue` for lock-free
//! single-producer / single-consumer communication between FreeRTOS tasks.
//!
//! For multi-consumer use, wrap in a Mutex or switch to a broadcast channel.

use heapless::spsc::Queue;
use heizbox_core::event::DomainEvent;

/// Capacity of the event queue (number of events that can be buffered).
const QUEUE_CAPACITY: usize = 16;

/// Single-producer / single-consumer event bus.
///
/// In production each FreeRTOS task holds either the producer or consumer half.
/// Here both halves are owned by the same struct for simplicity; split them with
/// `Queue::split()` when spawning tasks.
pub struct EventBus {
    queue: Queue<DomainEvent, QUEUE_CAPACITY>,
}

impl EventBus {
    pub fn new() -> Self {
        Self { queue: Queue::new() }
    }

    /// Enqueue an event.  Silently drops if the queue is full.
    pub fn publish(&mut self, event: DomainEvent) {
        let _ = self.queue.enqueue(event);
    }

    /// Try to dequeue the next event.  Returns `None` when empty.
    pub fn try_pop(&mut self) -> Option<DomainEvent> {
        self.queue.dequeue()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
