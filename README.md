# handlebars_misc_helpers <!-- omit in toc -->

[![crates license](https://img.shields.io/crates/l/handlebars_misc_helpers.svg)](http://creativecommons.org/publicdomain/zero/1.0/)
[![crate version](https://img.shields.io/crates/v/handlebars_misc_helpers.svg)](https://crates.io/crates/handlebars_misc_helpers)
[![Documentation](https://docs.rs/handlebars_misc_helpers/badge.svg)](https://docs.rs/handlebars_misc_helpers/)

[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)
[![Actions Status](https://github.com/davidB/handlebars_misc_helpers/workflows/ci-flow/badge.svg)](https://github.com/davidB/handlebars_misc_helpers/actions)
[![test coverage](https://codecov.io/gh/davidB/handlebars_misc_helpers/branch/master/graph/badge.svg)](https://codecov.io/gh/davidB/handlebars_misc_helpers)

A collection of helpers for handlebars (rust) to manage String, JSON, YAML, TOML, path, file, and HTTP requests.

Helpers extend the template to generate or transform content.
Few helpers are included, but if you need more helpers, ask via an issue or a PR.

To use a helper:

```handlebars
// arguments are space separated
{{ helper_name arguments}}
```

To chain helpers, use parenthesis:

```handlebars
{{ to_upper_case (to_singular "Hello foo-bars") }}
// -> BAR

{{ first_non_empty (unquote (json_str_query "package.edition" (read_to_str "Cargo.toml") format="toml")) (env_var "MY_VERSION") "foo" }}
// -> 2018
```

see [Handlebars templating language](https://handlebarsjs.com/)

To not "import" useless dependencies, use the crate's features:

```toml
default = ["string", "http_attohttpc", "json", "jsonnet", "regex", "uuid"]
http_attohttpc = ["dep:attohttpc"]
http_reqwest = ["dep:reqwest"]
json = [
    "dep:jmespath",
    "dep:serde",
    "dep:serde_json",
    "dep:serde_yaml",
    "dep:toml",
]
jsonnet = ["dep:jsonnet-rs"]
jsontype = ["dep:serde_json"]
regex = ["dep:regex"]
string = ["dep:cruet", "dep:enquote", "jsontype"]
uuid = ["dep:uuid"]
```

<!-- TOC depthFrom:2 -->

- [String transformation](#string-transformation)
- [Regular expression](#regular-expression)
- [UUID](#uuid)
- [HTTP content](#http-content)
- [Path extraction](#path-extraction)
- [File](#file)
- [Environment variable](#environment-variable)
- [JSON \& YAML \& TOML](#json--yaml--toml)
  - [Helpers](#helpers)
  - [Blocks](#blocks)
  - [Edition via Jsonnet](#edition-via-jsonnet)
- [Assign, set](#assign-set)
- [Replace section](#replace-section)

<!-- /TOC -->

## String transformation

| helper signature                         | usage sample                               | sample out            |
| ---------------------------------------- | ------------------------------------------ | --------------------- |
| `replace s:String from:String to:String` | `replace "Hello old" "old" "new"`          | `"Hello new"`         |
| `to_lower_case s:String`                 | `to_lower_case "Hello foo-bars"`           | `"hello foo-bars"`    |
| `to_upper_case s:String`                 | `to_upper_case "Hello foo-bars"`           | `"HELLO FOO-BARS"`    |
| `to_camel_case s:String`                 | `to_camel_case "Hello foo-bars"`           | `"helloFooBars"`      |
| `to_pascal_case s:String`                | `to_pascal_case "Hello foo-bars"`          | `"HelloFooBars"`      |
| `to_snake_case s:String`                 | `to_snake_case "Hello foo-bars"`           | `"hello_foo_bars"`    |
| `to_screaming_snake_case s:String`       | `to_screaming_snake_case "Hello foo-bars"` | `"HELLO_FOO_BARS"`    |
| `to_kebab_case s:String`                 | `to_kebab_case "Hello foo-bars"`           | `"hello-foo-bars"`    |
| `to_train_case s:String`                 | `to_train_case "Hello foo-bars"`           | `"Hello-Foo-Bars"`    |
| `to_sentence_case s:String`              | `to_sentence_case "Hello foo-bars"`        | `"Hello foo" bars`    |
| `to_title_case s:String`                 | `to_title_case "Hello foo-bars"`           | `"Hello Foo Bars"`    |
| `to_class_case s:String`                 | `to_class_case "Hello foo-bars"`           | `"HelloFooBar"`       |
| `to_table_case s:String`                 | `to_table_case "Hello foo-bars"`           | `"hello_foo_bars"`    |
| `to_plural s:String`                     | `to_plural "Hello foo-bars"`               | `"bars"`              |
| `to_singular s:String`                   | `to_singular "Hello foo-bars"`             | `"bar"`               |
| `to_foreign_key s:String`                | `to_foreign_key "Hello foo-bars"`          | `"hello_foo_bars_id"` |
| `demodulize s:String`                    | `demodulize "Test::Foo::Bar"`              | `"Bar"`               |
| `ordinalize s:String+`                   | `ordinalize "9"`                           | `"9th"`               |
| `deordinalize s:String+`                 | `deordinalize "9th"`                       | `"9"`                 |
| `trim s:String`                          | `trim " foo "`                             | `"foo"`               |
| `trim_start s:String`                    | `trim_start " foo "`                       | `"foo "`              |
| `trim_end s:String`                      | `trim_end " foo "`                         | `" foo"`              |
| `unquote s:String`                       | `unquote "\"foo\""`                        | `"foo"`               |
| `enquote symbol:String s:String`         | `enquote "" "foo"`                         | `"\"foo\""`           |
| `first_non_empty s:String+`              | `first_non_empty "" null "foo" "bar"`      | `"foo"`               |

## Regular expression

| usage                                                                                                                    | output                                                             |
| ------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------ |
| `{{ regex_is_match  pattern="(?<first>\\w)(\\w)(?:\\w)\\w(?<last>\\w)" on="today" }}`                                    | `true`                                                             |
| `{{#if (regex_is_match pattern="(?<first>\\w)(\\w)(?:\\w)\\w(?<last>\\w)" on="today" ) }}ok{{/if}}`                      | `ok`                                                               |
| `{{ regex_captures pattern="(?<first>\\w)(\\w)(?:\\w)\\w(?<last>\\w)" on="today" }}`                                     | `[object]`                                                         |
| `{{ json_to_str( regex_captures pattern="(?<first>\\w)(\\w)(?:\\w)\\w(?<last>\\w)" on="today" ) }}`                      | `{"_0":"today","_1":"t","_2":"o","_3":"y","first":"t","last":"y"}` |
| `{{ set captures=( regex_captures pattern="(?<first>\\w)(\\w)(?:\\w)\\w(?<last>\\w)" on="today" ) }}{{ captures.last }}` | `y`                                                                |

## UUID

| usage                     | output                                 |
| ------------------------- | -------------------------------------- |
| `{{ uuid_new_v4 }}`       | `6db4d8a7-8117-4b72-9dbc-988e6ee2a6e3` |
| `{{ len (uuid_new_v4) }}` | `36`                                   |
| `{{ uuid_new_v7 }}`       | `94d7bb75-9b16-40dd-878d-5fbb37b8ae2c` |
| `{{ len (uuid_new_v7) }}` | `36`                                   |

## HTTP content

The helpers can render the body's response from an HTTP request.

| helper signature                | usage sample                           |
| ------------------------------- | -------------------------------------- |
| `http_get url:String`           | `http_get "http://hello/..."`          |
| `gitignore_io templates:String` | `gitignore_io "rust,visualstudiocode"` |

## Path extraction

Helper able to extract (or transform) path (defined as string).

for the same input: `"/hello/bar/foo.txt"`

| helper_name | sample output  |
| ----------- | -------------- |
| file_name   | `"foo.txt"`    |
| parent      | `"/hello/bar"` |
| extension   | `"txt"`        |

## File

Helper to read file content.

| usage                                     | output                     |
| ----------------------------------------- | -------------------------- |
| `{{ read_to_str "/foo/bar" }}`            | content of file `/foo/bar` |
| `{{ read_to_str "file/does/not/exist" }}` | empty string               |

## Environment variable

The helper can get environment variables.

| helper_name | usage            |
| ----------- | ---------------- |
| env_var     | `env_var "HOME"` |

Some special environment variables are predefined (some of them come from [`std::env::consts` - Rust](https://doc.rust-lang.org/std/env/consts/index.html)):

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
        <tr>
          <td><code>"USERNAME"</code></td>
          <td>try to find the current username, in the order:<ol>
            <li>environment variable <code>"USERNAME"</code></li>
            <li>environment variable <code>"username"</code></li>
            <li>environment variable <code>"USER"</code></li>
            <li>environment variable <code>"user"</code></li>
          </ol></td>
        </tr>
    </tbody>
</table>

## JSON & YAML & TOML

### Helpers

- `json_query query:String data:Json`: Helper able to extract information from JSON using [JMESPath](http://jmespath.org/) syntax for `query`.
- `json_str_query query:String data:String`: Helper able to extract information from JSON using [JMESPath](http://jmespath.org/) syntax for `query`, data follows the requested `format`.
- `json_to_str data:Json`: convert JSON data into a string following the requested `format`.
- `str_to_json data:String`: convert(parse) a string into a JSON following the requested `format`.

The optional requested `format`, is the format of the string with data:

- `"json"` (default if omitted)
- `"json_pretty"` JSON with indentation,...
- `"yaml"`
- `"toml"`
- `"toml_pretty"`

| usage                                                                                              | output                          |
| -------------------------------------------------------------------------------------------------- | ------------------------------- |
| `{{ json_query "foo" {} }}`                                                                        | ``                              |
| `{{ json_to_str ( json_query "foo" {"foo":{"bar":{"baz":true}}} ) }}`                              | `{"bar":{"baz":true}}`          |
| `{{ json_to_str ( json_query "foo" (str_to_json "{\"foo\":{\"bar\":{\"baz\":true}}}" ) ) }}`       | `{"bar":{"baz":true}}`          |
| `{{ json_str_query "foo" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}`                                  | `{"bar":{"baz":true}}`          |
| `{{ json_str_query "foo.bar.baz" "{\"foo\":{\"bar\":{\"baz\":true}}}" }}`                          | `true`                          |
| `{{ json_str_query "foo" "foo:\n bar:\n  baz: true\n" format="yaml"}}`                             | `bar:\n  baz: true\n`           |
| `{{ json_to_str ( str_to_json "{\"foo\":{\"bar\":{\"baz\":true}}}" format="json") format="yaml"}}` | `foo:\n  bar:\n    baz: true\n` |

### Blocks

<table>
<tr>
<td><pre><code>{{#from_json format="toml"}}
{"foo": {"hello":"1.2.3", "bar":{"baz":true} } }
{{/from_json}}
</code></pre>
</td>
<td><pre><code>[foo]
hello = "1.2.3"

[foo.bar]
baz = true</code></pre></td>
</tr>
<tr>
<td><pre><code>{{#to_json format="toml"}}
[foo]
bar = { baz = true }
hello = "1.2.3"
{{/to_json}}</code></pre>
</td>
<td><pre><code>{
  "foo": {
    "bar": {
      "baz": true
    },
    "hello": "1.2.3"
  }
}</code></pre></td>
</tr>
<tr>
<td><pre><code>{{#from_json format="yaml"}}
{"foo":{"bar":{"baz":true}}}
{{/from_json}}</code></pre>
</td>
<td><pre><code>foo:
  bar:
    baz: true</code></pre></td>
</tr>
<tr>
<td><pre><code>{{#to_json format="yaml"}}
foo:
    bar:
        baz: true
{{/to_json}}</code></pre>
</td>
<td><pre><code>{
  "foo": {
    "bar": {
      "baz": true
    }
  }
}</code></pre></td>
</tr>
</table>

Note: YAML & TOML content are used as input and output formats for JSON data. So capabilities are limited to what JSON support (eg. no date-time type like in TOML).

### Edition via Jsonnet

For a more advanced edition of JSON, you can use Jsonnet.

> A data templating language for app and tool developers

- See the doc of [jsonnet](https://jsonnet.org/learning/tutorial.html) for more samples, and syntax info,...
- This block can be combined with conversion helper/block for YAML & TOML to provide edition capabilities for those format
- the output should be a valid JSON, except if `string_output = false` is set as a parameter of the block.

<table>
<tr>
<td><pre><code>{{#jsonnet}}
local v = {"foo":{"bar":{"baz":false}}};
v {
  "foo" +: {
      "bar" +: {
          "baz2": true
      }
  }
}
{{/jsonnet}}</code></pre>
</td>
<td><pre><code>{
  "foo": {
      "bar": {
          "baz": false,
          "baz2": true
      }
  }
}</code></pre></td>
</tr>
</table>

## Assign, set

The helpers can assign a variable to use later in the template.

⚠️ `assign` is deprecated and replaced by `set` (more compact and allows multiple assignments in one call)

| usage                                                             | output          |
| ----------------------------------------------------------------- | --------------- |
| `{{ assign "foo" "hello world" }}{{ foo }}`                       | `hello world`   |
| `{{ set foo="{}" }}`                                              | ``              |
| `{{ set foo="{}" }}{{ foo }}`                                     | `{}`            |
| `{{ set foo="hello world" }}{{ foo }}`                            | `hello world`   |
| `{{ set foo={} }}{{ foo }}`                                       | `[object]`      |
| `{{ set foo={"bar": 33} }}{{ foo }}`                              | `[object]`      |
| `{{ set foo={"bar": 33} }}{{ foo.bar }}`                          | `33`            |
| `{{ set foo="world" bar="hello" }}>{{ bar }} {{ foo }}<`          | `>hello world<` |
| `{{ set foo="world" }}{{ set bar="hello" }}>{{ bar }} {{ foo }}<` | `>hello world<` |

## Replace section

This helper can replace a section delimited by a boundary.

For example with this template:

```handlebars
{{~#replace_section  begin="<!-- #region head-->" end="<!-- #endregion head -->" content }}
This is the new content of the block
{{~/replace_section}}
```

The `content` having

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Document</title>
  </head>
  <body>
    <!-- #region head-->
    Something by default
    <!-- #endregion head -->
  </body>
</html>
```

The content between `<!-- #region head-->` and `<!-- #endregion head -->` is replaced by the result of the inner template:

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Document</title>
  </head>
  <body>
    <!-- #region head-->This is the new content of the block<!-- #endregion head -->
  </body>
</html>
```

Note: you can remove the boundary by adding `remove_boundaries=true`.
