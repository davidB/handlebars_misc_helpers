# handlebars_misc_helpers

[![Crates.io](https://img.shields.io/crates/l/handlebars_misc_helpers.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![Crates.io](https://img.shields.io/crates/v/handlebars_misc_helpers.svg)](https://crates.io/crates/ffizer)

[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)
[![Build Status](https://dev.azure.com/davidbernard31/handlebars_misc_helpers/_apis/build/status/davidB.handlebars_misc_helpers?branchName=master)](https://dev.azure.com/davidbernard31/handlebars_misc_helpers/_build/latest?definitionId=5&branchName=master)
[![Documentation](https://docs.rs/handlebars_misc_helpers/badge.svg)](https://docs.rs/handlebars_misc_helpers/)

A collection of helpers for handlebars (rust).

Helpers extend the template to generate or to transform content.
Few helpers are included, but if you need more helpers, ask via an issue or a PR.

To use an helper:

```handlebars
{{ helper_name argument}}
```

To chain helpers, use parenthesis:

```handlebars
{{ to_upper_case (to_singular "Hello foo-bars") }}
// -> "BAR"
```

see [Handlebars templating language](https://handlebarsjs.com/)

## String transformation

for the same input: `"Hello foo-bars"`

helper_name | example out
-- | --
`to_lower_case` | `"hello foo-bars"`
`to_upper_case` | `"HELLO FOO-BARS"`
`to_camel_case` | `"helloFooBars"`
`to_pascal_case` | `"HelloFooBars"`
`to_snake_case` | `"hello_foo_bars"`
`to_screaming_snake_case` | `"HELLO_FOO_BARS"`
`to_kebab_case` | `"hello-foo-bars"`
`to_train_case` | `"Hello-Foo-Bars"`
`to_sentence_case` | `"Hello foo bars"`
`to_title_case` | `"Hello Foo Bars"`
`to_class_case` | `"HelloFooBar"`
`to_table_case` | `"hello_foo_bars"`
`to_plural` | `"bars"`
`to_singular` | `"bar"`

## Http content

Helper able to render body response from an http request.

helper_name | usage
-- | --
`http_get` | `http_get "http://hello/..."`
`gitignore_io` | `gitignore_io "rust"`

## Path extraction

Helper able to extract (or transform) path (defined as string).

for the same input: `"/hello/bar/foo.txt"`

helper_name | sample output
-- | --
file_name | `"foo.txt"`
parent | `"/hello/bar"`
extension | `"txt"`

## Environment variable

Helper able to get environment variables.

helper_name | usage
-- | --
env_var | `env_var "HOME"`

Specials environment variables are predefined (from [std::env::consts - Rust](https://doc.rust-lang.org/std/env/consts/index.html)):

<table>
    <thead>
        <tr>
            <th>variable</th>
            <th>possible values</th>
        </tr>
    </thead>
    <tbody>
        <tr><td><code>"ARCH"</code></td><td><ul>
            <li>x86</li>
            <li>x86_64</li>
            <li>arm</li>
            <li>aarch64</li>
            <li>mips</li>
            <li>mips64</li>
            <li>powerpc</li>
            <li>powerpc64</li>
            <li>s390x</li>
            <li>sparc64</li>
        </ul></td></tr>
        <tr><td><code>"DLL_EXTENSION"</code></td><td><ul>
            <li>so</li>
            <li>dylib</li>
            <li>dll</li>
        </ul></td></tr>
        <tr><td><code>"DLL_PREFIX"</code></td><td><ul>
            <li>lib</li>
            <li>"" (an empty string)</li>
        </ul></td></tr>
        <tr><td><code>"DLL_SUFFIX"</code></td><td><ul>
            <li>.so</li>
            <li>.dylib</li>
            <li>.dll</li>
        </ul></td></tr>
        <tr><td><code>"EXE_EXTENSION"</code></td><td><ul>
            <li>exe</li>
            <li>"" (an empty string)</li>
        </ul></td></tr>
        <tr><td><code>"EXE_SUFFIX"</code></td><td><ul>
            <li>.exe</li>
            <li>.nexe</li>
            <li>.pexe</li>
            <li>"" (an empty string)</li>
        </ul></td></tr>
        <tr><td><code>"FAMILY"</code></td><td><ul>
            <li>unix</li>
            <li>windows</li>
        </ul></td></tr>
        <tr><td><code>"OS"</code></td><td><ul>
            <li>linux</li>
            <li>macos</li>
            <li>ios</li>
            <li>freebsd</li>
            <li>dragonfly</li>
            <li>netbsd</li>
            <li>openbsd</li>
            <li>solaris</li>
            <li>android</li>
            <li>windows</li>
        </ul></td></tr>
    </tbody>
</table>


## JSON

Helper able to extract information from json using [JMESPath](http://jmespath.org/) syntax.

usage| output
-- | --
`{{ json_query "foo" "{}" }}` | `null`
`{{ json_query "foo" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}` | `{"bar":{"baz":true}}`
`{{ json_query "foo.bar.baz" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}` | `true`

## Assign

Helper able to assign (to set) a variable to use later in the template.

usage| output
-- | --
`{{ assign "foo" "{}" }}` | ``
`{{ assign "foo" "{}" }}{{ foo }}` | `{}`
`{{ assign "foo" "hello world" }}{{ foo }}` | `hello world`
`{{ assign "foo" {} }}{{ foo }}` | `[object]`
`{{ assign "foo" {"bar": 33} }}{{ foo }}` | `[object]`
