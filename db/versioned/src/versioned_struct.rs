use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{Fields, FieldsNamed, ItemStruct};

pub struct VersionedStruct {
    pub inner: ItemStruct,
    pub is_final: bool,
    pub final_ident: Ident,
}

impl ToTokens for VersionedStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ItemStruct {
            attrs,
            vis,
            struct_token,
            ident,
            generics,
            fields,
            semi_token,
        } = &self.inner;
        let (impl_generics, trait_generics, where_clause) =
            generics.split_for_impl();

        let Fields::Named(FieldsNamed { named, .. }) = fields else {
            unreachable!();
        };

        let fields = named.iter();

        quote! {
            #(#attrs)*
            #vis #struct_token #ident #impl_generics
            #where_clause {
                #( #fields , )*
                // pub _phantom: ::core::marker::PhantomData < #trait_generics_without_angles >,
            } #semi_token
        }
        .to_tokens(tokens);

        let final_ident = &self.final_ident;
        if !self.is_final {
            quote! {
                impl #impl_generics #ident #trait_generics #where_clause {
                    #vis fn upgrade(self) -> Option< #final_ident #trait_generics > {
                        < #final_ident #trait_generics as ::core::convert::TryFrom::< #ident #trait_generics > >::try_from(self).ok()
                    }
                }
            }
            .to_tokens(tokens);
        } else {
            quote! {
                impl #impl_generics #ident #trait_generics #where_clause {
                    #vis fn upgrade(self) -> Option< #final_ident #trait_generics > {
                        Some(self)
                    }
                }
            }
            .to_tokens(tokens);
        }
    }
}
