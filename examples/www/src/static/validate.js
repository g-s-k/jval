import { default as init, validate } from "./www.js";

const emptyIndices = { start: null, end: null };

(async function run() {
  await init("./www_bg.wasm");

  let errorIndices = emptyIndices;

  const entryBox = document.getElementById("json-entry-box");
  const errorDisp = document.getElementById("error-display-box");
  const gotoError = document.getElementById("goto-error-btn");

  gotoError.addEventListener("click", function handleGotoClick() {
    entryBox.selectionStart = errorIndices.start;
    entryBox.selectionEnd = errorIndices.end;
    entryBox.focus();
  });

  entryBox.addEventListener("input", function handleChange({
    target: { value = "" }
  }) {
    const v = validate(value);

    if (v) {
      const { start, end } = v;
      const msg = "error between " + start + " and " + end + ".";
      errorDisp.innerText = msg;
      entryBox.setCustomValidity(msg);
      gotoError.disabled = false;
      errorIndices = { start, end };
    } else {
      errorDisp.innerText = "good!";
      entryBox.setCustomValidity("");
      gotoError.disabled = true;
      errorIndices = emptyIndices;
    }
  });
})();
