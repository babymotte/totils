#[cfg(feature = "mpsc_multiplexer")]
#[doc(hidden)]
pub use paste;
use tokio::select;

#[cfg(feature = "mpsc_multiplexer")]
mod mpsc_multiplexer;
#[cfg(feature = "trace_collector")]
pub mod trace_collector;
#[cfg(feature = "while_select")]
mod while_select;

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
