// 4-core Linux kernel boot test — wasm64 + Node.js
import { Emulator } from './pkg/emulator.js';
import { readFileSync } from 'fs';

const cores = 4;
const maxSteps = 2_000_000; // 2M steps (EFI + kernel)

console.log(`Loading kernel image...`);
const kernelImage = readFileSync('/Users/petreleon/code/WebBoxVM/Image.gz');
console.log(`  ${(kernelImage.length / 1024 / 1024).toFixed(1)} MB loaded`);

console.log(`Setting up ${cores}-core boot...`);
const emu = new Emulator(cores);
const result = emu.boot_kernel(new Uint8Array(kernelImage), cores);
console.log(`  ${result}`);

console.log(`Running EFI stub (${maxSteps} steps)...`);
const efiSteps = emu.run_efi(maxSteps);
console.log(`  ${efiSteps}`);

console.log(`Running ${cores}-core kernel phase...`);
const kernelSteps = emu.run_kernel(maxSteps);
console.log(`  ${kernelSteps}`);

console.log(`\nTotal steps: ${emu.total_steps()}`);
console.log(`PC (core 0): 0x${emu.pc().toString(16)}`);
const uartOut = emu.uart_output();
console.log(`UART output (${uartOut.length} bytes): "${uartOut.slice(0, 200)}"`);
console.log(`\nDone.`);
