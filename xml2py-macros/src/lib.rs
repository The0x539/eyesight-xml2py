use quote::quote;
use syn::{parse_quote, Expr, Field, Fields, FieldsNamed, ItemStruct, Lit, Meta, Type};

#[proc_macro_attribute]
pub fn node(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item = syn::parse_macro_input!(item as ItemStruct);

    item.attrs
        .push(parse_quote!(#[derive(Deserialize, Debug, PartialEq, Clone)]));

    item.attrs.push(parse_quote!(#[serde(deny_unknown_fields)]));

    item.vis = parse_quote!(pub);

    if let Fields::Named(fields) = &mut item.fields {
        process_fields(fields);
    }

    let name = &item.ident;

    quote! {
        #item

        impl crate::Named for #name {
            fn name(&self) -> &str {
                &self.name
            }

            fn name_mut(&mut self) -> &mut String {
                &mut self.name
            }
        }
    }
    .into()
}

fn process_fields(fields: &mut FieldsNamed) {
    let has_name = fields
        .named
        .iter()
        .flat_map(|f| f.ident.as_ref())
        .any(|i| i == "name");

    if !has_name {
        fields.named.push(parse_quote!(name: String));
    }

    for field in &mut fields.named {
        field.vis = parse_quote!(pub);
        let rename = find_rename(field);
        field.attrs.push(parse_quote!(#[serde(rename = #rename)]));
    }
}

fn find_rename(field: &mut Field) -> String {
    for i in 0..field.attrs.len() {
        let attr = &field.attrs[i];

        let Meta::NameValue(mnv) = &attr.meta else {
            continue;
        };

        if !mnv.path.get_ident().is_some_and(|ident| ident == "rename") {
            continue;
        }

        let Expr::Lit(lit) = &mnv.value else {
            continue;
        };

        let Lit::Str(s) = &lit.lit else {
            continue;
        };

        let rename = s.value();
        field.attrs.remove(i);
        return rename;
    }

    let ident = field.ident.as_ref().unwrap();

    if is_vec(&field.ty) {
        if let Some(singular) = ident.to_string().strip_suffix('s') {
            field.attrs.push(parse_quote!(#[serde(default)]));
            return singular.to_owned();
        }
    }

    format!("@{ident}")
}

fn is_vec(ty: &Type) -> bool {
    let Type::Path(ty) = ty else {
        return false;
    };

    let Some(last) = ty.path.segments.last() else {
        return false;
    };

    last.ident == "Vec"
}
