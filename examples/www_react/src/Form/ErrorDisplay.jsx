import React from "react";

function ErrorDisplay({ errors }) {
  if (!errors) return null;

  const { start, end } = errors;

  return (
    <span className="error-message">
      Error between {start} and {end}.
    </span>
  );
}

export default ErrorDisplay;
