//! Proc-macro derives for the Fable def wire model.
//!
//! These replace the former declaration macros (`def_struct!`, `wire_struct!`,
//! `def_variant!`, `def_enum!`, `def_flags!`). The declarations are now plain
//! Rust types carrying `#[def(...)]` / `#[flags(...)]` attributes; each derive
//! emits exactly the impls the old macro expanded to, so the compiled output is
//! byte-identical.
//!
//! All generated code refers to the def model through `crate::def::...`, so
//! these derives are only usable from within the `fable-data` crate (which is
//! their sole consumer).
//!
//! ## Attribute surface (`#[def(...)]`)
//!
//! The `def` namespace is intentionally open so later per-field annotations
//! (container semantics, ctor-arg maps — AGENTS.md §9 R4/R5) can be added as new
//! named arguments without breaking the forms below.
//!
//! - **[`macro@DefStruct`]** field: `#[def("WireName")]`, optionally
//!   `#[def("WireName", default = <expr>)]`. The leading string literal is the
//!   field's def-script wire name (its `crc32` id); `default` overrides the
//!   type's `DefDefault` for the (never-parsed) NULLDEF body.
//! - **[`macro@WireStruct`]** field: no attribute, or `#[def(default = <expr>)]`.
//!   Compound members carry no wire name (they are positional, id-less).
//! - **[`macro@DefVariant`]** variant: `#[def(<tag>)]`, the `u32` union tag.
//! - **[`macro@DefEnum`]** variant: `#[def("SYMBOL")]`, the C++ enumerator name;
//!   the wire value is the variant's explicit discriminant (`Variant = 3`).
//! - **[`macro@DefFlags`]** type: `#[flags(FLAG = <value> => "SYMBOL", ...)]`.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    Data, DeriveInput, Expr, Fields, Ident, LitInt, LitStr, Token, Type, Variant, parse_macro_input,
};

// ── shared `#[def(...)]` parsing ────────────────────────────────────────────

/// One argument inside a `#[def(...)]` list: a positional literal or a
/// `name = value` pair.
enum DefArg {
    Str(LitStr),
    Int(LitInt),
    Named(Ident, Expr),
}

impl Parse for DefArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(DefArg::Str(input.parse()?))
        } else if input.peek(LitInt) {
            Ok(DefArg::Int(input.parse()?))
        } else {
            let name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: Expr = input.parse()?;
            Ok(DefArg::Named(name, value))
        }
    }
}

/// The parsed contents of a `#[def(...)]` attribute.
#[derive(Default)]
struct DefAttr {
    /// Leading positional string literal (wire name / enum symbol).
    str_lit: Option<LitStr>,
    /// Leading positional integer literal (variant tag).
    int_lit: Option<LitInt>,
    /// `default = <expr>`.
    default: Option<Expr>,
}

/// Collect the (at most one) `#[def(...)]` attribute on an item, or default if
/// absent.
fn parse_def_attr(attrs: &[syn::Attribute]) -> syn::Result<DefAttr> {
    let mut out = DefAttr::default();
    let mut seen = false;
    for attr in attrs {
        if !attr.path().is_ident("def") {
            continue;
        }
        if seen {
            return Err(syn::Error::new_spanned(
                attr,
                "duplicate #[def(...)] attribute",
            ));
        }
        seen = true;
        let args = attr.parse_args_with(Punctuated::<DefArg, Token![,]>::parse_terminated)?;
        for arg in args {
            match arg {
                DefArg::Str(s) => {
                    if out.str_lit.replace(s).is_some() {
                        return Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            "multiple positional string literals in #[def(...)]",
                        ));
                    }
                }
                DefArg::Int(i) => {
                    if out.int_lit.replace(i).is_some() {
                        return Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            "multiple positional integer literals in #[def(...)]",
                        ));
                    }
                }
                DefArg::Named(name, value) => {
                    if name == "default" {
                        out.default = Some(value);
                    } else {
                        return Err(syn::Error::new_spanned(
                            name,
                            "unknown #[def(...)] argument",
                        ));
                    }
                }
            }
        }
    }
    Ok(out)
}

