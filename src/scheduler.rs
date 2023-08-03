use std::sync::Arc;
use std::time::Duration;

use log::warn;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::api::SearchResult;
use crate::{DehashedApi, DehashedError, Query};

/// A search request for the [Scheduler].
#[derive(Debug)]
pub struct ScheduledRequest {
    query: Query,
    ret: oneshot::Sender<Result<SearchResult, DehashedError>>,
}

impl ScheduledRequest {
    /// Create a new request
    ///
    /// The [Scheduler] will sent the result back through the provided channel.
    /// If sending fails, the result is dropped and the scheduler continues with the next request.
    pub fn new(query: Query, ret: oneshot::Sender<Result<SearchResult, DehashedError>>) -> Self {
        Self { query, ret }
    }
}

/// The scheduler to manage with the rate limit of the unhashed api
///
/// Make sure that you just spawn one instance of the scheduler.
/// You can receive and schedule as many requests as you like on the instance.
#[derive(Clone)]
pub struct Scheduler {
    handle: Arc<JoinHandle<()>>,
    tx: Sender<ScheduledRequest>,
}

impl Scheduler {
    pub(crate) fn new(api: &DehashedApi) -> Self {
        let (tx, rx) = mpsc::channel(5);

        let mut rx: Receiver<ScheduledRequest> = rx;
        let task_api = api.clone();
        let handle = tokio::spawn(async move {
            while let Some(req) = rx.recv().await {
                let res = task_api.search(req.query).await;
                if req.ret.send(res).is_err() {
                    warn!("Couldn't send result back through channel");
                }
                sleep(Duration::from_millis(200)).await;
            }
        });
        Self {
            tx,
            handle: Arc::new(handle),
        }
    }

    /// Retrieve a [Sender] to allow pushing tasks to the scheduler.
    ///
    /// To use multiple senders, you can clone the one you've received or
    /// retrieve a new one using this method
    pub fn retrieve_sender(&self) -> Sender<ScheduledRequest> {
        self.tx.clone()
    }

    /// Stop the [Scheduler].
    ///
    /// This will abort the tokio task.
    pub fn stop_scheduler(self) {
        self.handle.abort();
    }
}
