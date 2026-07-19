export function installUartProbe() {
  const probe = document.createElement("pre");
  const text = document.createTextNode("");
  probe.append(text);
  probe.dataset.testid = "webboxvm-uart-tail";
  probe.style.cssText = [
    "position:fixed",
    "left:-10000px",
    "top:0",
    "width:1px",
    "height:1px",
    "overflow:hidden",
  ].join(";");
  document.body.append(probe);
  return text;
}
