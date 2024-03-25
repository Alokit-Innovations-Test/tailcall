use criterion::{black_box, criterion_group, criterion_main, Criterion};
use derive_setters::Setters;
use hyper::HeaderMap;
use serde_json::json;
use tailcall::{endpoint::Endpoint, path_value::PathValue};
use tailcall::has_headers::HasHeaders;
use tailcall::http::RequestTemplate;

#[derive(Setters)]
struct Context {
    pub value: serde_json::Value,
    pub headers: HeaderMap,
}

impl Default for Context {
    fn default() -> Self {
        Self { value: serde_json::Value::Null, headers: HeaderMap::new() }
    }
}
impl PathValue for Context {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str> {
        self.value.get_path_value(path)
    }
}
impl HasHeaders for Context {
    fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}
fn benchmark_to_request(c: &mut Criterion) {
    let tmpl_mustache = RequestTemplate::try_from(Endpoint::new(
        "http://localhost:3000/{{args.b}}?a={{args.a}}&b={{args.b}}&c={{args.c}}".to_string(),
    ))
    .unwrap();

    let tmpl_literal = RequestTemplate::try_from(Endpoint::new(
        "http://localhost:3000/foo?a=bar&b=foo&c=baz".to_string(),
    ))
    .unwrap();

    let ctx = Context::default().value(json!({
      "args": {
        "b": "foo"
      }
    }));

    c.bench_function("with_mustache_literal", |b| {
        b.iter(|| {
            black_box(tmpl_literal.to_request(&ctx).unwrap());
        })
    });

    c.bench_function("with_mustache_expressions", |b| {
        b.iter(|| {
            black_box(tmpl_mustache.to_request(&ctx).unwrap());
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_to_request
}
criterion_main!(benches);
