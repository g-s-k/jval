import React, { useCallback, useContext, useReducer, useRef } from "react";

import WasmContext from "../WasmContext";
import ErrorDisplay from "./ErrorDisplay";
import Format from "./Format";

const PLACEHOLDER = "{\n  ...\n}";

const DEFAULT_FORM_STATE = {
  errors: "",
  text: ""
};

const ACTIONS = Object.freeze({
  FORMAT: "format",
  SET_TEXT: "set--text"
});

function formReducer(state, { type, value, errors, func }) {
  switch (type) {
    case ACTIONS.FORMAT:
      return { ...state, errors: "", text: func(state) };
    case ACTIONS.SET_TEXT:
      return { ...state, errors, text: value };
    default:
      return state;
  }
}

function Form() {
  const [{ errors, text }, dispatch] = useReducer(
    formReducer,
    DEFAULT_FORM_STATE
  );
  const { loading, validate = () => {} } = useContext(WasmContext);
  const textAreaRef = useRef();

  const onTextChange = useCallback(
    ({ target: { value = "" } }) => {
      const errors = validate(value);

      dispatch({ type: ACTIONS.SET_TEXT, value, errors });

      if (textAreaRef.current) {
        textAreaRef.current.setCustomValidity(errors ? "error in json" : "");
      }
    },
    [validate]
  );

  const onGotoError = useCallback(() => {
    if (textAreaRef.current && errors) {
      textAreaRef.current.setSelectionRange(errors.start, errors.end);
      textAreaRef.current.focus();
    }
  }, [errors]);

  const onFormat = useCallback(
    func => dispatch({ type: ACTIONS.FORMAT, func }),
    []
  );

  return (
    <section>
      <label>
        Enter JSON to validate:
        <textarea
          autoFocus
          disabled={loading}
          onChange={onTextChange}
          placeholder={PLACEHOLDER}
          ref={textAreaRef}
          rows={10}
          value={text}
        />
      </label>
      <div className="controls">
        <button onClick={onGotoError}>Go to next error</button>
        <ErrorDisplay errors={errors} />
        <Format onFormat={onFormat} />
      </div>
    </section>
  );
}

export default Form;