/// The default expression for a field: the explicit `default = expr` if given,
/// else the type's `DefDefault::def_default()`.
fn field_default(ty: &Type, default: &Option<Expr>) -> TokenStream2 {
    match default {
        Some(expr) => quote!(#expr),
        None => quote!(<#ty as crate::def::visit::DefDefault>::def_default()),
    }
}

fn named_fields(fields: &Fields, ctx: &str) -> Vec<syn::Field> {
    match fields {
        Fields::Named(named) => named.named.iter().cloned().collect(),
        _ => panic!("{ctx} requires named fields"),
    }
}

// ── DefStruct ───────────────────────────────────────────────────────────────

/// A def class: its body is a sequence of field controls (`u32 crc32(name)` +
/// [`Wire`] value) in declaration order. Each field needs a `#[def("WireName")]`.
#[proc_macro_derive(DefStruct, attributes(def))]
pub fn derive_def_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let fields = match &input.data {
        Data::Struct(s) => named_fields(&s.fields, "DefStruct"),
        _ => panic!("DefStruct can only derive for structs"),
    };

    let mut idents = Vec::new();
    let mut wire_names = Vec::new();
    let mut defaults = Vec::new();
    for f in &fields {
        let ident = f.ident.clone().expect("named field");
        let attr = match parse_def_attr(&f.attrs) {
            Ok(a) => a,
            Err(e) => return e.to_compile_error().into(),
        };
        let wire_name = match attr.str_lit {
            Some(s) => s,
            None => {
                return syn::Error::new_spanned(
                    &ident,
                    "DefStruct field needs #[def(\"WireName\")]",
                )
                .to_compile_error()
                .into();
            }
        };
        defaults.push(field_default(&f.ty, &attr.default));
        idents.push(ident);
        wire_names.push(wire_name);
    }

    let member_names: Vec<_> = idents.iter().map(|i| i.to_string()).collect();
    let type_name = name.to_string();

    let expanded = quote! {
        impl Default for #name {
            fn default() -> Self {
                Self { #( #idents: #defaults, )* }
            }
        }

        impl crate::def::visit::DefDefault for #name {
            fn def_default() -> Self {
                Self::default()
            }
        }

        impl #name {
            pub(crate) fn parse(
                cur: &mut &[u8],
            ) -> Result<Self, crate::def::binary::control::ParseControlError> {
                Ok(Self {
                    #( #idents: crate::def::wire::parse_field(cur, #wire_names)?, )*
                })
            }

            pub(crate) fn serialize(
                &self,
                out: &mut &mut [u8],
            ) -> Result<(), crate::def::binary::control::SerializeControlError> {
                #( crate::def::wire::serialize_field(out, #wire_names, &self.#idents)?; )*
                Ok(())
            }

            pub(crate) fn byte_size(&self) -> usize {
                0 #( + crate::def::wire::field_size(&self.#idents) )*
            }

            /// Hand each field to `visitor` as a typed
            /// [`FieldRef`](crate::def::visit::FieldRef), in declaration order.
            pub fn visit_fields<V: crate::def::visit::FieldVisitor>(&mut self, visitor: &mut V) {
                #(
                    visitor.field(
                        #wire_names,
                        crate::def::visit::AsField::as_field(&mut self.#idents),
                    );
                )*
            }
        }

        impl crate::def::visit::VisitFields for #name {
            fn visit_fields<V: crate::def::visit::FieldVisitor>(&mut self, visitor: &mut V) {
                #name::visit_fields(self, visitor);
            }
        }

        impl crate::def::wire::Wire for #name {
            fn parse(
                cur: &mut &[u8],
            ) -> Result<Self, crate::def::wire::ParseWireError> {
                #name::parse(cur).map_err(crate::def::wire::ParseWireError::from)
            }

            fn serialize(
                &self,
                out: &mut &mut [u8],
            ) -> Result<(), crate::bytes::UnexpectedEnd> {
                #name::serialize(self, out)
                    .map_err(crate::def::binary::control::SerializeControlError::unexpected_end)
            }

            fn wire_size(&self) -> usize {
                self.byte_size()
            }
        }

        impl crate::def::visit::StructSlot for #name {
            fn type_name(&self) -> &'static str {
                #type_name
            }
            fn member_count(&self) -> usize {
                [ #( #member_names ),* ].len()
            }
            fn member_name(&self, index: usize) -> Option<&'static str> {
                [ #( #member_names ),* ].get(index).copied()
            }
            #[allow(unused_assignments, unused_variables, unused_mut)]
            fn member<'b>(&'b mut self, index: usize) -> Option<crate::def::visit::FieldRef<'b>> {
                let mut i = 0usize;
                #(
                    if i == index {
                        return Some(crate::def::visit::AsField::as_field(&mut self.#idents));
                    }
                    i += 1;
                )*
                None
            }
            fn visit_named(&mut self, visitor: &mut dyn crate::def::visit::FieldVisitor) -> bool {
                // `DefStruct` fields carry their def-script names, so a def_struct
                // used as a `Vec` element / map value can be lowered by name.
                let mut fwd: &mut dyn crate::def::visit::FieldVisitor = visitor;
                #name::visit_fields(self, &mut fwd);
                true
            }
        }

        impl crate::def::visit::AsField for #name {
            fn as_field(&mut self) -> crate::def::visit::FieldRef<'_> {
                crate::def::visit::FieldRef::Struct(self)
            }
        }
    };
    expanded.into()
}

