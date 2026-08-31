/// Generates an event enum and a `multiplex` function that fan multiple
/// typed input channels into a single channel carrying that enum.
///
/// ```
/// # #[derive(Debug, PartialEq)] pub struct A;
/// # #[derive(Debug, PartialEq)] pub struct B;
/// # #[derive(Debug, PartialEq)] pub struct C;
/// totils::mpsc_multiplexer! {
///     #[derive(Debug, PartialEq)]
///     pub enum Event {
///         A(A),
///         B(B),
///         C(C),
///     }
///
///     pub fn multiplex;
/// }
/// ```
///
/// expands to an `Event` enum with one variant per listed type, and a
/// `multiplex(channel_size: usize)` function returning a `Sender<T>` for
/// each type followed by a single `Receiver<Event>` that receives whatever
/// is sent on any of the senders, wrapped in the matching variant.
#[macro_export]
macro_rules! mpsc_multiplexer {
    (
        $(#[$enum_meta:meta])*
        $enum_vis:vis enum $enum_name:ident {
            $( $variant:ident($ty:ty) ),+ $(,)?
        }

        $(#[$fn_meta:meta])*
        $fn_vis:vis fn $fn_name:ident;
    ) => {
        $(#[$enum_meta])*
        $enum_vis enum $enum_name {
            $( $variant($ty) ),+
        }

        $(#[$fn_meta])*
        $fn_vis fn $fn_name(
            channel_size: usize,
        ) -> (
            $( ::tokio::sync::mpsc::Sender<$ty>, )+
            ::tokio::sync::mpsc::Receiver<$enum_name>,
        ) {
            $crate::paste::paste! {
                $(
                    let ([<$variant:snake _tx>], mut [<$variant:snake _rx>]) =
                        ::tokio::sync::mpsc::channel::<$ty>(channel_size);
                )+
                let (event_tx, event_rx) = ::tokio::sync::mpsc::channel(channel_size);

                ::tokio::spawn(async move {
                    loop {
                        ::tokio::select! {
                            $(
                                recv = [<$variant:snake _rx>].recv() => {
                                    match recv {
                                        Some(value) => {
                                            if let Err(e) = event_tx.send($enum_name::$variant(value)).await {
                                                ::tracing::debug!("Failed to send event: {}", e);
                                                break;
                                            }
                                        }
                                        None => break,
                                    }
                                }
                            )+
                        }
                    }
                });

                ( $( [<$variant:snake _tx>], )+ event_rx )
            }
        }
    };
}

#[cfg(test)]
mod test {

    #[tokio::test]
    async fn test_multiplex_five_types() {
        #[derive(Debug, PartialEq)]
        pub struct A;
        #[derive(Debug, PartialEq)]
        pub struct B;
        #[derive(Debug, PartialEq)]
        pub struct C;
        #[derive(Debug, PartialEq)]
        pub struct D;
        #[derive(Debug, PartialEq)]
        pub struct E;

        mpsc_multiplexer! {
            #[derive(Debug, PartialEq)]
            enum WideEvent {
                A(A),
                B(B),
                C(C),
                D(D),
                E(E),
            }

            fn multiplex;
        }

        let (tx_a, tx_b, tx_c, tx_d, tx_e, mut rx_event) = multiplex(10);

        tx_a.send(A).await.unwrap();
        tx_b.send(B).await.unwrap();
        tx_c.send(C).await.unwrap();
        tx_d.send(D).await.unwrap();
        tx_e.send(E).await.unwrap();

        let mut events = Vec::new();
        for _ in 0..5 {
            events.push(rx_event.recv().await.unwrap());
        }

        assert!(events.contains(&WideEvent::A(A)));
        assert!(events.contains(&WideEvent::B(B)));
        assert!(events.contains(&WideEvent::C(C)));
        assert!(events.contains(&WideEvent::D(D)));
        assert!(events.contains(&WideEvent::E(E)));
    }
}
