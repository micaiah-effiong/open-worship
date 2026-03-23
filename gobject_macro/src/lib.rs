// gobject_macro/src/lib.rs

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Visibility, parse_macro_input, parse_quote};

/// Transforms a plain Rust struct into a GLib/GTK GObject with automatic property registration.
///
/// # Overview
///
/// This attribute macro replaces the struct definition with a full GObject implementation,
/// generating the `imp` module, `ObjectSubclass`, `ObjectImpl`, and `glib::wrapper!` boilerplate.
///
/// # Field Visibility Rules
///
/// - `pub` fields become GObject properties with `#[property(get, set)]` and emit `notify` signals.
/// - Private fields (no keyword) and restricted fields (`pub(super)`, etc.) are included in the
///   inner struct but are NOT registered as properties.
///
/// # Generated Items
///
/// - `mod imp` — contains the private `{Name}Priv` struct with `Default + Properties`
/// - `glib::wrapper!` — the public GObject newtype
/// - `impl {Name}::new(...)` — constructor taking only `pub` fields as arguments
///
/// # Example
///
/// ```rust
/// #[gobject_props]
/// struct Person {
///     pub name: String,   // becomes a GObject property
///     pub age:  u32,      // becomes a GObject property
///     secret: String,     // NOT a property, but accessible via self.imp().secret
/// }
///
/// // Generated constructor only takes pub fields:
/// let p = Person::new("Alice".to_string(), 30);
///
/// // Property access:
/// let name = p.name();                  // getter
/// p.set_name("Alicia".to_string());    // setter — emits notify::name
///
/// // Private field access:
/// p.imp().secret.borrow();
///
/// // Use in a GtkListStore:
/// let store = gio::ListStore::new::<Person>();
/// store.append(&Person::new("Bob".to_string(), 25));
///
/// // Reactive binding in XML — updates automatically when property changes:
/// // <binding name="label">
/// //   <lookup name="name" type="Person">
/// //     <lookup name="item">GtkListItem</lookup>
/// //   </lookup>
/// // </binding>
/// ```
///
/// # Supported Property Types
///
/// | Rust type | GLib ParamSpec        |
/// |-----------|----------------------|
/// | `String`  | `ParamSpecString`    |
/// | `u32`     | `ParamSpecUInt`      |
/// | `i32`     | `ParamSpecInt`       |
/// | `f64`     | `ParamSpecDouble`    |
/// | `f32`     | `ParamSpecFloat`     |
/// | `bool`    | `ParamSpecBoolean`   |
///
#[proc_macro_attribute]
pub fn gobject_props(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    match gobject_props_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn gobject_props_impl(input: DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let name = &input.ident;
    let priv_name = quote::format_ident!("{}Priv", name);
    let imp_name = quote::format_ident!("{}_priv", name.to_string().to_lowercase());
    let pub_super: Visibility = parse_quote!(pub(super));

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => {
                return Err(Error::new(
                    Span::call_site(),
                    "gobject_props only supports named fields",
                ));
            }
        },
        _ => {
            return Err(Error::new(
                Span::call_site(),
                "gobject_props only supports structs",
            ));
        }
    };

    let pub_fields: Vec<_> = fields
        .iter()
        .filter(|f| matches!(f.vis, Visibility::Public(_)))
        .collect();

    let priv_fields: Vec<_> = fields
        .iter()
        .filter(|f| !matches!(f.vis, Visibility::Public(_)))
        .collect();

    let pub_field_names: Vec<_> = pub_fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let pub_field_types: Vec<_> = pub_fields.iter().map(|f| &f.ty).collect();
    let pub_field_name_strs: Vec<_> = pub_field_names
        .iter()
        .map(|n| n.to_string().replace('_', "-"))
        .collect();

    let prop_fields: Vec<_> = pub_fields
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            quote! {
                #[property(get, set)]
                #name: std::cell::RefCell<#ty>
            }
        })
        .collect();

    let plain_fields: Vec<_> = priv_fields
        .iter()
        .map(|f| {
            let name = f.ident.as_ref().unwrap();
            let ty = &f.ty;
            let vis = &f.vis;

            let vis = match vis {
                Visibility::Inherited => pub_super.clone(),
                _ => vis.clone(),
            };

            quote! {
                #vis #name: std::cell::RefCell<#ty>
            }
        })
        .collect();

    let constructor_args: Vec<_> = pub_field_names
        .iter()
        .zip(pub_field_types.iter())
        .map(|(name, ty)| quote! { #name: #ty })
        .collect();

    let expanded = quote! {
        // imp contains the actual struct data — NOT the original Alert
        mod #imp_name {
            use super::*;
            use gtk::glib;
            use gtk::prelude::*;
            use gtk::subclass::prelude::*;
            use glib::Properties;

            #[derive(Default, Properties)]
            #[properties(wrapper_type = super::#name)]
            pub struct #priv_name {
                #(#prop_fields,)*
                #(#plain_fields,)*
            }

            #[glib::object_subclass]
            impl ObjectSubclass for #priv_name {
                const NAME: &'static str = stringify!(#name);
                type Type = super::#name;
            }

            #[glib::derived_properties]
            impl ObjectImpl for #priv_name {}
        }

        // This is the GObject wrapper — replaces the original struct
        glib::wrapper! {
            pub struct #name(ObjectSubclass<#imp_name::#priv_name>);
        }

        impl #name {
            pub fn new(#(#constructor_args,)*) -> Self {
                glib::Object::builder()
                    #(.property(#pub_field_name_strs, #pub_field_names))*
                    .build()
            }
        }
    }
    .into();
    // panic!("{}", expanded);
    Ok(expanded)
}
