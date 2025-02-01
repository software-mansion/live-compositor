import init, * as wasm from './generated/compositor_web';

/**
 * Loads and initializes wasm module required for the smelter to work.
 * @param wasmModuleUrl {string} - An URL for `smelter.wasm` file. The file is located in `dist` folder.
 */
export async function loadWasmModule(wasmModuleUrl: string) {
  await init({ module_or_path: wasmModuleUrl });
}

export { wasm };
