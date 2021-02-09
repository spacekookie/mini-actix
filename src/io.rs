use crate::Worker;
use std::sync::Arc;
use tokio::sync::mpsc;

// What actix does here that I don't
//
// - Actix implements their own channel concept
//   - Keep Mutex of worker reference
//   - Keep Mutex of task handles (receiver, sender)
//   - Uses unsafe queue under the hood
// - Worker/ Task handles are presumably used to wake waiting tasks
// - Implement Stream for Receiver, then use a pinned Poll

pub type Sender<W: Worker> = mpsc::Sender<W>;
pub type Receiver<W: Worker> = mpsc::Receiver<W>;
pub use mpsc::channel;
