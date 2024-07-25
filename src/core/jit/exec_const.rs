use std::sync::Arc;

use async_graphql_value::ConstValue;
use serde::Deserialize;
use super::context::Context;
use super::exec::{Executor, IRExecutor};
use super::{Error, OperationPlan, Request, Response, Result};
use crate::core::app_context::AppContext;
use crate::core::http::RequestContext;
use crate::core::ir::model::IR;
use crate::core::ir::EvalContext;
use crate::core::jit::synth::Synth;
use crate::core::json::JsonLike;

/// A specialized executor that executes with async_graphql::Value
pub struct ConstValueExecutor {
    // maybe we can convert it to generic val
    plan: OperationPlan<ConstValue>,
}

impl ConstValueExecutor {
    pub fn new<'a, Value: JsonLike<'a> + Deserialize<'a> + Clone>(request: &Request<ConstValue>, app_ctx: Arc<AppContext<Value>>) -> Result<Self> {
        Ok(Self { plan: request.create_plan(&app_ctx.blueprint)? })
    }

    pub async fn execute<'a, Value: JsonLike<'a> + Deserialize<'a> + Clone>(
        self,
        req_ctx: &'a RequestContext<Value>,
        request: Request<ConstValue>,
    ) -> Response<ConstValue, Error> {
        let exec = ConstValueExec::new(req_ctx);
        let plan = self.plan;
        // TODO: drop the clones in plan
        let vars = request.variables.clone();
        let exe = Executor::new(plan.clone(), exec);
        let store = exe.store(request).await;
        let synth = Synth::new(plan, store, vars);
        exe.execute(synth).await
    }
}

struct ConstValueExec<'a, Value> {
    req_context: &'a RequestContext<Value>,
}

impl<'a, Value: JsonLike<'a> + Deserialize<'a> + Clone> ConstValueExec<'a, Value> {
    pub fn new(ctx: &'a RequestContext<Value>) -> Self {
        Self { req_context: ctx }
    }
}

#[async_trait::async_trait]
impl<'ctx> IRExecutor for ConstValueExec<'ctx, async_graphql::Value> {
    type Input = ConstValue;
    type Output = ConstValue;
    type Error = Error;

    async fn execute<'a>(
        &'a self,
        ir: &'a IR<Self::Output>,
        ctx: &'a Context<'a, Self::Input, Self::Output>,
    ) -> Result<Self::Output> {
        let req_context = &self.req_context;
        let mut ctx = EvalContext::new(req_context, ctx);
        Ok(ir.eval(&mut ctx).await?)
    }
}
