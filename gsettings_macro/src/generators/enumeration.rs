use heck::ToPascalCase;

use super::{Context, KeyGenerator, SchemaEnum, SchemaKey};

pub fn key_generator<'a>(
    key: &'a SchemaKey,
    enum_: &SchemaEnum,
    aux_visibility: syn::Visibility,
) -> KeyGenerator<'a> {
    let enum_name = key.name.to_pascal_case();
    let enum_token_stream = super::new_variant_enum(
        &enum_name,
        &enum_
            .values
            .iter()
            .map(|value| (value.nick.as_str(), Some(value.value)))
            .collect::<Vec<_>>(),
        aux_visibility,
    );
    KeyGenerator::new(key, Context::new_with_aux(&enum_name, enum_token_stream))
}
