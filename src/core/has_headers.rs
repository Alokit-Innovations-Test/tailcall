use hyper::HeaderMap;

use crate::core::ir::{EvalContext, ResolverContextLike};

pub trait HasHeaders {
    fn headers(&self) -> &HeaderMap;
}

impl<'a, Ctx: ResolverContextLike, Value> HasHeaders for EvalContext<'a, Ctx, Value> {
    fn headers(&self) -> &HeaderMap {
        self.headers()
    }
}
