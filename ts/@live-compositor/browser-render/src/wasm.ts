import init, * as wasm from './generated/compositor_web';

/**
 * Loads and initializes wasm module required for the live compositor to work.
 * @param wasmModuleUrl {string} - An URL for `live-compositor.wasm` file. The file is located in `dist` folder.
 */
export async function loadWasmModule(wasmModuleUrl: string) {
  await init({ module_or_path: wasmModuleUrl });
}

export { wasm };
