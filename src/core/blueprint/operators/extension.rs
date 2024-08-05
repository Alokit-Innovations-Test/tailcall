use crate::core::blueprint::FieldDefinition;
use crate::core::config;
use crate::core::config::{ConfigModule, Field};
use crate::core::ir::model::{Rust, IO, IR};
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};

// pub trait Execution {
// async fn init(&self);
// fn prepare<Json: JsonLike>(&self, ir: IR, params: &[Json]) -> IR;
// fn process<Json: JsonLike>(&self, process: &[Json], value: Json) -> Json
// }

fn compile_extension(
    // TODO: convert this value to generic
    extension: &config::Extension<serde_json::Value>,
    config_module: &ConfigModule,
) -> Valid<IR, String> {
    Valid::from_option(
        config_module.extensions().rust_lib.as_ref(),
        "A @link with path to dylib is required".to_string(),
    )
    .and_then(|lib| {
        let rust = Rust { lib: lib.clone(), extension: extension.clone() };
        Valid::succeed(IR::IO(IO::Rust { rust }))
    })
}

pub fn update_extension<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
        |(config_module, field, type_of, _), b_field| {
            let Some(extension) = &field.extension else {
                return Valid::succeed(b_field);
            };

            compile_extension(extension, config_module)
                .map(|resolver| b_field.resolver(Some(resolver)))
                .and_then(|b_field| {
                    b_field
                        .validate_field(type_of, config_module)
                        .map_to(b_field)
                })
        },
    )
}
