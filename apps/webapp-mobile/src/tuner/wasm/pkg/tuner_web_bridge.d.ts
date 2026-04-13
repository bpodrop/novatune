/* tslint:disable */
/* eslint-disable */

export function new_detector(sample_rate: number, frame_size: number, hop_size: number): number;

export function next_output(detector_id: number): any | undefined;

export function push_samples(detector_id: number, samples: Float32Array): number;

export function reset(detector_id: number): boolean;

export function set_calibration_hz(detector_id: number, calibration_hz: number): boolean;

export function set_min_clarity(detector_id: number, min_clarity: number): boolean;

export function set_min_rms(detector_id: number, min_rms: number): boolean;

export function set_mode(detector_id: number, mode: string): boolean;

export function set_preset(detector_id: number, preset_id: string): boolean;

export function shutdown(detector_id: number): boolean;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly new_detector: (a: number, b: number, c: number) => number;
    readonly next_output: (a: number) => any;
    readonly push_samples: (a: number, b: number, c: number) => number;
    readonly reset: (a: number) => number;
    readonly set_calibration_hz: (a: number, b: number) => number;
    readonly set_min_clarity: (a: number, b: number) => number;
    readonly set_min_rms: (a: number, b: number) => number;
    readonly set_mode: (a: number, b: number, c: number) => number;
    readonly set_preset: (a: number, b: number, c: number) => number;
    readonly shutdown: (a: number) => number;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