// ── WireStruct ──────────────────────────────────────────────────────────────

/// A compound wire value: a fixed sequence of member values with NO control ids
/// (`C3DVector`, `CEngineGraphic`, …). Members are positional; a member may
/// carry `#[def(default = <expr>)]`.
#[proc_macro_derive(WireStruct, attributes(def))]
pub fn derive_wire_struct(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let fields = match &input.data {
        Data::Struct(s) => named_fields(&s.fields, "WireStruct"),
        _ => panic!("WireStruct can only derive for structs"),
    };

    let mut idents = Vec::new();
    let mut types = Vec::new();
    let mut defaults = Vec::new();
    for f in &fields {
        let ident = f.ident.clone().expect("named field");
        let attr = match parse_def_attr(&f.attrs) {
            Ok(a) => a,
            Err(e) => return e.to_compile_error().into(),
        };
        if let Some(s) = attr.str_lit {
            return syn::Error::new_spanned(s, "WireStruct members carry no wire name")
                .to_compile_error()
                .into();
        }
        defaults.push(field_default(&f.ty, &attr.default));
        idents.push(ident);
        types.push(f.ty.clone());
    }

    let member_names: Vec<String> = idents.iter().map(|i| i.to_string()).collect();
    let field_name_strs: Vec<String> = idents.iter().map(|i| i.to_string()).collect();
    let type_name = name.to_string();

    let expanded = quote! {
        impl crate::def::visit::DefDefault for #name {
            fn def_default() -> Self {
                Self { #( #idents: #defaults, )* }
            }
        }

        impl crate::def::wire::Wire for #name {
            fn parse(
                cur: &mut &[u8],
            ) -> Result<Self, crate::def::wire::ParseWireError> {
                Ok(Self {
                    #(
                        #idents: <#types as crate::def::wire::Wire>::parse(cur)
                            .map_err(crate::def::wire::ParseWireError::member(#field_name_strs))?,
                    )*
                })
            }

            fn serialize(
                &self,
                out: &mut &mut [u8],
            ) -> Result<(), crate::bytes::UnexpectedEnd> {
                #( self.#idents.serialize(out)?; )*
                Ok(())
            }

            fn wire_size(&self) -> usize {
                0 #( + self.#idents.wire_size() )*
            }
        }

        impl crate::def::visit::StructSlot for #name {
            fn type_name(&self) -> &'static str {
                #type_name
            }
            fn member_count(&self) -> usize {
                [ #( #member_names ),* ].len()
            }
            fn member_name(&self, index: usize) -> Option<&'static str> {
                [ #( #member_names ),* ].get(index).copied()
            }
            #[allow(unused_assignments, unused_variables, unused_mut)]
            fn member<'b>(&'b mut self, index: usize) -> Option<crate::def::visit::FieldRef<'b>> {
                let mut i = 0usize;
                #(
                    if i == index {
                        return Some(crate::def::visit::AsField::as_field(&mut self.#idents));
                    }
                    i += 1;
                )*
                None
            }
        }

        impl crate::def::visit::AsField for #name {
            fn as_field(&mut self) -> crate::def::visit::FieldRef<'_> {
                crate::def::visit::FieldRef::Struct(self)
            }
        }
    };
    expanded.into()
}

// ── DefVariant ──────────────────────────────────────────────────────────────

