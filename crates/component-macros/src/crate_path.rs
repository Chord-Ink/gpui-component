use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

/// Resolve the GPUI API exposed to the crate where a macro is expanded.
///
/// `gpui-kit` is preferred because it re-exports GPUI and is the only direct
/// dependency required by kit consumers. The `gpui-pre` package fallback
/// preserves standalone `gpui-component` consumers, including dependencies
/// that rename that package to `gpui` (the conventional name), and the plain
/// `gpui` fallback covers a workspace pinned to a GPUI fork that keeps the
/// upstream package name.
pub(crate) fn gpui() -> syn::Result<TokenStream> {
    const PACKAGES: [&str; 3] = ["gpui-kit", "gpui-pre", "gpui"];

    let mut failures = Vec::new();
    for package in PACKAGES {
        match crate_name(package) {
            Ok(found) => return Ok(found_crate_path(found)),
            Err(error) => failures.push(format!("{package} lookup failed: {error}")),
        }
    }

    Err(syn::Error::new(
        Span::call_site(),
        format!(
            "IntoPlot requires a direct dependency on one of `gpui-kit`, `gpui-pre` or `gpui`: {}",
            failures.join("; ")
        ),
    ))
}

fn found_crate_path(found: FoundCrate) -> TokenStream {
    match found {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}
