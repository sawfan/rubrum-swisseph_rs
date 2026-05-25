function setStatus(text) {
  document.getElementById("status").textContent = text;
}

function appendOutput(text) {
  const out = document.getElementById("output");
  out.textContent += text;
  out.scrollTop = out.scrollHeight;
}

function clearOutput() {
  document.getElementById("output").textContent = "";
}

async function fetchBytes(url, { optional = false } = {}) {
  const res = await fetch(url);
  if (!res.ok) {
    if (optional && res.status === 404) {
      return null;
    }
    throw new Error(`Failed to fetch ${url}: ${res.status} ${res.statusText}`);
  }
  return new Uint8Array(await res.arrayBuffer());
}

async function loadAssetManifest() {
  // Copied to dist via <link data-trunk rel="copy-dir" href="./assets" />
  // in web/index.html.
  const url = new URL("./assets/manifest.json", import.meta.url);
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(
      `Failed to fetch assets/manifest.json: ${res.status} ${res.statusText}`,
    );
  }
  return await res.json();
}

async function loadEpheAssetsFromManifest() {
  const manifest = await loadAssetManifest();

  const mount = typeof manifest.mount === "string" ? manifest.mount : "ephe";
  const files = Array.isArray(manifest.files) ? manifest.files : [];

  const loadedFiles = [];

  for (const f of files) {
    if (!f || typeof f.path !== "string") continue;
    const optional = !!f.optional;

    const fileUrl = new URL(`./assets/${f.path}`, import.meta.url);
    const bytes = await fetchBytes(fileUrl, { optional });
    if (bytes === null) {
      console.warn(`[assets] Missing optional asset: ${f.path}`);
      continue;
    }

    loadedFiles.push({ name: f.path, bytes });
  }

  return { mount, loadedFiles };
}

function setDateDefaults() {
  const dateEl = document.getElementById("date");
  if (!dateEl.value) {
    const now = new Date();
    const yyyy = now.getUTCFullYear();
    const mm = String(now.getUTCMonth() + 1).padStart(2, "0");
    const dd = String(now.getUTCDate()).padStart(2, "0");
    dateEl.value = `${yyyy}-${mm}-${dd}`;

    document.getElementById("hour").value = String(now.getUTCHours());
    document.getElementById("minute").value = String(now.getUTCMinutes());
    document.getElementById("second").value = String(now.getUTCSeconds());
  }
}

function parseUtcFromInputs() {
  const dateStr = document.getElementById("date").value;
  if (!dateStr) {
    throw new Error("Date is required");
  }
  const [y, m, d] = dateStr.split("-").map((n) => Number(n));
  if (!Number.isFinite(y) || !Number.isFinite(m) || !Number.isFinite(d)) {
    throw new Error("Invalid date");
  }

  const hour = Number(document.getElementById("hour").value || 0);
  const minute = Number(document.getElementById("minute").value || 0);
  const second = Number(document.getElementById("second").value || 0);

  const hourUt = hour + minute / 60 + second / 3600;
  return { year: y, month: m, day: d, hourUt };
}

function parseGeoFromInputs() {
  const lat = Number(document.getElementById("lat").value);
  const lon = Number(document.getElementById("lon").value);
  if (!Number.isFinite(lat) || !Number.isFinite(lon)) {
    throw new Error("Invalid lat/lon");
  }
  return { lat, lon };
}

function readLastError(instance) {
  const e = instance.exports;
  if (typeof e.swisseph_last_error_ptr !== "function") {
    return "";
  }

  const ptr = e.swisseph_last_error_ptr();
  if (!ptr) {
    return "";
  }

  const mem = new Uint8Array(e.memory.buffer);
  let end = ptr;
  while (end < mem.length && mem[end] !== 0) end++;
  return new TextDecoder().decode(mem.slice(ptr, end));
}

function writeUtf8ToWasm(instance, str) {
  const e = instance.exports;
  const enc = new TextEncoder();
  const bytes = enc.encode(str);
  const ptr = e.swisseph_alloc(bytes.length);
  const mem = new Uint8Array(e.memory.buffer);
  mem.set(bytes, ptr);
  return { ptr, len: bytes.length };
}

function freeUtf8FromWasm(instance, { ptr, len }) {
  instance.exports.swisseph_dealloc(ptr, len);
}

function writeF64ToWasm(instance, values) {
  const e = instance.exports;
  const len = values.length;
  const ptr = e.swisseph_alloc_f64(len);
  const arr = new Float64Array(e.memory.buffer, ptr, len);
  arr.set(values);
  return { ptr, len };
}

function readF64FromWasm(instance, ptr, len) {
  const e = instance.exports;
  const arr = new Float64Array(e.memory.buffer, ptr, len);
  return Array.from(arr);
}

function freeF64(instance, { ptr, len }) {
  instance.exports.swisseph_dealloc_f64(ptr, len);
}

function allocBytes(instance, len) {
  const ptr = instance.exports.swisseph_alloc(len);
  return { ptr, len };
}

function freeBytes(instance, { ptr, len }) {
  instance.exports.swisseph_dealloc(ptr, len);
}

