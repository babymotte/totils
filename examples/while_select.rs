use miette::{Context, miette};
use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};
use totils::while_select;

#[tokio::main]
async fn main() -> miette::Result<()> {
    eprintln!("starting while_select");

    while_select! {
        biased;
        _ = do_a() => {
            did_a();
            Ok(())
        },
        _ = do_b() => did_b(),
        it = do_c() => did_c(it),
    }
    .wrap_err("something went wrong")
}

async fn do_a() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

fn did_a() {
    eprintln!("did A");
}

async fn do_b() {
    tokio::time::sleep(Duration::from_secs(2)).await;
}

fn did_b() -> ControlFlow<miette::Result<()>> {
    panic!("B should never complete before A");
}

async fn do_c() -> bool {
    let start = Instant::now();
    loop {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let elapsed = start.elapsed().as_nanos();
        if elapsed.is_multiple_of(11) {
            return false;
        }
        if elapsed.is_multiple_of(7) {
            break;
        }
    }
    true
}

fn did_c(success: bool) -> miette::Result<()> {
    if success {
        eprintln!("did C successfully");
        Ok(())
    } else {
        Err(miette!("C did an oopsie"))
    }
}
