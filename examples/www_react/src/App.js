import React from "react";

import { WasmProvider } from "./WasmContext";
import Form from "./Form";
import "./App.css";

function App() {
  return (
    <div className="App">
      <header>
        <h1>json validator / formatter</h1>
      </header>
      <WasmProvider>
        <Form />
      </WasmProvider>
      <footer>
        made by{" "}
        <a
          href="https://github.com/g-s-k"
          rel="noopener noreferrer"
          target="_blank"
        >
          george
        </a>
      </footer>
    </div>
  );
}

export default App;
