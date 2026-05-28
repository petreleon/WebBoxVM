import { Emulator } from './pkg/emulator.js';
import { readFileSync } from 'fs';

const cores = 4;
const efiSteps = 100_000;
const kernelSteps = 20_000_000;

console.log(`Loading 35.9MB Linux kernel...`);
const kernelImage = readFileSync('/Users/petreleon/code/WebBoxVM/Image.gz');

const emu = new Emulator(cores);
const result = emu.boot_kernel(new Uint8Array(kernelImage), cores);
console.log(`Setup: ${result}`);

console.log(`EFI phase...`);
const efi = emu.run_efi(efiSteps);
console.log(`  ${efi}`);

for (let i = 0; i < 4; i++) {
  const p = emu.run_kernel(kernelSteps / 4);
  const uart = emu.uart_output();
  if (uart.length > 0) {
    console.log(`  Round ${i+1}: UART (${uart.length}B): "${uart.slice(0,200)}"`);
  }
}

console.log(`\nTotal: ${emu.total_steps()} steps, PC=0x${emu.pc().toString(16)}`);
console.log(`UART: ${emu.uart_output().length} bytes`);
console.log(`Done.`);
