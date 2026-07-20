export function installUartProbe() {
  return installTextProbe("webboxvm-uart-tail").text;
}

export function installTextProbe(testId) {
  const probe = document.createElement("pre");
  const text = document.createTextNode("");
  probe.append(text);
  probe.dataset.testid = testId;
  probe.setAttribute?.("aria-hidden", "true");
  probe.style.cssText = [
    "position:fixed",
    "left:-10000px",
    "top:0",
    "width:1px",
    "height:1px",
    "overflow:hidden",
  ].join(";");
  document.body.append(probe);
  return { probe, text };
}
