//! Action bus participant API (DEALER client / DEALER worker).

mod client;
mod worker;

pub use client::{ActionClient, ActionGoalIter, ActionKind, ActionMessage};
pub use worker::{ActionGoalHandler, ActionWorker};
