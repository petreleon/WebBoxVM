import { errorMessage } from "./vm-worker/errors.js";
import { handleMessage } from "./vm-worker/messages.js";

self.onmessage = (event) => {
  handleMessage(event.data).catch((error) => {
    postMessage({ error: errorMessage(error), event: "error" });
  });
};
