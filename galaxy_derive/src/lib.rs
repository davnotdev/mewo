use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(CheapComponent)]
pub fn cheap_component_macro_derive(input: TokenStream) -> TokenStream {
    let mut ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let generics = &mut ast.generics;
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(syn::parse_quote!(Copy));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics CheapComponent for #name #ty_generics #where_clause {
        }
        impl #impl_generics GenericComponent for #name #ty_generics #where_clause {
            fn mewo_component_duplicate() -> ValueDuplicate {
                <#name as CheapComponent>::mewo_component_duplicate()
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(Component)]
pub fn component_macro_derive(input: TokenStream) -> TokenStream {
    let mut ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let generics = &mut ast.generics;
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(syn::parse_quote!(Clone));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics Component for #name #ty_generics #where_clause {
        }
        impl #impl_generics GenericComponent for #name #ty_generics #where_clause {
            fn mewo_component_duplicate() -> ValueDuplicate {
                <#name as Component>::mewo_component_duplicate()
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(UniqueComponent)]
pub fn unique_component_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics UniqueComponent for #name #ty_generics #where_clause {
        }
        impl #impl_generics GenericComponent for #name #ty_generics #where_clause {
            fn mewo_component_duplicate() -> ValueDuplicate {
                <#name as UniqueComponent>::mewo_component_duplicate()
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(Event)]
pub fn event_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics Event for #name #ty_generics #where_clause {
        }
    };
    gen.into()
}

#[proc_macro_derive(Resource)]
pub fn resource_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics Resource for #name #ty_generics #where_clause {
        }
    };
    gen.into()
}

#[proc_macro_derive(SingleResource)]
pub fn single_resource_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics Resource for #name #ty_generics #where_clause {
        }
        impl #impl_generics #name #ty_generics {
            pub fn single_resource() -> std::any::TypeId {
                std::any::TypeId::of::<#name>()
            }
        }
    };
    gen.into()
}
