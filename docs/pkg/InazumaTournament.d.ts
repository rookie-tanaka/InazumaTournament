/* tslint:disable */
/* eslint-disable */
export function get_playable_opponents_info(settings_val: any): Promise<any>;
export function generate_tournament(settings_val: any): Promise<any>;
export function get_all_opponents(): Promise<any>;
export function update_match_result(tournament_val: any, round_index: number, match_index: number, winner_name: string): Promise<any>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly generate_tournament: (a: any) => any;
  readonly get_all_opponents: () => any;
  readonly get_playable_opponents_info: (a: any) => any;
  readonly update_match_result: (a: any, b: number, c: number, d: number, e: number) => any;
  readonly wasm_bindgen__convert__closures_____invoke__h845ebb1740122ec3: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__ha4f38df198ace1a1: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h613af60e4c1e1569: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
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
