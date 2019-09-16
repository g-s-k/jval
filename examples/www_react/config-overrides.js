const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const wasmPkgDir = path.resolve(__dirname, "node_modules", "www");
const wasmExtensionRegExp = /\.wasm$/;

module.exports = function override(config, env) {
  config.plugins.push(
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "..", "www"),
      extraArgs: "--no-typescript",
      outDir: wasmPkgDir
    })
  );

  config.resolve.extensions.push(".wasm");

  config.module.rules.forEach(rule => {
    (rule.oneOf || []).forEach(oneOf => {
      if (oneOf.loader && oneOf.loader.indexOf("file-loader") >= 0) {
        // make file-loader ignore WASM files
        oneOf.exclude.push(wasmExtensionRegExp);
      }
    });
  });

  // add a dedicated loader for WASM
  config.module.rules.push({
    test: wasmExtensionRegExp,
    include: path.resolve(__dirname, "src"),
    use: [{ loader: require.resolve("wasm-loader"), options: {} }]
  });

  return config;
};