/// A tagged-union wire value: a `u32` tag then case-specific fields. Each
/// variant carries `#[def(<tag>)]` and struct-style named wire fields.
#[proc_macro_derive(DefVariant, attributes(def))]
pub fn derive_def_variant(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let variants: Vec<Variant> = match &input.data {
        Data::Enum(e) => e.variants.iter().cloned().collect(),
        _ => panic!("DefVariant can only derive for enums"),
    };

    let mut vidents = Vec::new();
    let mut tags = Vec::new();
    let mut vfields: Vec<Vec<Ident>> = Vec::new();
    let mut vtypes: Vec<Vec<Type>> = Vec::new();
    for v in &variants {
        let attr = match parse_def_attr(&v.attrs) {
            Ok(a) => a,
            Err(e) => return e.to_compile_error().into(),
        };
        let tag = match attr.int_lit {
            Some(i) => i,
            None => {
                return syn::Error::new_spanned(&v.ident, "DefVariant variant needs #[def(<tag>)]")
                    .to_compile_error()
                    .into();
            }
        };
        let fields = named_fields(&v.fields, "DefVariant variant");
        vfields.push(fields.iter().map(|f| f.ident.clone().unwrap()).collect());
        vtypes.push(fields.iter().map(|f| f.ty.clone()).collect());
        vidents.push(v.ident.clone());
        tags.push(tag);
    }

    let first_ident = &vidents[0];
    let first_fields = &vfields[0];

    // Per-variant token fragments.
    let parse_arms = vidents.iter().zip(&tags).zip(&vfields).zip(&vtypes).map(
        |(((vident, tag), fields), types)| {
            let field_strs: Vec<String> = fields.iter().map(|f| f.to_string()).collect();
            quote! {
                #tag => Self::#vident {
                    #(
                        #fields: <#types as crate::def::wire::Wire>::parse(cur)
                            .map_err(crate::def::wire::ParseWireError::member(#field_strs))?,
                    )*
                },
            }
        },
    );

    let serialize_arms = vidents
        .iter()
        .zip(&tags)
        .zip(&vfields)
        .map(|((vident, tag), fields)| {
            quote! {
                Self::#vident { #( #fields ),* } => {
                    <u32 as crate::def::wire::Wire>::serialize(&#tag, out)?;
                    #( #fields.serialize(out)?; )*
                }
            }
        });

    let size_arms = vidents.iter().zip(&vfields).map(|(vident, fields)| {
        quote! {
            Self::#vident { #( #fields ),* } => {
                0 #( + #fields.wire_size() )*
            }
        }
    });

    let tag_arms = vidents
        .iter()
        .zip(&tags)
        .map(|(vident, tag)| quote!( Self::#vident { .. } => #tag, ));

    let set_tag_arms = vidents.iter().zip(&tags).zip(&vfields).map(|((vident, tag), fields)| {
        quote! {
            #tag => {
                *self = Self::#vident { #( #fields: crate::def::visit::DefDefault::def_default(), )* };
                true
            }
        }
    });

    let member_count_arms = vidents.iter().zip(&vfields).map(|(vident, fields)| {
        let n = fields.len();
        quote!( Self::#vident { .. } => #n, )
    });

    let member_name_arms = vidents.iter().zip(&vfields).map(|(vident, fields)| {
        let strs: Vec<String> = fields.iter().map(|f| f.to_string()).collect();
        quote!( Self::#vident { .. } => [ #( #strs ),* ].get(index).copied(), )
    });

    let member_arms = vidents.iter().zip(&vfields).map(|(vident, fields)| {
        quote! {
            Self::#vident { #( #fields ),* } => {
                let mut i = 0usize;
                #(
                    if i == index {
                        return Some(crate::def::visit::AsField::as_field(#fields));
                    }
                    i += 1;
                )*
                None
            }
        }
    });

    let expanded = quote! {
        impl crate::def::visit::DefDefault for #name {
            fn def_default() -> Self {
                // The first (tag-0) variant is the conventional default.
                Self::#first_ident {
                    #( #first_fields: crate::def::visit::DefDefault::def_default(), )*
                }
            }
        }

        impl crate::def::wire::Wire for #name {
            fn parse(
                cur: &mut &[u8],
            ) -> Result<Self, crate::def::wire::ParseWireError> {
                let tag = <u32 as crate::def::wire::Wire>::parse(cur)?;
                Ok(match tag {
                    #( #parse_arms )*
                    other => {
                        return Err(crate::def::wire::ParseWireError::InvalidVariantTag(other));
                    }
                })
            }

            fn serialize(
                &self,
                out: &mut &mut [u8],
            ) -> Result<(), crate::bytes::UnexpectedEnd> {
                match self {
                    #( #serialize_arms )*
                }
                Ok(())
            }

            fn wire_size(&self) -> usize {
                size_of::<u32>() + match self {
                    #( #size_arms )*
                }
            }
        }

        impl crate::def::visit::VariantSlot for #name {
            fn type_name(&self) -> &'static str {
                stringify!(#name)
            }
            fn tag(&self) -> u32 {
                match self {
                    #( #tag_arms )*
                }
            }
            fn set_tag(&mut self, tag: u32) -> bool {
                match tag {
                    #( #set_tag_arms )*
                    _ => false,
                }
            }
            fn member_count(&self) -> usize {
                match self {
                    #( #member_count_arms )*
                }
            }
            fn member_name(&self, index: usize) -> Option<&'static str> {
                match self {
                    #( #member_name_arms )*
                }
            }
            #[allow(unused_assignments, unused_variables, unused_mut)]
            fn member<'b>(&'b mut self, index: usize) -> Option<crate::def::visit::FieldRef<'b>> {
                match self {
                    #( #member_arms )*
                }
            }
        }

        impl crate::def::visit::AsField for #name {
            fn as_field(&mut self) -> crate::def::visit::FieldRef<'_> {
                crate::def::visit::FieldRef::Variant(self)
            }
        }
    };
    expanded.into()
}

