use heck::{ToPascalCase, ToShoutySnakeCase};
use quote::quote;
use syn::{spanned::Spanned, Ident};

use super::{Context, KeyGenerator, SchemaFlag, SchemaKey};

pub fn key_generator<'a>(
    key: &'a SchemaKey,
    flag: &SchemaFlag,
    aux_visibility: syn::Visibility,
) -> KeyGenerator<'a> {
    let flag_name = key.name.to_pascal_case();
    KeyGenerator::new(
        key,
        Context::new_with_aux(
            &flag_name,
            bitflag_token_stream(&flag_name, flag, aux_visibility),
        ),
    )
}

fn bitflag_token_stream(
    name: &str,
    flag: &SchemaFlag,
    visibility: syn::Visibility,
) -> proc_macro2::TokenStream {
    let value_idents = flag
        .values
        .iter()
        .map(|value| Ident::new(&value.nick.to_shouty_snake_case(), value.nick.span()))
        .collect::<Vec<_>>();

    let flags_arms = value_idents
        .iter()
        .zip(flag.values.iter())
        .map(|(value_ident, value)| {
            let value = value.value;
            quote! {
                const #value_ident = #value;
            }
        });

    let from_variant_arms =
        value_idents
            .iter()
            .zip(flag.values.iter())
            .map(|(value_ident, value)| {
                let nick = &value.nick;
                quote! {
                    #nick => this.insert(Self::#value_ident)
                }
            });

    let to_variant_arms =
        value_idents
            .iter()
            .zip(flag.values.iter())
            .map(|(value_ident, value)| {
                let nick = &value.nick;
                quote! {
                    if self.contains(Self::#value_ident) {
                        string_array.push(#nick)
                    }
                }
            });

    let name_pascal_case = name.to_pascal_case();
    let ident = Ident::new(&name_pascal_case, name_pascal_case.span());

    quote! {
        gio::glib::bitflags::bitflags! {
            #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
            #visibility struct #ident: u32 {
                #(#flags_arms)*
            }
        }

        impl gio::glib::variant::StaticVariantType for #ident {
            fn static_variant_type() -> std::borrow::Cow<'static, gio::glib::VariantTy> {
                std::borrow::Cow::Borrowed(gio::glib::VariantTy::STRING_ARRAY)
            }
        }

        impl gio::glib::variant::FromVariant for #ident {
            fn from_variant(variant: &gio::glib::Variant) -> Option<Self> {
                let mut this = Self::empty();

                for string in variant.get::<Vec<String>>()? {
                    match string.as_str() {
                        #(#from_variant_arms),*,
                        _ => return None,
                    }
                }

                Some(this)
            }
        }

        impl gio::glib::variant::ToVariant for #ident {
            fn to_variant(&self) -> gio::glib::Variant {
                let mut string_array = Vec::new();

                #(#to_variant_arms)*

                gio::glib::variant::ToVariant::to_variant(&string_array)
            }
        }

        impl std::convert::From<#ident> for gio::glib::Variant {
            fn from(this: #ident) -> gio::glib::Variant {
                gio::glib::variant::ToVariant::to_variant(&this)
            }
        }
    }
}
