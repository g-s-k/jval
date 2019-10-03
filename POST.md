# Write Once(-ish), Run Anywhere(-ish) with Rust and WebAssembly

> Tips for deploying the same functionality on the web, the desktop, and the command line

A popular aphorism in the software industry is "Don't Repeat Yourself" (often
abbreviated "DRY"). While there is hardly a consensus about just how much
repetition is appropriate, it is likely that building your core functionality
from scratch on each platform you want to deploy to is too much from a business
or engineering perspective.

If your platforms all support the same language or runtime (and you want to use
that language or runtime), sharing code can be simple. A great example of this
is JavaScript - not only is it the native language of the web, but it can (and
is!) used to build command-line interfaces with Node and desktop applications
with Electron. You can even use other languages that compile to JavaScript in
its place for a different development experience. Some other examples, to
varying degrees, are the JVM and the .NET CLR, each of which supports multiple
platforms and provides developers with a wide selection of languages to target
them.

If this is not the case, it is still possible to share code between platforms
using [FFI](https://en.wikipedia.org/wiki/Foreign_function_interface). This is
frequently done by writing the core library in C or C++ and providing a safe
wrapper for each platform. For desktop and mobile applications, this would be
distributed as a dynamically linked library (e.g. a `.dll` file on Windows or a
`.so` file on Unix-like systems) or statically linked into the binary. For web
applications, modern browsers support an executable format called WebAssembly
(WASM), which can expose values and procedures to the JavaScript runtime (and
vice versa).

While FFI is generally effective, it can make debugging painful, slow down
development cycles, and create opportunities for a whole host of avoidable bugs
to creep into your product. However, depending on the platforms you want to
target, it can provide a performance boost - especially appreciated for
resource-intensive applications like games or media editors.

There is another option - writing your library in a language flexible enough to
build a whole application in, but with a toolchain that allows you to use FFI on
platforms that require a specific language or runtime...Rust!

### Building your shared library

As an example of some library functionality, I wrote [a simple JSON validator
and formatter](https://github.com/g-s-k/jval). The actual functionality is not
so important, but here are some basic tips on what should and shouldn't go in
yours:

- Scope your shared library as narrowly as possible. Don't be afraid to split it
  into several independent crates if there are orthogonal, loosely coupled (or
  even better, completely disjoint) pieces.

- Limit platform-specific code, especially I/O and networking, as much as you
  can. While Rust does support conditional compilation, it can make code pretty
  hard to follow when the flow of control is interrupted and reorganized with
  `#[cfg]`s. For example, instead of using the `println!` macro, take a `mut`
  reference to some type implementing `std::io::Write` and use `writeln!`. Your
  consumer applications can then pass in a reference to `Stdout`, a `File`, or
  something else at their discretion.

- Keep your dependency count low. Many libraries are cross-platform, but some
  have limited support or expect certain primitives to exist.

  - A common reason for platform specificity in dependencies is threading. Some
    crates depend on `rayon` or `crossbeam`, which cannot be compiled for WASM
    targets. A silver lining here is that some of these crates (e.g. `image`)
    provide [`cargo` features](https://doc.rust-lang.org/cargo/appendix/glossary.html#feature)
    to turn off threading.

### Consuming your library

Once you have your core library written (or at least stubbed out), you can build
clients to consume it. For this example, we will build three (in order of
increasing complexity): a command-line interface, a desktop application, and a
static web page. Each of these has an basic implementation inside the
[`examples` directory](https://github.com/g-s-k/jval/tree/master/examples) in
the GitHub repository linked above.

#### A command-line client

It's not much of a stretch to write a CLI for a library. The main points to
focus on are:

- Input
  - How will you handle arguments?
  - Will you support Unix pipes?
  - What about configuration?

- Output
  - What format should your output have? Do you want to support multiple
    formats?
  - Logging: stdout or stderr? Dedicated log files?

- Help
  - How should you format the `--help` output?
  - Will you write a man page?
  
Thankfully, several of the above questions are answered by the venerable
[`structopt` crate](https://docs.rs/structopt/). Using a proc-macro and
attributes, it can turn a Rust data structure into an argument parser and
validator, and provide you with nicely formatted help output. Some helpful tips
for using `structopt`:

- If you need to break your configuration into multiple data types for
  convenience, you can use the `#[structopt(flatten)]` item attribute to make a
  whole struct's fields act as if they were inline in another.

- Use `#[structopt(parse(from_os_str))]` and type `PathBuf` for file paths.

- Use `#[structopt(requires = "..."')]` to make an argument or flag's validity
  depend on another one being passed in.

- For colorful help text, use the `#[structopt(raw(global_setting =
  "structopt::clap::AppSettings::ColoredHelp"))]` attribute on the struct.
  
For other I/O questions, [the standard library's `io`
module](https://doc.rust-lang.org/std/io/) is a great place to start. If you
plan on reading more than a few lines of text from stdin or writing more than
that to stdout or another file, consider [buffered
I/O](https://doc.rust-lang.org/std/io/#bufreader-and-bufwriter).

#### A desktop client

The possibilities begin opening up here. There are many established UI
frameworks for the desktop, and choosing the right one for your application (if
you build a desktop app at all!) is highly context-dependent. For example, a
Windows-only application could consider the `winapi` or `abscissa` crates, and a
MacOS-only application could consider the `cocoa` crate. Larger, more ambitious
projects might be interested in `rust-qt`. For our purposes, however, we are
going to consider `gtk-rs` as GTK+ has the most extensive support in Rust at the
time of writing.

One library that caught my eye while researching for this article was
[relm](https://github.com/antoyo/relm), which is a runtime for `gtk-rs` that
tries to emulate the [Elm
Architecture](https://guide.elm-lang.org/architecture/). In practice, it feels
much more satisfying and clean to work with than the imperative style encouraged
by the basic GTK+ bindings. As a spoiled React developer, however, it still did
not deliver the level of comfort that I'm used to. 

##### Free side project idea

> Glade XML-in-Rust just like JSX for React

At least for my toy example, the `relm` and `gtk-rs` documentation made things
easy. My only tip for desktop app development is to give them a try.

#### A static web page: take one

One thing you might notice while exploring the example repository is that there
are actually two static site implementations included. At first, I wrote a very
simple, vanilla JS version to show just how easy it is to integrate WebAssembly
into a static page. Later on, I added a (mostly) identical one built using
`create-react-app` to demonstrate how to bundle a local WASM dependency with
`webpack`.

In the vanilla JS implementation, take a look in the [`static`
directory](https://github.com/g-s-k/jval/tree/master/examples/www/src/static)
for inspiration. The high-level control flow is like so:

1. The HTML file (`examples/www/src/static/index.html`) sets up the document
   structure and gives IDs to the interactive elements. No elements will need to
   be added or removed dynamically. It then includes the JS file with a `script` tag.

2. The JS file (`examples/www/src/static/validate.js`) `import`s the
   automatically generated bindings and initializes the WebAssembly before doing
   its work. Once the WASM is fetched, compiled, and ready to go, it connects
   callbacks and event listeners to the buttons and the textarea to validate the
   input and display errors.
   
##### Ensure that the WebAssembly code is in place before hooking up the UI with a good ol' IIFE

```javascript
import init, { ... } from "./example.js";

(async function run() {
  try {
    // fetch and compile the WASM
    await init("./example.wasm");
  } catch (e) {
    console.error("Failed to initialize WASM dependency", e);
    return;
  }

  // do everything else
  ...
})()
```

3. In the parent directory (`examples/www/src`) are the FFI bindings for the
   library. Thanks to
   [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen), they're pretty
   minimal. All we needed to define were four functions and a single type, in
   just over 50 lines of code! There are [some
   limitations](https://rustwasm.github.io/docs/wasm-bindgen/reference/types.html)
   on what data types can pass over the FFI boundary, but all that's really
   needed here are strings, unsigned integers (for character indices), and
   optional values.

#### A static web page: take two

For the React implementation, things got a bit more complicated. That isn't to
say that it's too difficult to be worth it, but just that you need to be aware
of the pitfalls waiting for you. Special credit for this section goes to Preston
Richey for [this extremely helpful blog
post](https://prestonrichey.com/blog/react-rust-wasm/) that got me started.

If you use `create-react-app`, you are probably familiar with the limitations of
the `webpack` configuration provided by `react-scripts`. The official way of
dealing with these limitations is to "eject" and edit the configuration to your
liking, then maintain it for the rest of eternity.

An alternative that I learned about while researching for this article is called
[`react-app-rewired`](https://github.com/timarney/react-app-rewired). Apparently
it does not work for CRA 2+, but it worked fine for me here, so YMMV. What it
(and similar projects) allows you to do is provide custom code to amend the
`webpack` configuration your app uses. If you already manage your own build
configuration, it should be pretty easy to follow along.

The first change needed was to make WebAssembly binary files `import`-able from JS files.

```javascript
config.resolve.extensions.push(".wasm");
```

The next was to set up the loaders correctly. This required two steps: first,
make sure `file-loader` won't accept WASM files, then add a specific loader that
knows how to handle it. Before you add this configuration, make sure you install
`wasm-loader` as a `devDependency`.

```javascript
config.module.rules.forEach(rule => {
  (rule.oneOf || []).forEach(oneOf => {
    if (oneOf.loader && oneOf.loader.indexOf("file-loader") >= 0) {
      // make file-loader ignore WASM files
      oneOf.exclude.push(/\.wasm$/);
    }
  });
});

// add a dedicated loader for WASM
config.module.rules.push({
  test: /\.wasm$/,
  include: path.resolve(__dirname, "src"),
  use: [{ loader: require.resolve("wasm-loader"), options: {} }]
});
```

Finally, we can make our lives much easier (and obviate the `npm link` step from
the blog post linked above) if we build our Rust crate as part of the `webpack`
build process. We can achieve this using the
[`wasm-pack-plugin`](https://github.com/wasm-tool/wasm-pack-plugin). Make sure
that you substitute the `crateDirectory` key appropriately for your project, and
set the `outDir` to `node_modules/<crate name from Cargo.toml here>`.

```javascript
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

config.plugins.push(
  new WasmPackPlugin({
    crateDirectory: path.resolve(__dirname, "..", "www"),
    extraArgs: "--no-typescript",
    outDir: path.resolve(__dirname, "node_modules", "www")
  })
);
```

Now that we have all of the configuration set up, let's take a look at the React
code! There are multiple ways to handle this, but in this case I decided to
handle the WASM with a [Context](https://reactjs.org/docs/context.html). The
downsides of this approach are listed in the React documentation, and some of
them sound scary. However, Context is a great way to avoid passing props down
multiple levels and to reduce render overhead. It also lets you isolate your
implementation of deriving an expensive or persistent value (e.g. a WASM module,
a WebSocket connection, a large data structure) and give components access to it
throughout the tree.

Here is a bare-bones implementation of a Provider that fetches the WASM file and
lets its children access the functions it defines:

```javascript
import React, { createContext, useEffect, useMemo, useState } from "react";

const DEFAULT_WASM = {
  format_packed() {},
  format_spaces() {},
  format_tabs() {},
  validate() {}
};

const WasmContext = createContext({...DEFAULT_WASM, loading: false});

function WasmProvider({ children }) {
  const [wasm, setWasm] = useState(DEFAULT_WASM);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    (async () => {
      setLoading(true);
      try {
        setWasm(await import("www"));
      } catch (err) {
        console.error("Failed to load wasm: " + err.message);
      } finally {
        setLoading(false);
      }
    })();
  }, []);

  const value = useMemo(() => ({ ...wasm, loading }), [loading, wasm]);

  return <WasmContext.Provider value={value}>{children}</WasmContext.Provider>;
}
```

Now any component inside of this Provider can reference one of the functions it
loads (snippet from `examples/www_react/src/Form/Format.jsx`):

```javascript
  ...

  const { format_packed, format_spaces, format_tabs } = useContext(WasmContext);

  const formatText = useMemo(
    () => ({ text, errors }) => {
      if (errors || !text) return text;

      switch (style) {
        case NONE:
          return format_packed(text);
        case TAB:
          return format_tabs(text);
        case SPACE:
        default:
          return format_spaces(text, number);
      }
    },
    [format_packed, format_spaces, format_tabs, number, style]
  );
  
  ...
```

### Next steps

Now that you know the basics of deploying a library on multiple platforms, there
are still more options you can explore. One example is WebAssembly support in
V8 - this means you can package WASM in an Electron app or as part of a larger
CLI written mainly in JavaScript. Another, more experimental area to check out
are pure-Rust web frameworks - my personal favorite is called
[`yew`](https://github.com/yewstack/yew).

#### Mobile apps

You may have noticed that I failed to include mobile applications in this guide.
That's because I don't know how to build mobile apps! That doesn't mean it can't
be done - if you're curious about how to integrate Rust code into a mobile
application, here are some resources to get you started:

- [Building a cryptocurrency wallet app for Android with Rust](https://medium.com/@marekkotewicz/building-a-mobile-app-in-rust-and-react-native-part-1-project-setup-b8dbcf3f539f)
- [Basic guide to Rust for iOS](https://medium.com/visly/rust-on-ios-39f799b3c1dd)
- [Resources for building mobile apps in Rust](https://github.com/Geal/rust_on_mobile) (may be out of date)
