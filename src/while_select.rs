/// Runs a [`tokio::select!`] in a loop until one of its branches decides to
/// stop, then evaluates to the value that branch produced.
///
/// Each branch handler must evaluate to a [`ControlFlow`]:
///
/// - [`ControlFlow::Continue`] runs another iteration of the `select!`.
/// - [`ControlFlow::Break(v)`] stops the loop, and the whole `while_select!`
///   expression evaluates to `v`.
///
/// The loop is labelled internally, so a branch may also just `break <value>`
/// directly instead of returning a `ControlFlow`.
///
/// Prefixing the branches with `biased;` is forwarded to [`tokio::select!`],
/// making it poll the branches in order rather than in a random order.
///
/// [`ControlFlow`]: std::ops::ControlFlow
/// [`ControlFlow::Continue`]: std::ops::ControlFlow::Continue
/// [`ControlFlow::Break(v)`]: std::ops::ControlFlow::Break
///
/// # Examples
///
/// ```
/// use std::ops::ControlFlow;
/// use tokio::sync::mpsc;
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// let (nums_tx, mut nums_rx) = mpsc::channel::<i32>(8);
/// let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
///
/// tokio::spawn(async move {
///     for n in [1, 2, 3] {
///         nums_tx.send(n).await.unwrap();
///     }
///     /// comment out this line for the test to fail, as the `while_select!` will never break
///     stop_tx.send(()).await.unwrap();
/// });
///
/// let check = async move {
///     totils::while_select! {
///         Some(n) = nums_rx.recv() => {
///             ControlFlow::Continue(())
///         }
///         _ = stop_rx.recv() => ControlFlow::Break(()),
///     }
/// };
/// assert_eq!(tokio::time::timeout(std::time::Duration::from_secs(1), check).await, Ok(()));
/// # }
/// ```
#[macro_export]
macro_rules! while_select {
    (biased; $($tokens:tt)*) => {
        '__while_select: loop {
            match ::tokio::select! { biased; $($tokens)* } {
                ::std::ops::ControlFlow::Continue(_) => {}
                ::std::ops::ControlFlow::Break(v) => break '__while_select v,
            }
        }
    };
    ($($tokens:tt)*) => {
        '__while_select: loop {
            match ::tokio::select! { $($tokens)* } {
                ::std::ops::ControlFlow::Continue(_) => {}
                ::std::ops::ControlFlow::Break(v) => break '__while_select v,
            }
        }
    };
}

#[cfg(test)]
mod test {

    #![allow(clippy::as_conversions)]
    #![allow(clippy::unwrap_used)]

    #[tokio::test]
    async fn while_select_breaks_as_expected_on_control_flow() {
        use std::{ops::ControlFlow, time::Duration};
        use tokio::time::sleep;

        let mut fut_a = Box::pin(async { ControlFlow::Break::<&'static str>("hello") });
        let mut fut_b = Box::pin(async {
            sleep(Duration::from_secs(1)).await;
            ControlFlow::Break::<&'static str>("nein")
        });

        let res = while_select!(
            it = &mut fut_a => it,
            it = &mut fut_b => it,
        );

        assert_eq!("hello", res);
    }

    #[tokio::test]
    async fn while_select_biased_breaks_as_expected_on_control_flow() {
        use std::{ops::ControlFlow, time::Duration};
        use tokio::time::sleep;

        let mut fut_a = Box::pin(async { ControlFlow::Break::<&'static str>("hello") });
        let mut fut_b = Box::pin(async {
            sleep(Duration::from_secs(1)).await;
            ControlFlow::Break::<&'static str>("nein")
        });

        let res = while_select!(
            biased;
            it = &mut fut_a => it,
            it = &mut fut_b => it,
        );

        assert_eq!("hello", res);
    }

    #[tokio::test]
    async fn while_select_breaks_as_expected_on_break() {
        use std::{ops::ControlFlow, time::Duration};
        use tokio::time::sleep;

        let mut fut_a = Box::pin(async {});
        let mut fut_b = Box::pin(async {
            sleep(Duration::from_secs(1)).await;
            ControlFlow::Break::<&'static str>("nein")
        });

        let res = while_select!(
            _ = &mut fut_a => break "hello",
            it = &mut fut_b => it,
        );

        assert_eq!("hello", res);
    }
}
