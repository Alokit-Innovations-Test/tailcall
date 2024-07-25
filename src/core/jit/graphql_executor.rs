use std::future::Future;
use std::sync::Arc;

use async_graphql::{Data, Executor, Response};
use futures_util::stream::BoxStream;
use serde::Deserialize;
use crate::core::app_context::AppContext;
use crate::core::http::RequestContext;
use crate::core::jit;
use crate::core::jit::ConstValueExecutor;
use crate::core::json::JsonLike;

#[derive(Clone)]
pub struct JITExecutor<Value> {
    app_ctx: Arc<AppContext<Value>>,
    req_ctx: Arc<RequestContext<Value>>,
}

impl<'a, Value: JsonLike<'a> + Deserialize<'a> + Clone> JITExecutor<Value> {
    pub fn new(app_ctx: Arc<AppContext<Value>>, req_ctx: Arc<RequestContext<Value>>) -> Self {
        Self { app_ctx, req_ctx }
    }
}

impl<'a, Value: JsonLike<'a> + Clone> Executor for JITExecutor<Value> {
    fn execute(&self, request: async_graphql::Request) -> impl Future<Output = Response> + Send {
        let request = jit::Request::from(request);

        async {
            match ConstValueExecutor::new(&request, self.app_ctx.clone()) {
                Ok(exec) => {
                    let resp = exec.execute(&self.req_ctx, request).await;
                    resp.into_async_graphql()
                }
                Err(error) => Response::from_errors(vec![error.into_server_error()]),
            }
        }
    }

    fn execute_stream(
        &self,
        _: async_graphql::Request,
        _: Option<Arc<Data>>,
    ) -> BoxStream<'static, Response> {
        unimplemented!("streaming not supported")
    }
}
