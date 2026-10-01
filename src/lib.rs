#[cfg(feature = "mpsc_multiplexer")]
#[doc(hidden)]
pub use paste;
use tokio::select;

#[cfg(feature = "mpsc_multiplexer")]
mod mpsc_multiplexer;
#[cfg(feature = "trace_collector")]
pub mod trace_collector;
#[cfg(feature = "while_select")]
pub mod while_select;

pub trait CancelOn {
    type Output;
    fn or_cancel_on(self, cancel: impl Future) -> impl Future<Output = Option<Self::Output>>;
}

impl<F: Future> CancelOn for F {
    type Output = F::Output;

    async fn or_cancel_on(self, cancel: impl Future) -> Option<Self::Output> {
        select! {
            biased;
            _ = cancel => None,
            output = self => Some(output),
        }
    }
}

pub trait ReceiveOrCancelOn {
    type Output;
    fn recv_or_cancel_on(
        &mut self,
        cancel: impl Future,
    ) -> impl Future<Output = Option<Self::Output>>;
}

impl<T> ReceiveOrCancelOn for tokio::sync::mpsc::Receiver<T> {
    type Output = T;

    async fn recv_or_cancel_on(&mut self, cancel: impl Future) -> Option<Self::Output> {
        self.recv().or_cancel_on(cancel).await.flatten()
    }
}

impl<T> ReceiveOrCancelOn for tokio::sync::oneshot::Receiver<T> {
    type Output = T;

    async fn recv_or_cancel_on(&mut self, cancel: impl Future) -> Option<Self::Output> {
        self.or_cancel_on(cancel).await.and_then(|it| it.ok())
    }
}