// ── DefEnum ─────────────────────────────────────────────────────────────────

/// A closed `i32`-repr enum with a total mapping to/from the wire value and the
/// C++ enumerator symbols. Each variant carries its explicit discriminant
/// (`Variant = 3`) and `#[def("SYMBOL")]`. Out-of-table parse is an error.
#[proc_macro_derive(DefEnum, attributes(def))]
pub fn derive_def_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let variants: Vec<Variant> = match &input.data {
        Data::Enum(e) => e.variants.iter().cloned().collect(),
        _ => panic!("DefEnum can only derive for enums"),
    };

    let mut vidents = Vec::new();
    let mut values = Vec::new();
    let mut symbols = Vec::new();
    for v in &variants {
        let attr = match parse_def_attr(&v.attrs) {
            Ok(a) => a,
            Err(e) => return e.to_compile_error().into(),
        };
        let symbol = match attr.str_lit {
            Some(s) => s,
            None => {
                return syn::Error::new_spanned(
                    &v.ident,
                    "DefEnum variant needs #[def(\"SYMBOL\")]",
                )
                .to_compile_error()
                .into();
            }
        };
        let value = match &v.discriminant {
            Some((_, expr)) => expr.clone(),
            None => {
                return syn::Error::new_spanned(
                    &v.ident,
                    "DefEnum variant needs an explicit discriminant (`Variant = <value>`)",
                )
                .to_compile_error()
                .into();
            }
        };
        vidents.push(v.ident.clone());
        values.push(value);
        symbols.push(symbol);
    }

    let first_ident = &vidents[0];

    let expanded = quote! {
        impl #name {
            /// The original C++ enumerator name, as used by text defs.
            pub const fn symbol(self) -> &'static str {
                match self { #( Self::#vidents => #symbols, )* }
            }

            /// Look up a variant by its C++ enumerator name.
            pub fn from_symbol(symbol: &str) -> Option<Self> {
                match symbol {
                    #( #symbols => Some(Self::#vidents), )*
                    _ => None,
                }
            }
        }

        impl crate::def::enums::DefEnum for #name {
            fn from_i32(value: i32) -> Option<Self> {
                match value {
                    #( #values => Some(Self::#vidents), )*
                    _ => None,
                }
            }

            fn to_i32(self) -> i32 {
                self as i32
            }
        }

        impl crate::def::visit::DefDefault for #name {
            fn def_default() -> Self {
                // The first variant is the conventional default/zero value.
                Self::#first_ident
            }
        }

        impl crate::def::wire::Wire for #name {
            fn parse(
                cur: &mut &[u8],
            ) -> Result<Self, crate::def::wire::ParseWireError> {
                use crate::def::enums::DefEnum;
                let value = <i32 as crate::def::wire::Wire>::parse(cur)?;
                Self::from_i32(value)
                    .ok_or(crate::def::wire::ParseWireError::InvalidEnumValue { value })
            }

            fn serialize(
                &self,
                out: &mut &mut [u8],
            ) -> Result<(), crate::bytes::UnexpectedEnd> {
                use crate::def::enums::DefEnum;
                crate::def::wire::Wire::serialize(&self.to_i32(), out)
            }

            fn wire_size(&self) -> usize {
                size_of::<i32>()
            }
        }

        impl crate::def::visit::EnumSlot for #name {
            fn get_i32(&self) -> i32 {
                use crate::def::enums::DefEnum;
                self.to_i32()
            }

            fn set_i32(&mut self, value: i32) -> Result<(), i32> {
                use crate::def::enums::DefEnum;
                *self = Self::from_i32(value).ok_or(value)?;
                Ok(())
            }
        }

        impl crate::def::visit::AsField for #name {
            fn as_field(&mut self) -> crate::def::visit::FieldRef<'_> {
                crate::def::visit::FieldRef::Enum(self)
            }
        }
    };
    expanded.into()
}

