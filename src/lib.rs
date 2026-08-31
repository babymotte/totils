#[cfg(feature = "mpsc_multiplexer")]
#[doc(hidden)]
pub use paste;

#[cfg(feature = "mpsc_multiplexer")]
mod mpsc_multiplexer;
#[cfg(feature = "while_select")]
mod while_select;
