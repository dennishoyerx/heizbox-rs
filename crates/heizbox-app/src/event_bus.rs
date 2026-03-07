use heapless::spsc::Queue;
use heizbox_core::event::DomainEvent;

/// Event bus for inter-task communication.
/// Uses a lock-free SPSC queue with a fixed capacity.
pub struct EventBus {
    queue: Queue<DomainEvent, 16>,
}

impl EventBus {
    /// Create a new empty event bus.
    pub fn new() -> Self {
        Self {
            queue: Queue::new(),
        }
    }

    /// Publish an event to the bus.
    /// If the queue is full, the event is dropped.
    pub fn publish(&mut self, event: DomainEvent) {
        let _ = self.queue.enqueue(event);
    }

    /// Try to pop the next event from the bus.
    /// Returns `None` if the queue is empty.
    pub fn try_pop(&mut self) -> Option<DomainEvent> {
        self.queue.dequeue()
    }

    /// Check if the bus is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}