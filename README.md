# brutal_i18n

A nonchalant, disturbingly simplistic and brutal i18n library, directly inspired by rust-i18n.

## This is for you if:

- You don't care about features, you just want to replace text blocks at compile time.
- You have ISO-3166 or similar for keys, but don't care about any area codes.
- You don't mind copying the line `t!(<key>, locale=<locale>)` wherever you need to replace text blocks.
- You want a block of code that works and that you then edit yourself for more complex i18n needs.

## What's different in this one compared to rust-i18n (at the time of writing):

- Less features.
- Less code.
- Avoids heap allocations.
- Ignored area codes.
- Panics if a key is not found in the translation file.
- Works with Strings and str slices, and probably some other types that implement `as_ref`.

## What's the same compared to rust-i18n (at the time of writing):

- Replaces text blocks at compile time.
- Macros have similar functions and similar names.
- Supports a single YAML file that contains your replacement text blocks (translations).

## Examples
TODO