async function loadUnknownInstance() {
  clearOutput();
  setStatus("Loading wasm...");

  const wasmUrl = new URL("./rubrum_swisseph_unknown.wasm", import.meta.url);
  const wasmResponse = await fetch(wasmUrl);
  if (!wasmResponse.ok) {
    throw new Error(
      `Failed to fetch rubrum_swisseph_unknown.wasm: ${wasmResponse.status} ${wasmResponse.statusText}`,
    );
  }

  let module;
  try {
    module = await WebAssembly.compileStreaming(wasmResponse);
  } catch {
    module = await WebAssembly.compile(await wasmResponse.arrayBuffer());
  }

  setStatus("Instantiating...");

  // The wasm module expects a minimal set of libc-like imports.
  // Currently Swiss Ephemeris pulls in `toupper`.
  const imports = {
    env: {
      toupper: (c) => {
        // C signature: int toupper(int c). The input is either EOF (-1) or an unsigned char.
        if (c === -1) return -1;
        const cc = c & 0xff;
        return cc >= 97 && cc <= 122 ? cc - 32 : cc;
      },
    },
  };

  const instance = await WebAssembly.instantiate(module, imports);

  const e = instance.exports;
  if (!e.memory) {
    throw new Error("Missing export: memory");
  }

  const required = [
    "swisseph_alloc",
    "swisseph_dealloc",
    "swisseph_alloc_f64",
    "swisseph_dealloc_f64",
    "swisseph_vfs_init",
    "swisseph_add_ephe_file",
    "swisseph_set_ephe_path_utf8",
    "rubrum_render_chart_svg_utf8",
  ];

  for (const name of required) {
    if (typeof e[name] !== "function") {
      throw new Error(`Missing export: ${name}`);
    }
  }

  const initRc = e.swisseph_vfs_init();
  if (initRc !== 0) {
    throw new Error(`swisseph_vfs_init failed: ${readLastError(instance)}`);
  }
  appendOutput("[host] VFS initialized\n");

  const { mount, loadedFiles } = await loadEpheAssetsFromManifest();
  appendOutput(`[host] Loaded ${loadedFiles.length} ephemeris assets\n`);

  for (const f of loadedFiles) {
    const nameBuf = writeUtf8ToWasm(instance, f.name);
    const dataBuf = allocBytes(instance, f.bytes.length);
    try {
      const mem = new Uint8Array(e.memory.buffer);
      mem.set(f.bytes, dataBuf.ptr);

      const addRc = e.swisseph_add_ephe_file(
        nameBuf.ptr,
        nameBuf.len,
        dataBuf.ptr,
        dataBuf.len,
      );
      if (addRc !== 0) {
        throw new Error(
          `swisseph_add_ephe_file(${f.name}) failed: ${readLastError(instance)}`,
        );
      }
    } finally {
      freeUtf8FromWasm(instance, nameBuf);
      freeBytes(instance, dataBuf);
    }
  }

  const mountBuf = writeUtf8ToWasm(instance, mount);
  try {
    const rc = e.swisseph_set_ephe_path_utf8(mountBuf.ptr, mountBuf.len);
    if (rc !== 0) {
      throw new Error(`swe_set_ephe_path failed: ${readLastError(instance)}`);
    }
    appendOutput(`[host] swe_set_ephe_path("${mount}") ok\n`);
  } finally {
    freeUtf8FromWasm(instance, mountBuf);
  }

  setStatus("Ready");
  return { instance, mount };
}

let wasmStatePromise = null;

async function getWasmState() {
  if (!wasmStatePromise) {
    wasmStatePromise = loadUnknownInstance();
  }
  return await wasmStatePromise;
}

function disableCalc(disabled) {
  const btn = document.getElementById("calc");
  btn.disabled = disabled;
}

function setChartSvg(svgText) {
  const el = document.getElementById("chart");
  el.innerHTML = svgText;
}

async function renderChart() {
  const { instance } = await getWasmState();
  const e = instance.exports;

  const { year, month, day, hourUt } = parseUtcFromInputs();
  const { lat, lon } = parseGeoFromInputs();

  // house system: 'P' (Placidus)
  const hsysCharCode = "P".charCodeAt(0);

  const svgBuf = writeF64ToWasm(instance, [0.0]); // dummy to force memory init
  freeF64(instance, svgBuf);

  const svgPtr = e.rubrum_render_chart_svg_utf8(
    year,
    month,
    day,
    hourUt,
    lat,
    lon,
    hsysCharCode,
  );

  if (!svgPtr) {
    const msg = readLastError(instance) || "unknown error";
    throw new Error(`render failed: ${msg}`);
  }

  // Read NUL-terminated UTF-8 from wasm memory.
  const mem = new Uint8Array(e.memory.buffer);
  let end = svgPtr;
  while (end < mem.length && mem[end] !== 0) end++;
  const svgText = new TextDecoder().decode(mem.slice(svgPtr, end));

  // Free the string via the wasm allocator.
  e.swisseph_dealloc(svgPtr, end - svgPtr + 1);

  setChartSvg(svgText);
  appendOutput("[host] Rendered SVG chart\n");
}

document.addEventListener("DOMContentLoaded", () => {
  setDateDefaults();
  setStatus("Initializing...");

  getWasmState().catch((err) => {
    appendOutput(`[host] Error during init: ${err?.stack || err}\n`);
    setStatus("Error");
  });

  document.getElementById("calc").addEventListener("click", () => {
    disableCalc(true);
    setStatus("Rendering...");

    renderChart()
      .then(() => {
        setStatus("Ready");
      })
      .catch((err) => {
        appendOutput(`[host] Error: ${err?.stack || err}\n`);
        setStatus("Error");
      })
      .finally(() => {
        disableCalc(false);
      });
  });
});

