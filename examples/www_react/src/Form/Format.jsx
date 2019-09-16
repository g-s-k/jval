import React, { useCallback, useContext, useMemo, useReducer } from "react";

import WasmContext from "../WasmContext";

const SPACE = "space",
  TAB = "tab",
  NONE = "none";

const DEFAULT_FORMAT_STATE = {
  number: 2,
  style: SPACE
};

const ACTIONS = Object.freeze({
  SET_NUMBER: "set--number",
  SET_STYLE: "set--style"
});

function formatReducer(state, { number, style, type }) {
  switch (type) {
    case ACTIONS.SET_NUMBER:
      return { ...state, number };
    case ACTIONS.SET_STYLE:
      return { number: style === SPACE ? 2 : 0, style };
    default:
      return state;
  }
}

function Format({ onFormat }) {
  const [{ number, style }, dispatch] = useReducer(
    formatReducer,
    DEFAULT_FORMAT_STATE
  );

  const setNumber = useCallback(({ target: { value = "" } }) => {
    const number = parseInt(value);

    if (!isNaN(number)) {
      dispatch({ type: ACTIONS.SET_NUMBER, number });
    }
  }, []);

  const setStyle = useCallback(({ target: { value = "" } }) => {
    dispatch({ type: ACTIONS.SET_STYLE, style: value });
  }, []);

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

  const onClick = useCallback(() => {
    onFormat(formatText);
  }, [formatText, onFormat]);

  return (
    <span>
      <input
        disabled={number < 2}
        min={2}
        onChange={setNumber}
        step={2}
        type="number"
        value={number}
      />
      <select onChange={setStyle} value={style}>
        <option value={SPACE}>spaces</option>
        <option value={TAB}>tabs</option>
        <option value={NONE}>none</option>
      </select>
      <button onClick={onClick}>Format</button>
    </span>
  );
}

export default Format;
