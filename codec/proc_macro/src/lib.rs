use proc_macro::TokenStream;
use syn::*;

#[proc_macro_attribute]
pub fn call(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_impl = parse_macro_input!(input as ItemImpl);
    assert!(item_impl.trait_.is_some());
    item_impl.trait_ = None;
    item_impl.attrs = vec![];

    let self_ty = item_impl.self_ty.clone();
    let items = item_impl.items.clone();

    let mut idents = Vec::with_capacity(items.len());
    let mut names = Vec::with_capacity(items.len());
    let mut types = Vec::with_capacity(items.len());
    let mut nt = Vec::with_capacity(items.len());
    let mut blocks = Vec::with_capacity(items.len());

    for item in items {
        if let ImplItem::Method(method) = item {
            let mut pairs = method.sig.inputs.pairs();
            match pairs.next() {
                Some(origin) => {
                    if let FnArg::Typed(pat) = origin.value() {
                        if let Type::Path(path) = pat.ty.as_ref() {
                            let ident = path
                                .path
                                .segments
                                .last()
                                .unwrap()
                                .ident
                                .clone();
                            assert!(ident == "Origin");
                        }
                    } else {
                        panic!("Should have \"_: Origin\" argument")
                    }
                }
                _ => panic!("Should have \"_: Origin\" argument"),
            }
            let ident = method.sig.ident;
            idents.push(ident);
            let (new_names, new_types): (Vec<_>, Vec<_>) = pairs
                .filter_map(|pair| match pair.into_value().clone() {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(ty) => Some((ty.pat, *ty.ty)),
                })
                .unzip();
            types.push(quote::quote! { #( #new_types, )* });
            names.push(quote::quote! { #( #new_names, )* });
            nt.push(quote::quote! { #( #new_names : #new_types, )* });
            blocks.push(method.block);
        } else {
            panic!("Only methods are allowed");
        }
    }

    quote::quote! {
        #item_impl

        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[allow(non_camel_case_types)]
        pub enum Call {#(
            #idents (#types),
        )*}

        impl crate::call::Dispatchable for Call {
            fn dispatch(self, origin: crate::call::Origin) -> crate::call::DispatchResult {
                match self {#(
                    Call::#idents (#names) => #self_ty::#idents (origin, #names),
                )*}
            }
        }
    }
    .into()
}
