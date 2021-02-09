
use crate::{Context, Worker};

// What actix does here that I don't
//
// - Handler type `Result` which needs to implement `MesageResponse`
//   - Has a further `handle(...)` which takes a return channel tx
// - Message has arbitrary Result type (bound to 'static)


pub trait Handler<M: Message>: Worker {
    fn handle(&mut self, _msg: &M, _ctx: &mut Self::Context);
}

pub trait Message {}


// Magic Envelope type over worker stolen almost verbatim from actix
pub struct Envelope<W: Worker>(Box<dyn EnvelopeProxy<W> + Send>);

// This trait exists to let whatever type EnvelopeProxy is provide a
// Message type that doesn't need to be present in the Envelope
// (i.e. an Envelope is _for_ a worker, but the actual
// _implementation_ (`SyncEnvelopeProxy` - same name as in actix)
// depends also on M, which is how it knows how to call the correct
// message handler!)
pub trait EnvelopeProxy<W: Worker> {
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
        // This call only exists to wrap around the trait object in Envelope
        self.0.handle(act, ctx)
    }
}

// In actix this type also contains a TX return channel for Message
// replies, which I dropped because my messages don't want to be
// replied to :)
pub struct SyncEnvelopeProxy<M: Message + Send> {
    msg: M,
}

impl<W, M> EnvelopeProxy<W> for SyncEnvelopeProxy<M>
where
    W: Worker + Handler<M>,
    M: Message + Send + 'static,
{
    fn handle(&mut self, act: &mut W, ctx: &mut <W as Worker>::Context) {
        // Call the actual message handle!
        <W as Handler<M>>::handle(act, &self.msg, ctx);
    }
}

// A very simple packing trait to turn messages into Envelopes and
// blanket implemented for any worker context that implements a
// handler for the specific message type being packed.  This trait is
// being enforced in `WorkerHandle::send(...)`
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
