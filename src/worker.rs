use crate::{Context, Handler, Message, Sender, ToEnvelope, WorkerContext};

// In actix the `WorkerHandle` is called an Address.

// Another difference to actix: Actors in actix are Sized + Unpin +
// 'static, however to make this code work I need to declare Worker:
// Send.  I wonder what the differences are.

pub trait Worker: Sized + Send + 'static {
    type Context: WorkerContext;

    fn start(self, addr: String) -> WorkerHandle<Self>
    where
        Self: Worker<Context = Context<Self>>,
    {
        let (ctx, tx) = Context::new();
        tokio::spawn(async move {
            ctx.run().await;
        });
        WorkerHandle::new(addr, tx)
    }
}

// !! Problem !!
//
// WorkeHandle is a type that we might want to be able to store
// somewhere in the ockam internals, because currenly external network
// nodes would not be able to send messages to this worker.  The
// problem with that approach becomes that we need to bubble the `W:
// Worker` generic up to the container type that is storing this
// wrapper.
//
// In the current Relay design this is not required because the Sender
// is not generic over a worker, but a concrete carrier Message type!

pub struct WorkerHandle<W: Worker> {
    addr: String,
    tx: Sender<W>,
}

impl<W: Worker<Context = Context<W>>> WorkerHandle<W> {
    fn new(addr: String, tx: Sender<W>) -> Self {
        Self { addr, tx }
    }

    pub async fn send<M: 'static>(&mut self, msg: M)
    where
        M: Message + Send,
        W::Context: ToEnvelope<W, M>,
        W: Handler<M>,
    {
        // !! Problem !!
        //
        // actix relies on its custom implementation of channels here,
        // because Sender<W>::send is actually generic over a Message,
        // and then Sender::send<M> ensures the bound of W: Handler<M>.
        //
        // This is a core part of the actix design (having handlers
        // implemented on Workers, and supporting many Message types).
        //
        // Potentially we could have a wrapper type that does
        // something similar, but more research will be required for
        // that.

        // self.tx.send(msg).await;
    }
}
