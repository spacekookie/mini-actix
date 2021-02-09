// What actix does here that I don't
//
// - Handler type `Result` which needs to implement `MesageResponse`
//   - Has a further `handle(...)` which takes a return channel tx
// - Message has arbitrary Result type (bound to 'static)

use crate::{Context, Worker};

pub trait Handler<M: Message>: Worker {
    fn handle(&mut self, _msg: &M, _ctx: &mut Self::Context);
}

/// A simple marker trait for messages
pub trait Message {}

pub struct Envelope<W: Worker>(Box<dyn EnvelopeProxy<W> + Send>);

pub trait EnvelopeProxy<W: Worker> {
    /// handle message within new actor and context
    fn handle(&mut self, worker: &mut W, ctx: &mut W::Context);
}

impl<W: Worker> Envelope<W> {
    pub fn new<M>(msg: M) -> Self
    where
        W: Handler<M>,
        M: Message + Send + 'static,
    {
        Envelope(Box::new(SyncEnvelopeProxy { msg }))
    }
}

impl<W: Worker> EnvelopeProxy<W> for Envelope<W> {
    fn handle(&mut self, act: &mut W, ctx: &mut <W as Worker>::Context) {
        self.0.handle(act, ctx)
    }
}

pub struct SyncEnvelopeProxy<M: Message + Send> {
    msg: M,
}

impl<W, M> EnvelopeProxy<W> for SyncEnvelopeProxy<M>
where
    W: Worker + Handler<M>,
    M: Message + Send + 'static,
{
    fn handle(&mut self, act: &mut W, ctx: &mut <W as Worker>::Context) {
        // Call the message handle!
        <W as Handler<M>>::handle(act, &self.msg, ctx);
    }
}

pub trait ToEnvelope<W, M>
where
    W: Worker + Handler<M>,
    M: Message,
{
    /// Pack message into suitable envelope
    fn pack(msg: M) -> Envelope<W>;
}

impl<W, M> ToEnvelope<W, M> for Context<W>
where
    W: Worker<Context = Context<W>> + Handler<M>,
    M: Message + Send + 'static,
{
    fn pack(msg: M) -> Envelope<W> {
        Envelope::new(msg)
    }
}
