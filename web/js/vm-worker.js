import { errorMessage } from "./vm-worker/errors.js?v=20260718-staged-fast-boot";
import { handleMessage } from "./vm-worker/messages.js?v=20260718-staged-fast-boot";

self.onmessage = (event) => {
  handleMessage(event.data).catch((error) => {
    postMessage({ error: errorMessage(error), event: "error" });
  });
};
