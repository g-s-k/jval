
let wasm;

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new TextEncoder('utf-8');

let cachegetUint8Memory = null;
function getUint8Memory() {
    if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory;
}

let passStringToWasm;
if (typeof cachedTextEncoder.encodeInto === 'function') {
    passStringToWasm = function(arg) {


        let size = arg.length;
        let ptr = wasm.__wbindgen_malloc(size);
        let offset = 0;
        {
            const mem = getUint8Memory();
            for (; offset < arg.length; offset++) {
                const code = arg.charCodeAt(offset);
                if (code > 0x7F) break;
                mem[ptr + offset] = code;
            }
        }

        if (offset !== arg.length) {
            arg = arg.slice(offset);
            ptr = wasm.__wbindgen_realloc(ptr, size, size = offset + arg.length * 3);
            const view = getUint8Memory().subarray(ptr + offset, ptr + size);
            const ret = cachedTextEncoder.encodeInto(arg, view);

            offset += ret.written;
        }
        WASM_VECTOR_LEN = offset;
        return ptr;
    };
} else {
    passStringToWasm = function(arg) {


        let size = arg.length;
        let ptr = wasm.__wbindgen_malloc(size);
        let offset = 0;
        {
            const mem = getUint8Memory();
            for (; offset < arg.length; offset++) {
                const code = arg.charCodeAt(offset);
                if (code > 0x7F) break;
                mem[ptr + offset] = code;
            }
        }

        if (offset !== arg.length) {
            const buf = cachedTextEncoder.encode(arg.slice(offset));
            ptr = wasm.__wbindgen_realloc(ptr, size, size = offset + buf.length);
            getUint8Memory().set(buf, ptr + offset);
            offset += buf.length;
        }
        WASM_VECTOR_LEN = offset;
        return ptr;
    };
}
/**
* @param {string} json
* @returns {Error}
*/
export function validate(json) {
    const ret = wasm.validate(passStringToWasm(json), WASM_VECTOR_LEN);
    return ret === 0 ? undefined : Error.__wrap(ret);
}

let cachegetInt32Memory = null;
function getInt32Memory() {
    if (cachegetInt32Memory === null || cachegetInt32Memory.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory;
}

let cachedTextDecoder = new TextDecoder('utf-8');

function getStringFromWasm(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}
/**
* @param {string} json
* @returns {string}
*/
export function format_packed(json) {
    const retptr = 8;
    const ret = wasm.format_packed(retptr, passStringToWasm(json), WASM_VECTOR_LEN);
    const memi32 = getInt32Memory();
    let v0;
    if (memi32[retptr / 4 + 0] !== 0) {
        v0 = getStringFromWasm(memi32[retptr / 4 + 0], memi32[retptr / 4 + 1]).slice();
        wasm.__wbindgen_free(memi32[retptr / 4 + 0], memi32[retptr / 4 + 1] * 1);
    }
    return v0;
}

/**
* @param {string} json
* @returns {string}
*/
export function format_tabs(json) {
    const retptr = 8;
    const ret = wasm.format_tabs(retptr, passStringToWasm(json), WASM_VECTOR_LEN);
    const memi32 = getInt32Memory();
    let v0;
    if (memi32[retptr / 4 + 0] !== 0) {
        v0 = getStringFromWasm(memi32[retptr / 4 + 0], memi32[retptr / 4 + 1]).slice();
        wasm.__wbindgen_free(memi32[retptr / 4 + 0], memi32[retptr / 4 + 1] * 1);
    }
    return v0;
}

/**
* @param {string} json
* @param {number} spacing
* @returns {string}
*/
export function format_spaces(json, spacing) {
    const retptr = 8;
    const ret = wasm.format_spaces(retptr, passStringToWasm(json), WASM_VECTOR_LEN, spacing);
    const memi32 = getInt32Memory();
    let v0;
    if (memi32[retptr / 4 + 0] !== 0) {
        v0 = getStringFromWasm(memi32[retptr / 4 + 0], memi32[retptr / 4 + 1]).slice();
        wasm.__wbindgen_free(memi32[retptr / 4 + 0], memi32[retptr / 4 + 1] * 1);
    }
    return v0;
}

/**
*/
export class Error {

    static __wrap(ptr) {
        const obj = Object.create(Error.prototype);
        obj.ptr = ptr;

        return obj;
    }

    free() {
        const ptr = this.ptr;
        this.ptr = 0;

        wasm.__wbg_error_free(ptr);
    }
    /**
    * @returns {number}
    */
    get start() {
        const ret = wasm.__wbg_get_error_start(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} arg0
    */
    set start(arg0) {
        wasm.__wbg_set_error_start(this.ptr, arg0);
    }
    /**
    * @returns {number}
    */
    get end() {
        const ret = wasm.__wbg_get_error_end(this.ptr);
        return ret >>> 0;
    }
    /**
    * @param {number} arg0
    */
    set end(arg0) {
        wasm.__wbg_set_error_end(this.ptr, arg0);
    }
}

function init(module) {
    if (typeof module === 'undefined') {
        module = import.meta.url.replace(/\.js$/, '_bg.wasm');
    }
    let result;
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm(arg0, arg1));
    };

    if ((typeof URL === 'function' && module instanceof URL) || typeof module === 'string' || (typeof Request === 'function' && module instanceof Request)) {

        const response = fetch(module);
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            result = WebAssembly.instantiateStreaming(response, imports)
            .catch(e => {
                return response
                .then(r => {
                    if (r.headers.get('Content-Type') != 'application/wasm') {
                        console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);
                        return r.arrayBuffer();
                    } else {
                        throw e;
                    }
                })
                .then(bytes => WebAssembly.instantiate(bytes, imports));
            });
        } else {
            result = response
            .then(r => r.arrayBuffer())
            .then(bytes => WebAssembly.instantiate(bytes, imports));
        }
    } else {

        result = WebAssembly.instantiate(module, imports)
        .then(result => {
            if (result instanceof WebAssembly.Instance) {
                return { instance: result, module };
            } else {
                return result;
            }
        });
    }
    return result.then(({instance, module}) => {
        wasm = instance.exports;
        init.__wbindgen_wasm_module = module;

        return wasm;
    });
}

export default init;

