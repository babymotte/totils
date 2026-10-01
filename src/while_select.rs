/*
 *  Copyright 2026 Michael Bachmann
 *
 * Licensed under either the MIT or the Apache License, Version 2.0,
 * as per the user's preference.
 * You may not use this file except in compliance with at least one
 * of these two licenses.
 * You may obtain a copy of the Licenses at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *     and
 *     https://opensource.org/license/MIT
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::ops::ControlFlow;

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
        $crate::while_select!(@munch [biased;] $($tokens)*)
    };

    // all branches consumed: emit the loop
    (@munch [$($acc:tt)*]) => {
        '__while_select: loop {
            match ::tokio::select! { $($acc)* } {
                ::std::ops::ControlFlow::Continue(_) => {}
                ::std::ops::ControlFlow::Break(v) => break '__while_select v,
            }
        }
    };

    // `else => handler`
    (@munch [$($acc:tt)*] else => $h:block $(,)?) => {
        $crate::while_select!(@munch [$($acc)* else => { $crate::__while_select_branch!($h) },])
    };
    (@munch [$($acc:tt)*] else => $h:expr $(,)?) => {
        $crate::while_select!(@munch [$($acc)* else => { $crate::__while_select_branch!($h) },])
    };

    // `pattern = future, if precondition => handler`
    (@munch [$($acc:tt)*] $p:pat = $f:expr, if $c:expr => $h:block, $($rest:tt)*) => {
        $crate::while_select!(@munch [$($acc)* $p = $f, if $c => { $crate::__while_select_branch!($h) },] $($rest)*)
    };
    (@munch [$($acc:tt)*] $p:pat = $f:expr, if $c:expr => $h:block $($rest:tt)*) => {
        $crate::while_select!(@munch [$($acc)* $p = $f, if $c => { $crate::__while_select_branch!($h) },] $($rest)*)
    };
    (@munch [$($acc:tt)*] $p:pat = $f:expr, if $c:expr => $h:expr $(, $($rest:tt)*)?) => {
        $crate::while_select!(@munch [$($acc)* $p = $f, if $c => { $crate::__while_select_branch!($h) },] $($($rest)*)?)
    };

    // `pattern = future => handler`
    (@munch [$($acc:tt)*] $p:pat = $f:expr => $h:block, $($rest:tt)*) => {
        $crate::while_select!(@munch [$($acc)* $p = $f => { $crate::__while_select_branch!($h) },] $($rest)*)
    };
    (@munch [$($acc:tt)*] $p:pat = $f:expr => $h:block $($rest:tt)*) => {
        $crate::while_select!(@munch [$($acc)* $p = $f => { $crate::__while_select_branch!($h) },] $($rest)*)
    };
    (@munch [$($acc:tt)*] $p:pat = $f:expr => $h:expr $(, $($rest:tt)*)?) => {
        $crate::while_select!(@munch [$($acc)* $p = $f => { $crate::__while_select_branch!($h) },] $($($rest)*)?)
    };

    ($($tokens:tt)*) => {
        $crate::while_select!(@munch [] $($tokens)*)
    };
}

/// Converts a single `while_select!` branch handler into a [`ControlFlow`].
#[doc(hidden)]
#[macro_export]
macro_rules! __while_select_branch {
    ($h:expr) => {{
        #[allow(unreachable_code)]
        $crate::while_select::IntoControlFlow::into_control_flow($h)
    }};
}

pub trait IntoControlFlow<T> {
    fn into_control_flow(self) -> ControlFlow<T>;
}

impl<T> IntoControlFlow<T> for ControlFlow<T> {
    fn into_control_flow(self) -> ControlFlow<T> {
        self
    }
}

impl<T> IntoControlFlow<T> for () {
    fn into_control_flow(self) -> ControlFlow<T> {
        ControlFlow::Continue(())
    }
}

// Handlers that diverge (e.g. `break 'label v` or `return`) have type `!`.
// The never type can't be named on stable, but it can be reached through
// the return type of a function pointer.
#[doc(hidden)]
pub trait FnOutput {
    type Output;
}

impl<T> FnOutput for fn() -> T {
    type Output = T;
}

type Never = <fn() -> ! as FnOutput>::Output;

impl<T> IntoControlFlow<T> for Never {
    fn into_control_flow(self) -> ControlFlow<T> {
        self
    }
}

impl<E> IntoControlFlow<Result<(), E>> for Result<(), E> {
    fn into_control_flow(self) -> ControlFlow<Result<(), E>> {
        match self {
            Ok(_) => ControlFlow::Continue(()),
            Err(e) => ControlFlow::Break(Err(e)),
        }
    }
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
