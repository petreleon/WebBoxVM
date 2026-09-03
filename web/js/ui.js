import { formatBytes } from "./utils.js?v=20260903-virgl-blend-r1";

export class UiController {
  constructor(els) {
    this.els = els;
    this.controlState = "idle";
  }

  setControls(state, disk, emulator) {
    this.controlState = state;
    const busy = state === "booting";
    const active = state === "running";
    const paused = state === "paused";
    const hasVm = Boolean(emulator);

    this.els.bootIso.disabled = busy || active;
    this.els.bootDebian.disabled = busy || active;
    this.els.bootDisk.disabled = busy || active || !disk.available || disk.persistedBytes === 0;
    this.els.diskSize.disabled = busy || active;
    this.els.isoFile.disabled = busy || active;
    this.els.pauseVm.disabled = !active;
    this.els.resumeVm.disabled = !paused;
    this.els.resetVm.disabled = !(active || paused || busy);
    this.els.saveDisk.disabled = busy || !disk.available || !hasVm || disk.saving;
    this.els.clearDisk.disabled = busy || active || !disk.available || disk.persistedBytes === 0;
  }

  updateMetrics(emulator, disk) {
    if (!emulator) {
      this.setText(this.els.stepsValue, "0");
      this.setText(this.els.pcValue, "0x0");
      this.setText(this.els.uartValue, "0 B");
      this.setText(this.els.pagesValue, "0");
      this.setText(this.els.netValue, "Off");
      this.setText(this.els.diskValue, "0 B");
      this.updateJitStats(undefined);
      this.updateStorageMetric(disk);
      return;
    }

    this.setText(this.els.stepsValue, emulator.total_steps().toString());
    this.setText(this.els.pcValue, `0x${emulator.pc().toString(16)}`);
    this.setText(this.els.uartValue, formatBytes(emulator.uart_output_len()));
    this.setText(this.els.pagesValue, emulator.allocated_pages().toString());
    this.updateNetworkMetric(emulator);
    this.setText(this.els.diskValue, formatBytes(Number(emulator.install_disk_allocated_bytes())));
    this.updateStorageMetric(disk);
  }

  updateJitStats(emulator) {
    this.setText(this.els.jitStatsValue, JSON.stringify(emulator?.jit_stats?.() ?? null));
  }

  updateNetworkMetric(emulator) {
    const net = emulator.network_stats();
    const rx = Number(net.rxPackets);
    const tx = Number(net.txPackets);
    this.setText(this.els.netValue, `${net.status} ${rx}/${tx}`);
  }

  updateStorageMetric(disk) {
    if (!disk.available) {
      this.setText(this.els.savedValue, "Off");
    } else if (disk.saving) {
      this.setText(this.els.savedValue, "Saving");
    } else if (disk.persistedBytes > 0) {
      this.setText(this.els.savedValue, formatBytes(disk.persistedBytes));
    } else {
      this.setText(this.els.savedValue, "Ready");
    }
  }

  setStatus(message, tone = "normal") {
    this.setText(this.els.statusLine, message);
    this.els.statusLine.dataset.tone = tone;
  }

  setText(element, text) {
    if (element.textContent !== text) {
      element.textContent = text;
    }
  }

  log(message) {
    const timestamp = new Date().toLocaleTimeString();
    this.els.eventLog.textContent += `[${timestamp}] ${message}\n`;
    const lines = this.els.eventLog.textContent.split("\n");
    if (lines.length > 200) {
      this.els.eventLog.textContent = `${lines.slice(-200).join("\n")}\n`;
    }
    this.els.eventLog.scrollTop = this.els.eventLog.scrollHeight;
  }
}
