use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(CheapComponent)]
pub fn cheap_component_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        impl CheapComponent for #name {
        }
        impl GenericComponent for #name {
            fn mewo_component_duplicate() -> ValueDuplicate {
                <#name as CheapComponent>::mewo_component_duplicate()
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(Component)]
pub fn component_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        impl Component for #name {
        }
        impl GenericComponent for #name {
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
    let gen = quote! {
        impl UniqueComponent for #name {
        }
        impl GenericComponent for #name {
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
    let gen = quote! {
        impl Event for #name {
        }
    };
    gen.into()
}

#[proc_macro_derive(Resource)]
pub fn resource_macro_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let gen = quote! {
        impl Resource for #name {
        }
    };
    gen.into()
}
