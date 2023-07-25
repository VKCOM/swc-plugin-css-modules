# swc-plugin-css-modules

A [SWC](https://swc.rs) plugin to use [CSS Modules](https://github.com/css-modules/css-modules).

Instead of creating an object, the plugin injects the css class names directly into the js code.
This allows to reduce the size of the bundle.

```js
// Input Code
import styles from "./style.css";
element.innerHTML = '<div class="' + styles.className + '">';

// Output   ↓ ↓ ↓ ↓ ↓ ↓
import "./style.css";
element.innerHTML = '<div class="' + "style__className___clZD5" + '">';
```

## Installation

**npm:**

```sh
npm i -D swc-plugin-css-modules
```

**yarn:**

```sh
yarn add -D swc-plugin-css-modules
```

## Usage

Via `.swcrc`

```json
{
  "jsc": {
    "experimental": {
      "plugins": [
        [
          "swc-plugin-css-modules",
          {
            "generate_scoped_name": "[name]__[local]___[hash:base64:5]"
          }
        ]
      ]
    }
  }
}
```

Using css modules in code:

```js
import styles from "./style.module.css";

// ✅ ok
const className = styles.className;

// ✅ ok
const className = styles["class-name"];

// ⛔ Computed hit cannot be injected
const className = styles["class" + "Name"];

// ⛔ Computed hit cannot be injected
const className = styles[localClassName];

// ✅ ok
const className = classNames(styles.title, styles.className);

// ⛔ Computed hit cannot be injected
const className = getClassNameFromCssModules(styles);
```

## Options

### `generate_scoped_name`

Default: `"[hash:base64]"`

Allows to configure the generated local ident name.

Supported template strings:

- `[name]` the basename of the resource
- `[folder]` the folder the resource relative
- `[ext]` - extension with leading
- `[hash]` - the hash of the string(by default it's the `hex` digest of the `xxhash64` hash)
- `[<hashFunction>:hash:<hashDigest>:<hashDigestLength>]` - hash with hash settings
- `[local]` - original class

Supported hash functions:

- `xxhash64`
- `md4`
- `md5`
- `sha1`
- `sha224`
- `sha256`
- `sha384`
- `sha512`

Supported hash digests:

- `hex`
- `base32`
- `base64`

### `hash_prefix`

Add custom hash prefix to generate more unique classes.

### `css_modules_suffix`

Default: `".css"`

### `root`

If you need, you can pass any needed root path.
