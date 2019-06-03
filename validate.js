import {
  default as init,
  format_packed,
  format_spaces,
  format_tabs,
  validate
} from "./www.js";

const emptyIndices = { start: null, end: null };

(async function run() {
  await init("./www_bg.wasm");

  let errorIndices = emptyIndices;

  const entryBox = document.getElementById("json-entry-box");
  const errorDisp = document.getElementById("error-display-box");
  const gotoError = document.getElementById("goto-error-btn");
  const formatBtn = document.getElementById("format-json-btn");
  const formatInput = document.getElementById("format-json-spacing");
  const formatDropdown = document.getElementById("format-spacing-type");

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
      formatBtn.disabled = true;
      errorIndices = { start, end };
    } else {
      errorDisp.innerText = "good!";
      entryBox.setCustomValidity("");
      gotoError.disabled = true;
      formatBtn.disabled = false;
      errorIndices = emptyIndices;
    }
  });

  formatBtn.addEventListener("click", function handleFormat() {
    let formatted;
    switch (formatDropdown.value) {
      case "none":
        formatted = format_packed(entryBox.value);
        break;
      case "tab":
        formatted = format_tabs(entryBox.value);
        break;
      default:
        formatted = format_spaces(entryBox.value, parseInt(formatInput.value));
    }
    if (formatted) entryBox.value = formatted;
  });

  formatDropdown.addEventListener("change", function handleDropdown({
    target: { value }
  }) {
    formatInput.disabled = value !== "space";
  });
})();