// ── DefFlags ────────────────────────────────────────────────────────────────

/// One `FLAG = value => "SYMBOL"` row of a `#[flags(...)]` list.
struct FlagRow {
    ident: Ident,
    value: LitInt,
    symbol: LitStr,
}

impl Parse for FlagRow {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        let value: LitInt = input.parse()?;
        input.parse::<Token![=>]>()?;
        let symbol: LitStr = input.parse()?;
        Ok(FlagRow {
            ident,
            value,
            symbol,
        })
    }
}

/// A bit-set newtype (`struct Name(pub i32)`) for the "enums" the game ORs
/// together. Flags are listed in a type-level `#[flags(FLAG = value => "SYMBOL",
/// ...)]` attribute.
#[proc_macro_derive(DefFlags, attributes(flags))]
pub fn derive_def_flags(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let flags_attr = match input.attrs.iter().find(|a| a.path().is_ident("flags")) {
        Some(a) => a,
        None => {
            return syn::Error::new_spanned(name, "DefFlags needs a #[flags(...)] attribute")
                .to_compile_error()
                .into();
        }
    };
    let rows = match flags_attr.parse_args_with(Punctuated::<FlagRow, Token![,]>::parse_terminated)
    {
        Ok(r) => r,
        Err(e) => return e.to_compile_error().into(),
    };

    let idents: Vec<&Ident> = rows.iter().map(|r| &r.ident).collect();
    let values: Vec<&LitInt> = rows.iter().map(|r| &r.value).collect();
    let symbols: Vec<&LitStr> = rows.iter().map(|r| &r.symbol).collect();

    let expanded = quote! {
        #[allow(non_upper_case_globals)]
        impl #name {
            #( pub const #idents: Self = Self(#values); )*

            pub const fn from_i32(value: i32) -> Self {
                Self(value)
            }

            pub const fn to_i32(self) -> i32 {
                self.0
            }

            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }

            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }

            /// The C++ enumerator name, when this is exactly one known flag.
            pub const fn symbol(self) -> Option<&'static str> {
                // Some flag sets alias a value (e.g. a `NONE = 0`), so later arms
                // for the same value are unreachable — the first symbol wins.
                #[allow(unreachable_patterns)]
                match self.0 {
                    #( #values => Some(#symbols), )*
                    _ => None,
                }
            }

            /// Look up a single flag by its C++ enumerator name.
            pub fn from_symbol(symbol: &str) -> Option<Self> {
                match symbol {
                    #( #symbols => Some(Self::#idents), )*
                    _ => None,
                }
            }
        }

        impl crate::def::visit::DefDefault for #name {
            fn def_default() -> Self {
                Self(0)
            }
        }

        impl core::ops::BitOr for #name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl core::ops::BitOrAssign for #name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl crate::def::wire::Wire for #name {
            fn parse(
                cur: &mut &[u8],
            ) -> Result<Self, crate::def::wire::ParseWireError> {
                Ok(Self::from_i32(<i32 as crate::def::wire::Wire>::parse(cur)?))
            }

            fn serialize(
                &self,
                out: &mut &mut [u8],
            ) -> Result<(), crate::bytes::UnexpectedEnd> {
                crate::def::wire::Wire::serialize(&self.to_i32(), out)
            }

            fn wire_size(&self) -> usize {
                size_of::<i32>()
            }
        }

        impl crate::def::visit::FlagsSlot for #name {
            fn get_i32(&self) -> i32 {
                self.0
            }

            fn set_i32(&mut self, value: i32) {
                self.0 = value;
            }
        }

        impl crate::def::visit::AsField for #name {
            fn as_field(&mut self) -> crate::def::visit::FieldRef<'_> {
                crate::def::visit::FieldRef::Flags(self)
            }
        }
    };
    expanded.into()
}
