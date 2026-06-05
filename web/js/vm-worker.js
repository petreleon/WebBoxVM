import { errorMessage } from "./vm-worker/errors.js";
import { handleMessage } from "./vm-worker/messages.js?v=20260606-jitprobe";

self.onmessage = (event) => {
  handleMessage(event.data).catch((error) => {
    postMessage({ error: errorMessage(error), event: "error" });
  });
};
