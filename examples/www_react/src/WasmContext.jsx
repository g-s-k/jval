import React, { createContext, useEffect, useMemo, useState } from "react";

const DEFAULT_WASM = {
  format_packed() {},
  format_spaces() {},
  format_tabs() {},
  validate() {}
};

const WasmContext = createContext(DEFAULT_WASM);

export function WasmProvider({ children }) {
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

export default WasmContext;
