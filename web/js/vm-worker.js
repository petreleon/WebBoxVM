import { errorMessage } from "./vm-worker/errors.js?v=20260903-virgl-blend-r1";
import { handleMessage } from "./vm-worker/messages.js?v=20260903-virgl-blend-r1";

self.onmessage = (event) => {
  handleMessage(event.data).catch((error) => {
    postMessage({ error: errorMessage(error), event: "error" });
  });
};
