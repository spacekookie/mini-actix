//! An exploration of the actix worker model and message sending

#![allow(warnings)]

mod context;
pub(crate) use context::*;

mod io;
pub(crate) use io::*;

mod messages;
pub(crate) use messages::*;

mod worker;
pub(crate) use worker::*;

use tokio::runtime::Runtime;

struct Printer {
    count: usize,
}

pub struct PrintMsg(String);

impl Message for PrintMsg {}

impl Worker for Printer {
    type Context = Context<Printer>;
}

impl Handler<PrintMsg> for Printer {
    fn handle(&mut self, print: &PrintMsg, _ctx: &mut Context<Self>) {
        println!("Printer: {}", print.0);
    }
}

fn main() {
    // What actix does that I don't
    //
    // - Setup a single-threaded tokio runtime per thread
    // - Setup threads manually (actix::Arbiter)
    // - Create channels between Arbiters and actix::System to be able to send commands
    //   - Command type: actix::SystemCommand
    //
    // System implements Future and can be poslled to read messages
    // from the rx-channel-end and then handles commands.
    //
    // In this example I simply rely on the default tokio runtime
    // which is multi-threaded
    let rt = Runtime::new().expect("Couldn't create runtime");

    // This is the equivalent of `#[actix::main] fn main() {}`
    rt.block_on(async {
        let mut w = Printer { count: 0 }.start("com.example.printer".into());

        w.send(PrintMsg("Hello, world!".into())).await;
    });
}
