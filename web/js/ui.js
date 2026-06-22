import { formatBytes } from "./utils.js";

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
      this.els.stepsValue.textContent = "0";
      this.els.pcValue.textContent = "0x0";
      this.els.uartValue.textContent = "0 B";
      this.els.pagesValue.textContent = "0";
      this.els.netValue.textContent = "Off";
      this.els.diskValue.textContent = "0 B";
      this.updateJitStats(undefined);
      this.updateStorageMetric(disk);
      return;
    }

    this.els.stepsValue.textContent = emulator.total_steps().toString();
    this.els.pcValue.textContent = `0x${emulator.pc().toString(16)}`;
    this.els.uartValue.textContent = formatBytes(emulator.uart_output_len());
    this.els.pagesValue.textContent = emulator.allocated_pages().toString();
    this.updateNetworkMetric(emulator);
    this.els.diskValue.textContent = formatBytes(Number(emulator.install_disk_allocated_bytes()));
    this.updateStorageMetric(disk);
  }

  updateJitStats(emulator) {
    this.els.jitStatsValue.textContent = JSON.stringify(emulator?.jit_stats?.() ?? null);
  }

  updateNetworkMetric(emulator) {
    const net = emulator.network_stats();
    const rx = Number(net.rxPackets);
    const tx = Number(net.txPackets);
    this.els.netValue.textContent = `${net.status} ${rx}/${tx}`;
  }

  updateStorageMetric(disk) {
    if (!disk.available) {
      this.els.savedValue.textContent = "Off";
    } else if (disk.saving) {
      this.els.savedValue.textContent = "Saving";
    } else if (disk.persistedBytes > 0) {
      this.els.savedValue.textContent = formatBytes(disk.persistedBytes);
    } else {
      this.els.savedValue.textContent = "Ready";
    }
  }

  setStatus(message, tone = "normal") {
    this.els.statusLine.textContent = message;
    this.els.statusLine.dataset.tone = tone;
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
