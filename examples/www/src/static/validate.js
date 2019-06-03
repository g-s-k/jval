import { default as init, validate } from "./www.js";

(async function run() {
  await init("./www_bg.wasm");

  const entryBox = document.getElementById("json-entry-box");
  const errorDisp = document.getElementById("error-display-box");

  entryBox.addEventListener("input", function handleChange({
    target: { value = "" }
  }) {
    const v = validate(value);

    if (v) {
      const { start, end } = v;
      const msg = "error between " + start + " and " + end + ".";
      errorDisp.innerText = msg;
      entryBox.setCustomValidity(msg);
    } else {
      errorDisp.innerText = "good!";
      entryBox.setCustomValidity("");
    }
  });
})();
