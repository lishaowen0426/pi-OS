use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use quote::quote;
use syn::{parse::Parser, parse_macro_input, Expr::Field, Ident, ItemFn, ItemStruct};

#[proc_macro_attribute]
pub fn kernel_test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);

    let test_name = &format!("{}", f.sig.ident);
    let test_ident = Ident::new(
        &format!("{}_TEST_CONTAINER", f.sig.ident.to_string().to_uppercase()),
        Span::call_site(),
    );
    let test_code_block = f.block;

    quote!(
        #[test_case]
        const #test_ident: test_types::UnitTest = test_types::UnitTest {
            name: #test_name,
            test_func: || #test_code_block,
        };
    )
    .into()
}

fn add_doubly_linkable(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let f = syn::Field::parse_named
        .parse2(quote! {pub doubly_link:DoublyLink<Self>})
        .unwrap();

    let mut item_struct = syn::parse2::<ItemStruct>(item).unwrap();

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(f);
    }

    quote!(#item_struct)
}

fn _impl_doubly_linkable(tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let item = syn::parse2::<ItemStruct>(tokens).unwrap();
    let mut found = false;
    for f in item.fields.iter() {
        if let Some(id) = &f.ident {
            if id.to_string().eq("doubly_link") {
                found = true;
                break;
            }
        }
    }

    if !found {
        abort_call_site!("Struct does not contain the field doubly_link");
    }

    let struct_name = item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    quote!(
        impl #impl_generics DoublyLinkable for #struct_name #ty_generics #where_clause{
            type T = Self;
            fn set_prev(&mut self, link: Link<Self::T>){
                self.doubly_link.prev = link;
            }
            fn set_next(&mut self, link: Link<Self::T>){
                self.doubly_link.next = link;
            }

            fn prev(&self) -> Link<Self::T>{
                self.doubly_link.prev
            }
            fn next(&self) -> Link<Self::T>{
                self.doubly_link.next
            }
        }
    )
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn doubly_linkable(_: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let ast = add_doubly_linkable(annotated_item.into());
    let impl_token = _impl_doubly_linkable(ast.clone());
    quote!(
        #ast
        #impl_token
    )
    .into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn impl_doubly_linkable(_: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let ast: proc_macro2::TokenStream = annotated_item.clone().into();
    let impl_token = _impl_doubly_linkable(annotated_item.clone().into());
    quote!(
        #ast
        #impl_token
    )
    .into()
}

fn _single_field_derive(item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let item_struct = syn::parse2::<ItemStruct>(item).unwrap();
    if item_struct.fields.len() != 1 {
        abort_call_site!("Not a single field struct");
    }
    let struct_name = item_struct.ident;

    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();
    let ty = &item_struct.fields.iter().next().unwrap().ty;

    quote!(
        impl #impl_generics From<#ty> for #struct_name #ty_generics #where_clause{
            fn from(value: #ty) -> Self{
                Self(value)
            }
        }
        impl #impl_generics Into<#ty> for #struct_name #ty_generics #where_clause{
            fn into(self) -> #ty{
                self.0
            }
        }

        impl #impl_generics fmt::Display for #struct_name #ty_generics #where_clause{
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>{
                write!(f, "{}", self.0)
            }
        }


    )
}
#[proc_macro_error]
#[proc_macro_derive(SingleField)]
pub fn single_field_derive(item: TokenStream) -> TokenStream {
    _single_field_derive(item.into()).into()
}

fn _impl_display_sync_send(tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let item_struct = syn::parse2::<ItemStruct>(tokens).unwrap();
    let struct_name = item_struct.ident;
    let fields = item_struct.fields;
    let struct_name_str = &format!("{}", struct_name);

    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();
    quote!(
        unsafe impl #impl_generics Sync for #struct_name #ty_generics #where_clause{
        }
        unsafe impl #impl_generics Send for #struct_name #ty_generics #where_clause{
        }

        impl #impl_generics fmt::Display for #struct_name #ty_generics #where_clause{
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>{
                write!(f, "{}", #struct_name_str)
            }
        }

    )
}
#[proc_macro_error]
#[proc_macro_attribute]
pub fn display_sync_send(_: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let ast: proc_macro2::TokenStream = annotated_item.clone().into();
    let impls = _impl_display_sync_send(annotated_item.clone().into());
    quote!(
        #ast
        #impls

    )
    .into()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_single_field() {
        _single_field_derive(quote!(
            pub struct Test(u64);
        ));
    }
}
