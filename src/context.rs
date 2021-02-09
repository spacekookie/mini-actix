use crate::{channel, Envelope, EnvelopeProxy, Receiver, Sender, Worker};

pub enum WorkerState {
    Started,
    Running,
    Stopping,
    Stopped,
    Unknown,
}

// What actix does here that I don't
//
// - `actix::Context` keeps track of worker state via state machine
// - This is implemented by `actix::ContextParts`
//   - `Receiver` from actix::System
//   - Generator function for Sender<A>
//
// Something I didn't find in the code: how do you get the address
// handle to an arbitrary Actor in actix?
pub trait WorkerContext: Sized {
    fn state(&self) -> WorkerState;
}

pub struct Context<W>
where
    W: Worker<Context = Context<W>>,
{
    mb: Mailbox<W>,
}

impl<W: Worker<Context = Context<W>>> Context<W> {
    pub(crate) fn new() -> (Self, Sender<W>) {
        let (tx, rx) = channel(16);
        (
            Self {
                mb: Mailbox::new(rx),
            },
            tx,
        )
    }

    /// Poll this context (mailbox)
    pub(crate) async fn run(mut self) {
        println!("Context::run()");
        self.mb.poll().await
    }
}

impl<W> WorkerContext for Context<W>
where
    W: Worker<Context = Context<W>>,
{
    fn state(&self) -> WorkerState {
        WorkerState::Unknown
    }
}

pub struct Mailbox<W: Worker> {
    msgs: Receiver<W>,
}

impl<W: Worker<Context = Context<W>>> Mailbox<W> {
    fn new(msgs: Receiver<W>) -> Self {
        Self { msgs }
    }

    async fn poll(&mut self) {
        println!("Mailbox::poll()");

        // This loop _should_ be getting an Envelope type - see
        // comment in worker.rs for a clarification on that problem!
        if let Some(_) = self.msgs.recv().await {
            let env: Envelope<W> = todo!();

            // What actix does here that I don't
            //
            // The mailbox needs to have mutable access to the Worker
            // and the Context.  Both should be available in
            // Context::run(), but it does require some mutability
            // shuffling that I didn't spend time on: the Context is
            // wrapped by a ContextFut which implements `Future` and
            // can then mutably borrow the three fields from itself
            // that it needs, instead of having to `&mut self` as
            // would be required in this example.
            
            // env.handle(worker, ctx)
        }
    }
}
