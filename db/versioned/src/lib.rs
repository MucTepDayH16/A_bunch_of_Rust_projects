use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Field, Fields, FieldsNamed, ItemStruct,
    LitInt,
};
use versioned_struct::VersionedStruct;

mod versioned_struct;

#[proc_macro_attribute]
pub fn versioned(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);

    match try_versioned(item) {
        Ok(stream) => stream,
        Err(error) => error.to_compile_error(),
    }
    .into()
}

fn try_versioned(
    ItemStruct {
        attrs,
        vis,
        struct_token: _,
        ident,
        generics,
        fields,
        semi_token,
    }: ItemStruct,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let (max_version, new_fields) = parse_fields(fields)?;

    let mut structs = Vec::new();
    let mut versions = Vec::new();
    let mut variants = Vec::new();

    let (impl_generics, trait_generics, where_clause) =
        generics.split_for_impl();

    let final_ident = ident.to_string() + "Final";
    let final_ident = Ident::new(&final_ident, proc_macro2::Span::mixed_site());

    for version in 0..=max_version {
        let versioned_fields = new_fields
            .iter()
            .filter(|(v, _)| &version >= v)
            .map(|(_, f)| f.clone())
            .collect::<Vec<_>>();

        let version_str = format!("V{version}");
        let version_ident =
            Ident::new(&version_str, proc_macro2::Span::mixed_site());

        let ident = ident.to_string() + &version_str;
        let ident = Ident::new(&ident, proc_macro2::Span::mixed_site());

        let versioned_struct = VersionedStruct {
            inner: ItemStruct {
                attrs: attrs.clone(),
                vis: vis.clone(),
                struct_token: Default::default(),
                ident: ident.clone(),
                generics: generics.clone(),
                fields: Fields::Named(FieldsNamed {
                    brace_token: Default::default(),
                    named: versioned_fields.into_iter().collect(),
                }),
                semi_token,
            },
            is_final: version == max_version,
            final_ident: final_ident.clone(),
        };

        structs.push(versioned_struct);
        versions.push(version_ident);
        variants.push(ident);
    }

    let last_ident = &variants[variants.len() - 1];

    Ok(quote! {
        #( #structs )*

        #vis type #final_ident #trait_generics = #last_ident #trait_generics ;

        #( #attrs )* #vis enum #ident #impl_generics #where_clause {
            #( #versions ( #variants #trait_generics ) , )*
        }

        #(
            impl #impl_generics ::core::convert::From<#variants #trait_generics> for #ident #trait_generics #where_clause {
                fn from(item: #variants #trait_generics) -> Self {
                    Self:: #versions (item)
                }
            }
        )*

        impl #impl_generics ::core::convert::TryFrom<#ident #trait_generics> for #final_ident #trait_generics #where_clause {
            type Error = ();

            fn try_from(item: #ident #trait_generics) -> ::core::result::Result<Self, ()> {
                match item {
                    #(
                        #ident :: #versions (item) => item.upgrade().ok_or(()),
                    )*
                }
            }
        }
    })
}

fn parse_fields(
    fields: Fields,
) -> Result<(usize, Vec<(usize, Field)>), syn::Error> {
    if let Fields::Named(named_fields) = fields {
        let mut new_fields = vec![];
        let mut max_version = 0;

        for mut field in named_fields.named {
            let version = find_v_attr(&mut field.attrs)?;
            if max_version < version {
                max_version = version;
            }
            new_fields.push((version, field));
        }

        Ok((max_version, new_fields))
    } else {
        Err(syn::Error::new_spanned(fields, "Not named structure"))
    }
}

fn find_v_attr(attrs: &mut Vec<Attribute>) -> Result<usize, syn::Error> {
    let mut version = 0;
    let mut maybe_idx = None;
    for (idx, attr) in attrs.iter_mut().enumerate() {
        if attr
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .as_deref()
            != Some("version")
        {
            continue;
        }

        version = attr.parse_args::<LitInt>()?.base10_parse::<usize>()?;
        maybe_idx = Some(idx);
    }

    if let Some(idx) = maybe_idx {
        attrs.remove(idx);
    }

    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields() {
        let item = syn::parse_str::<ItemStruct>(
            "
            #[derive(Clone, Debug, Serialize, Deserialize)]
            pub struct MessageValue<Timestamp> {
                pub msg: String,
                #[version(1)]
                pub timestamp: Timestamp,
            }
        ",
        )
        .unwrap();

        println!("{}", try_versioned(item).unwrap().to_string());
    }
}
