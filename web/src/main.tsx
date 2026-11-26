import { StrictMode } from "react";
import { flushSync } from "react-dom";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import "./styles.css";
import { applyStoredTheme } from "./Theme";
import type { DocsData } from "./types";

const payload = document.getElementById("docs-data")?.textContent;
const container = document.getElementById("root");

applyStoredTheme();

if (payload && container) {
  const data = JSON.parse(payload) as DocsData;
  // Rendering before the document finishes loading lets the browser resolve a URL fragment itself.
  flushSync(() => {
    createRoot(container).render(
      <StrictMode>
        <App data={data} />
      </StrictMode>,
    );
  });
}
