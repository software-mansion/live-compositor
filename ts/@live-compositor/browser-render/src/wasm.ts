import init, * as wasm from './generated/compositor_web';

/**
 * Loads and initializes all resources required for the live compositor to work.
 * @param wasmFileUrl {string} - An URL for `live-compositor.wasm` file. The file is located in `dist` folder.
 */
export async function initCompositor(wasmFileUrl: string) {
  await init(wasmFileUrl);
}

export { wasm };
