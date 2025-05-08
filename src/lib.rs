extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Expr, Ident, LitStr, Token,
};

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use core::panic;

struct I18nInput {
    file: LitStr,
    fallback: LitStr,
}

impl Parse for I18nInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let file: LitStr = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let fallback_ident: Ident = input.parse()?;
        if fallback_ident.to_string() != "fallback" {
            return Err(input.error("Expected 'fallback'"));
        }
        let _eq: Token![=] = input.parse()?;
        let fallback: LitStr = input.parse()?;
        Ok(I18nInput { file, fallback })
    }
}

#[proc_macro]
pub fn i18n(input: TokenStream) -> TokenStream {
    let I18nInput { file, fallback } = parse_macro_input!(input as I18nInput);
    let file_name = file.value();

    let file_name_with_ext = if file_name.ends_with(".yaml") {
        file_name.clone()
    } else {
        format!("{}.yaml", file_name)
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR env variable not set");

    let file_path = PathBuf::from(manifest_dir).join(&file_name_with_ext);

    let content = fs::read_to_string(&file_path)
        .unwrap_or_else(|_| panic!("Could not read translation file: {:?}", file_path));

    let raw_yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .unwrap_or_else(|e| panic!("Invalid YAML format in {}: {}", file_path.display(), e));

    let mut translations: HashMap<String, HashMap<String, String>> = HashMap::new();
    if let serde_yaml::Value::Mapping(map) = raw_yaml {
        for (key, value) in map {
            if let serde_yaml::Value::String(key_str) = key {
                if key_str.starts_with('_') {
                    continue;
                }
                if let serde_yaml::Value::Mapping(locale_map) = value {
                    let mut inner_map = HashMap::new();
                    for (locale, translation) in locale_map {
                        if let (serde_yaml::Value::String(locale_str), serde_yaml::Value::String(translation_str)) =
                            (locale, translation)
                        {
                            inner_map.insert(locale_str, translation_str);
                        }
                    }
                    translations.insert(key_str, inner_map);
                }
            }
        }
    } else {
        panic!("Expected YAML top-level mapping in {}", file_path.display());
    }

    let fallback_locale = fallback.value();

    let mut key_match_arms = vec![];
    for (key, locale_map) in translations.iter() {
        let mut locale_match_arms = vec![];
        for (locale, translation) in locale_map.iter() {
            let locale_literal = LitStr::new(locale, Span::call_site());
            let translation_literal = LitStr::new(translation, Span::call_site());
            locale_match_arms.push(quote! {
                #locale_literal => #translation_literal,
            });
        }
        let fallback_branch = if let Some(default_translation) = locale_map.get(&fallback_locale) {
            let default_translation_literal = LitStr::new(default_translation, Span::call_site());
            quote! {
                _ => #default_translation_literal,
            }
        } else {
            quote! {
                _ => panic!("Translation for key '{}' not found in the requested locale and fallback locale '{}'", #key, #fallback_locale),
            }
        };
        locale_match_arms.push(fallback_branch);
        let key_literal = LitStr::new(key, Span::call_site());
        key_match_arms.push(quote! {
            #key_literal => {
                match norm_locale {
                    #(#locale_match_arms)*
                }
            }
        });
    }

    let mut available_locales_set = HashSet::new();
    for (_key, locale_map) in translations.iter() {
        for locale in locale_map.keys() {
            available_locales_set.insert(locale.clone());
        }
    }
    let mut available_locales_vec: Vec<String> = available_locales_set.into_iter().collect();

    // sort it, doesn't need to be stable
    available_locales_vec.sort_unstable();

    let available_locales_tokens = available_locales_vec.iter().map(|loc| {
        LitStr::new(loc, Span::call_site())
    });

    let default_key_arm = quote! {
        _ => panic!("Translation key not found: {}", key),
    };

    let expanded = quote! {
        pub mod __i18n_internal {
            pub const FALLBACK: &str = #fallback_locale;

            pub fn translate(key: &str, locale: &str) -> &'static str {
                // Normalize the locale: take only the prefix before '-' (e.g. "en-US" becomes "en")
                let norm_locale = if let Some(idx) = locale.find('-') {
                    &locale[..idx]
                } else {
                    locale
                };
                match key {
                    #(#key_match_arms),*,
                    #default_key_arm
                }
            }

            pub fn available_locales() -> &'static [&'static str] {
                &[#( #available_locales_tokens ),*]
            }
        }
    };

    TokenStream::from(expanded)
}

struct TInput {
    key: Expr,
    locale: Expr,
}

impl Parse for TInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let key: Expr = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let ident: Ident = input.parse()?;
        if ident.to_string() != "locale" {
            return Err(input.error("Expected 'locale'"));
        }
        let _eq: Token![=] = input.parse()?;
        let locale: Expr = input.parse()?;
        Ok(TInput { key, locale })
    }
}

/**
 * t as in translate. Usage: t!(<key>, locale=<locale>).
 */
#[proc_macro]
pub fn t(input: TokenStream) -> TokenStream {
    let TInput { key, locale } = parse_macro_input!(input as TInput);
    let expanded = quote! {
        crate::__i18n_internal::translate((#key).as_ref(), (#locale).as_ref())
    };
    TokenStream::from(expanded)
}

/**
 * Get all locales found in the translation file. Returns a sorted &[&str].
 */
#[proc_macro]
pub fn available_locales(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        crate::__i18n_internal::available_locales()
    };
    TokenStream::from(expanded)
}
