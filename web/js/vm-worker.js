import { errorMessage } from "./vm-worker/errors.js?v=20260720-input-latency-r4";
import { handleMessage } from "./vm-worker/messages.js?v=20260720-input-latency-r4";

self.onmessage = (event) => {
  handleMessage(event.data).catch((error) => {
    postMessage({ error: errorMessage(error), event: "error" });
  });
};
