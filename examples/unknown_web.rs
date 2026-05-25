//! wasm32-unknown-unknown demo that combines:
//! - Swiss Ephemeris (via `swisseph`)
//! - `rubrum_swisseph` conversions
//! - optional `rubrum_svg` SVG rendering
//!
//! The JS host must:
//! - call `swisseph_add_ephe_file()` for each `.se1` file
//! - call `swisseph_set_ephe_path_utf8("ephe")`
//! - then call `rubrum_render_chart_svg_utf8()` to get an SVG string.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_int, c_void};

use swisseph::{Body, CalandarKind, HouseSystemKind, Seflg, swe, swe2};

thread_local! {
    static LAST_ERROR: RefCell<[u8; 512]> = const { RefCell::new([0; 512]) };
    static EPHE_FILES: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(HashMap::new());
}

fn set_last_error(msg: &str) {
    LAST_ERROR.with(|buf| {
        let mut buf = buf.borrow_mut();
        buf.fill(0);

        let bytes = msg.as_bytes();
        let n = bytes.len().min(buf.len().saturating_sub(1));
        buf[..n].copy_from_slice(&bytes[..n]);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn swisseph_last_error_ptr() -> *const u8 {
    LAST_ERROR.with(|buf| buf.borrow().as_ptr())
}

/// Entry point (not used in this embedding).
fn main() {}

#[unsafe(no_mangle)]
pub extern "C" fn swisseph_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// # Safety
///
/// - `ptr` must have been allocated by `swisseph_alloc`.
/// - `len` must match the original allocation capacity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn swisseph_dealloc(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }

    unsafe {
        drop(Vec::<u8>::from_raw_parts(ptr, 0, len));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn swisseph_alloc_f64(len: usize) -> *mut f64 {
    let mut buf = Vec::<f64>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// # Safety
///
/// - `ptr` must have been allocated by `swisseph_alloc_f64`.
/// - `len` must match the original allocation capacity.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn swisseph_dealloc_f64(ptr: *mut f64, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }

    unsafe {
        drop(Vec::<f64>::from_raw_parts(ptr, 0, len));
    }
}

/// Add (or replace) an ephemeris file in the in-wasm VFS.
///
/// Returns 0 on success, -1 on error.
///
/// # Safety
///
/// - `name_ptr` must point to a readable UTF-8 buffer of length `name_len`.
/// - `data_ptr` must point to a readable buffer of length `data_len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn swisseph_add_ephe_file(
    name_ptr: *const u8,
    name_len: usize,
    data_ptr: *const u8,
    data_len: usize,
) -> i32 {
    if name_ptr.is_null() || data_ptr.is_null() {
        set_last_error("null pointer");
        return -1;
    }

    let name_bytes = unsafe { std::slice::from_raw_parts(name_ptr, name_len) };
    let name = match std::str::from_utf8(name_bytes) {
        Ok(s) => s.to_string(),
        Err(e) => {
            set_last_error(&format!("invalid utf-8 file name: {e}"));
            return -1;
        }
    };

    let data = unsafe { std::slice::from_raw_parts(data_ptr, data_len) }.to_vec();

    EPHE_FILES.with(|m| {
        m.borrow_mut().insert(name, data);
    });

    0
}

#[repr(C)]
struct VfsHandle {
    /// Stored as a heap-allocated C string (so we can carry it through `void*`).
    name: *mut c_char,
    /// Current read cursor for stdio-compat helpers (fread/fgets/fseek).
    pos: usize,
}

unsafe extern "C" fn vfs_open(
    _ifno: c_int,
    fname: *const c_char,
    _ephepath: *const c_char,
    _serr: *mut c_char,
) -> *mut c_void {
    if fname.is_null() {
        return std::ptr::null_mut();
    }

    let name = unsafe { CStr::from_ptr(fname) }
        .to_string_lossy()
        .to_string();

    let exists = EPHE_FILES.with(|m| m.borrow().contains_key(&name));
    if !exists {
        return std::ptr::null_mut();
    }

    let cname = match CString::new(name) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let h = Box::new(VfsHandle {
        name: cname.into_raw(),
        pos: 0,
    });

    Box::into_raw(h) as *mut c_void
}

unsafe extern "C" fn vfs_read_at(
    h: *mut c_void,
    dst: *mut c_void,
    size: usize,
    count: usize,
    offset: i32,
    _serr: *mut c_char,
) -> usize {
    if h.is_null() || dst.is_null() {
        return 0;
    }

    if size == 0 || count == 0 {
        return 0;
    }

    let h = unsafe { &*(h as *const VfsHandle) };
    if h.name.is_null() {
        return 0;
    }

    let name = unsafe { CStr::from_ptr(h.name) }
        .to_string_lossy()
        .to_string();
    let buf_opt = EPHE_FILES.with(|m| m.borrow().get(&name).cloned());
    let buf = match buf_opt {
        Some(b) => b,
        None => return 0,
    };

    if offset < 0 {
        return 0;
    }
    let offset = offset as usize;
    if offset > buf.len() {
        return 0;
    }

    let available = buf.len() - offset;
    let want_bytes = size.saturating_mul(count);
    let read_bytes = available.min(want_bytes);

    unsafe {
        std::ptr::copy_nonoverlapping(buf.as_ptr().add(offset), dst as *mut u8, read_bytes);
    }

    // fread-like semantics: return number of whole items read.
    read_bytes / size
}

unsafe extern "C" fn vfs_close(h: *mut c_void) {
    if h.is_null() {
        return;
    }

    let h = unsafe { Box::from_raw(h as *mut VfsHandle) };
    if !h.name.is_null() {
        unsafe {
            drop(CString::from_raw(h.name));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn swisseph_vfs_init() -> i32 {
    let api = swe::swe_vfs_api {
        open: Some(vfs_open),
        read_at: Some(vfs_read_at),
        close: Some(vfs_close),
    };

    // Leak the API struct so the pointer remains stable for the duration of the program.
    let api_ptr: *const swe::swe_vfs_api = Box::leak(Box::new(api));

    // Safety: `api_ptr` is valid and points to a stable (leaked) VFS API struct for the
    // remainder of the module's lifetime.
    unsafe {
        swe::set_vfs_api(api_ptr);
    }

    0
}

/// Set Swiss Ephemeris ephemeris path.
///
/// Returns 0 on success, -1 on error.
///
/// # Safety
///
/// - `ptr` must point to a valid readable buffer of length `len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn swisseph_set_ephe_path_utf8(ptr: *const u8, len: usize) -> i32 {
    if ptr.is_null() {
        set_last_error("null pointer");
        return -1;
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let path = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("invalid utf-8: {e}"));
            return -1;
        }
    };

    if path.contains('\0') {
        set_last_error("path contained NUL byte");
        return -1;
    }

    swe::set_ephe_path(path);
    0
}

fn house_system_from_char_code(code: i32) -> HouseSystemKind {
    // Keep it simple for the demo; default to Placidus.
    match char::from_u32(code as u32).unwrap_or('P') {
        'A' => HouseSystemKind::Equal,
        'W' => HouseSystemKind::WholeSign,
        'C' => HouseSystemKind::Campanus,
        'R' => HouseSystemKind::Regiomontanus,
        _ => HouseSystemKind::Placidus,
    }
}

fn compute_chart_data(
    jd_ut: f64,
    lat: f64,
    lon: f64,
    hsys: HouseSystemKind,
) -> Result<rubrum_render::ChartData, String> {
    // Compute bodies (dataset "natal").
    let bodies = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
        Body::Pluto,
    ];

    let flag = Seflg::SWIEPH | Seflg::SPEED;

    let mut placements = Vec::with_capacity(bodies.len());
    for body in bodies {
        let pos = swe2::calc_ut2_ecliptic(jd_ut, body.clone(), flag).map_err(|e| e.to_string())?;
        let pm =
            rubrum_swisseph::adapter::calc::placement_motion_from_ecliptic_position(body, &pos)
                .map_err(|e| e.to_string())?;
        placements.push(pm);
    }

    let natal_bodies = placements.clone();

    // Compute houses.
    // For now, we use `houses_ex2` with an iflag derived from Seflg bits.
    let (cusps, _ascmc) = swe2::houses_ex2(jd_ut, flag.bits() as i32, lat, lon, hsys);
    let house_cusps = rubrum_swisseph::adapter::houses::house_cusps_from_swisseph(&cusps)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|hsd| rubrum_render::chart_data::HouseCuspData {
            house: hsd.house,
            sign_degree: hsd.sign_degree,
        })
        .collect::<Vec<_>>();

    Ok(rubrum_render::ChartData {
        natal_bodies,
        datasets: vec![rubrum_render::DatasetData {
            id: "natal".to_string(),
            bodies: placements.clone(),
        }],
        house_sets: vec![rubrum_render::HouseSetData {
            id: "natal".to_string(),
            house_cusps: house_cusps.clone(),
        }],
        house_cusps,
    })
}

#[cfg(feature = "wasm_example_svg")]
fn render_svg(data: &rubrum_render::ChartData) -> Result<String, String> {
    use rubrum_render::embedded_configs::{
        CHART_SPEC_NATAL_ASPECTS_TOML, CHART_SPEC_NATAL_LAYOUT_ONLY_TOML, THEME_DARK_TOML,
    };

    #[derive(serde::Deserialize)]
    struct ThemeRoot {
        theme: rubrum_render::Theme,
    }

    #[derive(serde::Deserialize)]
    struct LayoutRoot {
        layout: rubrum_render::Layout,
    }

    #[derive(serde::Deserialize)]
    struct RulesRoot {
        rules: rubrum::AspectRules,
    }

    let mut theme: rubrum_render::Theme = toml::from_str::<ThemeRoot>(THEME_DARK_TOML)
        .map_err(|e| e.to_string())?
        .theme;

    // Point the theme's sprite URL at our staged asset.
    theme.svg.glyph_sprite_url = Some("./assets/glyphs_white.svg".to_string());

    let layout: rubrum_render::Layout =
        toml::from_str::<LayoutRoot>(CHART_SPEC_NATAL_LAYOUT_ONLY_TOML)
            .map_err(|e| e.to_string())?
            .layout;

    let rules: rubrum::AspectRules = toml::from_str::<RulesRoot>(CHART_SPEC_NATAL_ASPECTS_TOML)
        .map_err(|e| e.to_string())?
        .rules;

    rubrum_svg::chart_to_svg_string_spec(&theme, &layout, Some(&rules), data)
        .map_err(|e| e.to_string())
}

#[cfg(not(feature = "wasm_example_svg"))]
fn render_svg(_data: &rubrum_render::ChartData) -> Result<String, String> {
    Err("This example requires the `wasm_example_svg` feature".to_string())
}

/// Render a Rubrum chart to an SVG UTF-8 C string.
///
/// Returns a pointer to a NUL-terminated string allocated by `swisseph_alloc`.
/// The JS host must free it using `swisseph_dealloc(ptr, len)`.
///
/// # Safety
///
/// - The returned pointer must be freed by the caller.
#[unsafe(no_mangle)]
pub extern "C" fn rubrum_render_chart_svg_utf8(
    year: i32,
    month: i32,
    day: i32,
    hour_ut: f64,
    lat: f64,
    lon: f64,
    hsys_char_code: i32,
) -> *mut u8 {
    let jd_ut = swe::julday(year, month, day, hour_ut, CalandarKind::Gregorian as u32);
    let hsys = house_system_from_char_code(hsys_char_code);

    let data = match compute_chart_data(jd_ut, lat, lon, hsys) {
        Ok(d) => d,
        Err(e) => {
            set_last_error(&e);
            return std::ptr::null_mut();
        }
    };

    let svg = match render_svg(&data) {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&e);
            return std::ptr::null_mut();
        }
    };

    let mut bytes = svg.into_bytes();
    bytes.push(0); // NUL

    let ptr = swisseph_alloc(bytes.len());
    if ptr.is_null() {
        set_last_error("allocation failed");
        return std::ptr::null_mut();
    }

    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
    }

    ptr
}

// -----------------------------------------------------------------------------
// wasm32-unknown-unknown: minimal libc/stdio shims
// -----------------------------------------------------------------------------

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod wasm_shims {
    use super::{EPHE_FILES, VfsHandle, set_last_error};
    use core::ffi::{c_char, c_int, c_void};
    use core::ptr;
    use std::alloc::{Layout, alloc, dealloc, realloc as std_realloc};
    use std::ffi::CStr;

    const HEADER_SIZE: usize = core::mem::size_of::<usize>();
    const HEADER_ALIGN: usize = core::mem::align_of::<usize>();

    unsafe fn c_strlen(mut p: *const c_char) -> usize {
        if p.is_null() {
            return 0;
        }
        let mut n = 0usize;
        while unsafe { *p } != 0 {
            n += 1;
            p = unsafe { p.add(1) };
        }
        n
    }

    unsafe fn c_str_bytes(p: *const c_char) -> &'static [u8] {
        if p.is_null() {
            return &[];
        }
        let len = unsafe { c_strlen(p) };
        unsafe { core::slice::from_raw_parts(p as *const u8, len) }
    }

    unsafe fn c_str_to_string(p: *const c_char) -> String {
        if p.is_null() {
            return String::new();
        }
        unsafe { CStr::from_ptr(p) }.to_string_lossy().to_string()
    }

    unsafe fn write_bytes_with_nul(dst: *mut c_char, bytes: &[u8]) -> c_int {
        if dst.is_null() {
            return 0;
        }
        let mut i = 0usize;
        for &b in bytes {
            unsafe {
                *dst.add(i) = b as c_char;
            }
            i += 1;
        }
        unsafe {
            *dst.add(i) = 0;
        }
        i as c_int
    }

    // ----------------------------- memory -----------------------------------

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
        if size == 0 {
            return ptr::null_mut();
        }

        let total = size.saturating_add(HEADER_SIZE);
        let layout = match Layout::from_size_align(total, HEADER_ALIGN) {
            Ok(l) => l,
            Err(_) => return ptr::null_mut(),
        };

        let base = unsafe { alloc(layout) };
        if base.is_null() {
            return ptr::null_mut();
        }

        unsafe {
            *(base as *mut usize) = size;
            base.add(HEADER_SIZE) as *mut c_void
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn calloc(nmemb: usize, size: usize) -> *mut c_void {
        let bytes = match nmemb.checked_mul(size) {
            Some(b) => b,
            None => return ptr::null_mut(),
        };
        let p = unsafe { malloc(bytes) };
        if !p.is_null() {
            unsafe { ptr::write_bytes(p, 0, bytes) };
        }
        p
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn free(ptr_: *mut c_void) {
        if ptr_.is_null() {
            return;
        }
        let base = unsafe { (ptr_ as *mut u8).sub(HEADER_SIZE) };
        let size = unsafe { *(base as *const usize) };
        let total = size.saturating_add(HEADER_SIZE);
        if let Ok(layout) = Layout::from_size_align(total, HEADER_ALIGN) {
            unsafe { dealloc(base, layout) };
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn realloc(ptr_: *mut c_void, new_size: usize) -> *mut c_void {
        if ptr_.is_null() {
            return unsafe { malloc(new_size) };
        }
        let base = unsafe { (ptr_ as *mut u8).sub(HEADER_SIZE) };
        let old_size = unsafe { *(base as *const usize) };
        let old_total = old_size.saturating_add(HEADER_SIZE);
        let new_total = new_size.saturating_add(HEADER_SIZE);

        let old_layout = match Layout::from_size_align(old_total, HEADER_ALIGN) {
            Ok(l) => l,
            Err(_) => return ptr::null_mut(),
        };

        let new_base = unsafe { std_realloc(base, old_layout, new_total) };
        if new_base.is_null() {
            return ptr::null_mut();
        }
        unsafe {
            *(new_base as *mut usize) = new_size;
            new_base.add(HEADER_SIZE) as *mut c_void
        }
    }

    // ----------------------------- env --------------------------------------

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn getenv(_name: *const c_char) -> *mut c_char {
        ptr::null_mut()
    }

    // ----------------------------- parsing ----------------------------------

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn tolower(c: c_int) -> c_int {
        if (b'A' as c_int..=b'Z' as c_int).contains(&c) {
            c + 32
        } else {
            c
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn isdigit(c: c_int) -> c_int {
        if (b'0' as c_int..=b'9' as c_int).contains(&c) {
            1
        } else {
            0
        }
    }

    fn parse_i64(s: &str) -> i64 {
        let s = s.trim_start();
        let mut chars = s.chars();
        let mut sign = 1i64;
        if let Some(c) = chars.clone().next() {
            if c == '-' {
                sign = -1;
                chars.next();
            } else if c == '+' {
                chars.next();
            }
        }
        let mut acc = 0i64;
        for c in chars {
            if !c.is_ascii_digit() {
                break;
            }
            acc = acc
                .saturating_mul(10)
                .saturating_add((c as u8 - b'0') as i64);
        }
        acc * sign
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn atoi(s: *const c_char) -> c_int {
        parse_i64(&unsafe { c_str_to_string(s) }) as c_int
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn atol(s: *const c_char) -> c_int {
        parse_i64(&unsafe { c_str_to_string(s) }) as c_int
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn atof(s: *const c_char) -> f64 {
        let st = unsafe { c_str_to_string(s) };
        st.trim().parse::<f64>().unwrap_or(0.0)
    }

    // ----------------------------- strings ----------------------------------

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char {
        if dst.is_null() || src.is_null() {
            return dst;
        }
        let mut i = 0usize;
        loop {
            let b = unsafe { *src.add(i) };
            unsafe { *dst.add(i) = b };
            i += 1;
            if b == 0 {
                break;
            }
        }
        dst
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn strncpy(
        dst: *mut c_char,
        src: *const c_char,
        n: usize,
    ) -> *mut c_char {
        if dst.is_null() || src.is_null() {
            return dst;
        }
        let mut i = 0usize;
        while i < n {
            let b = unsafe { *src.add(i) };
            unsafe { *dst.add(i) = b };
            i += 1;
            if b == 0 {
                break;
            }
        }
        while i < n {
            unsafe { *dst.add(i) = 0 };
            i += 1;
        }
        dst
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char {
        if dst.is_null() || src.is_null() {
            return dst;
        }
        let mut d = dst;
        while unsafe { *d } != 0 {
            d = unsafe { d.add(1) };
        }
        let mut s = src;
        loop {
            let b = unsafe { *s };
            unsafe { *d = b };
            if b == 0 {
                break;
            }
            d = unsafe { d.add(1) };
            s = unsafe { s.add(1) };
        }
        dst
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn strcmp(a: *const c_char, b: *const c_char) -> c_int {
        let mut i = 0usize;
        loop {
            let ca = unsafe { *a.add(i) as u8 };
            let cb = unsafe { *b.add(i) as u8 };
            if ca != cb {
                return ca as c_int - cb as c_int;
            }
            if ca == 0 {
                return 0;
            }
            i += 1;
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int {
        let mut i = 0usize;
        while i < n {
            let ca = unsafe { *a.add(i) as u8 };
            let cb = unsafe { *b.add(i) as u8 };
            if ca != cb {
                return ca as c_int - cb as c_int;
            }
            if ca == 0 {
                return 0;
            }
            i += 1;
        }
        0
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn strchr(s: *const c_char, c: c_int) -> *mut c_char {
        if s.is_null() {
            return ptr::null_mut();
        }
        let mut p = s;
        let c = c as u8;
        loop {
            let b = unsafe { *p as u8 };
            if b == c {
                return p as *mut c_char;
            }
            if b == 0 {
                return ptr::null_mut();
            }
            p = unsafe { p.add(1) };
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn strrchr(s: *const c_char, c: c_int) -> *mut c_char {
        if s.is_null() {
            return ptr::null_mut();
        }
        let c = c as u8;
        let mut last: *const c_char = ptr::null();
        let mut p = s;
        loop {
            let b = unsafe { *p as u8 };
            if b == c {
                last = p;
            }
            if b == 0 {
                break;
            }
            p = unsafe { p.add(1) };
        }
        last as *mut c_char
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
        if haystack.is_null() || needle.is_null() {
            return ptr::null_mut();
        }
        let h = unsafe { c_str_bytes(haystack) };
        let n = unsafe { c_str_bytes(needle) };
        if n.is_empty() {
            return haystack as *mut c_char;
        }
        if let Some(pos) = h.windows(n.len()).position(|w| w == n) {
            unsafe { haystack.add(pos) as *mut c_char }
        } else {
            ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn strpbrk(s: *const c_char, accept: *const c_char) -> *mut c_char {
        if s.is_null() || accept.is_null() {
            return ptr::null_mut();
        }
        let acc = unsafe { c_str_bytes(accept) };
        let mut p = s;
        loop {
            let b = unsafe { *p as u8 };
            if b == 0 {
                return ptr::null_mut();
            }
            if acc.contains(&b) {
                return p as *mut c_char;
            }
            p = unsafe { p.add(1) };
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn strdup(s: *const c_char) -> *mut c_char {
        if s.is_null() {
            return ptr::null_mut();
        }
        let len = unsafe { c_strlen(s) };
        let out = unsafe { malloc(len + 1) } as *mut c_char;
        if out.is_null() {
            return ptr::null_mut();
        }
        unsafe {
            ptr::copy_nonoverlapping(s, out, len + 1);
        }
        out
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn sprintf(
        dst: *mut c_char,
        fmt: *const c_char,
        arg1: *const c_char,
    ) -> c_int {
        if dst.is_null() || fmt.is_null() {
            return 0;
        }

        let fmt_s = unsafe { c_str_to_string(fmt) };
        let arg1_s = if arg1.is_null() {
            String::new()
        } else {
            unsafe { c_str_to_string(arg1) }
        };

        let out = if let Some(idx) = fmt_s.find("%s") {
            let mut s = String::new();
            s.push_str(&fmt_s[..idx]);
            s.push_str(&arg1_s);
            s.push_str(&fmt_s[idx + 2..]);
            s
        } else {
            fmt_s
        };

        unsafe { write_bytes_with_nul(dst, out.as_bytes()) }
    }

    // ----------------------------- stdio ------------------------------------

    const SEEK_SET: c_int = 0;
    const SEEK_CUR: c_int = 1;
    const SEEK_END: c_int = 2;

    unsafe fn with_file_bytes<T>(h: &VfsHandle, f: impl FnOnce(&[u8]) -> T) -> Option<T> {
        if h.name.is_null() {
            return None;
        }
        let name = unsafe { CStr::from_ptr(h.name) }
            .to_string_lossy()
            .to_string();
        EPHE_FILES.with(|m| m.borrow().get(&name).map(|v| f(v.as_slice())))
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn ftell(stream: *mut c_void) -> c_int {
        if stream.is_null() {
            return -1;
        }
        let h = unsafe { &*(stream as *const VfsHandle) };
        h.pos as c_int
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn fseek(stream: *mut c_void, offset: c_int, whence: c_int) -> c_int {
        if stream.is_null() {
            return -1;
        }
        let h = unsafe { &mut *(stream as *mut VfsHandle) };

        let len = unsafe { with_file_bytes(h, |b| b.len()).unwrap_or(0usize) };

        let base: isize = match whence {
            SEEK_SET => 0,
            SEEK_CUR => h.pos as isize,
            SEEK_END => len as isize,
            _ => return -1,
        };

        let new_pos = base.saturating_add(offset as isize);
        if new_pos < 0 {
            return -1;
        }
        let new_pos = new_pos as usize;
        h.pos = new_pos.min(len);
        0
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn rewind(stream: *mut c_void) {
        if stream.is_null() {
            return;
        }
        let h = unsafe { &mut *(stream as *mut VfsHandle) };
        h.pos = 0;
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn fread(
        ptr_: *mut c_void,
        size: usize,
        count: usize,
        stream: *mut c_void,
    ) -> usize {
        if ptr_.is_null() || stream.is_null() || size == 0 || count == 0 {
            return 0;
        }
        let h = unsafe { &mut *(stream as *mut VfsHandle) };

        let want = size.saturating_mul(count);
        let start_pos = h.pos;
        let Some(read_bytes) = (unsafe {
            with_file_bytes(h, |b| {
                if start_pos >= b.len() {
                    return 0usize;
                }
                let avail = b.len() - start_pos;
                let n = avail.min(want);
                ptr::copy_nonoverlapping(b.as_ptr().add(start_pos), ptr_ as *mut u8, n);
                n
            })
        }) else {
            return 0;
        };

        h.pos = start_pos.saturating_add(read_bytes);
        read_bytes / size
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn fgets(s: *mut c_char, n: c_int, stream: *mut c_void) -> *mut c_char {
        if s.is_null() || stream.is_null() || n <= 0 {
            return ptr::null_mut();
        }

        let h = unsafe { &mut *(stream as *mut VfsHandle) };
        let start_pos = h.pos;

        let Some(got) = (unsafe {
            with_file_bytes(h, |b| {
                if start_pos >= b.len() {
                    return 0usize;
                }

                let max = (n as usize).saturating_sub(1);
                let mut i = 0usize;
                while i < max && start_pos + i < b.len() {
                    let ch = b[start_pos + i];
                    i += 1;
                    if ch == b'\n' {
                        break;
                    }
                }

                ptr::copy_nonoverlapping(b.as_ptr().add(start_pos), s as *mut u8, i);
                *(s.add(i) as *mut u8) = 0;
                i
            })
        }) else {
            return ptr::null_mut();
        };

        h.pos = start_pos.saturating_add(got);

        if got == 0 { ptr::null_mut() } else { s }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn fclose(stream: *mut c_void) -> c_int {
        if stream.is_null() {
            return 0;
        }

        unsafe { super::vfs_close(stream) };
        0
    }

    // ----------------------------- JPL stubs --------------------------------

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn swi_open_jpl_file(
        _ss: *mut f64,
        _fname: *mut c_char,
        _fpath: *mut c_char,
        serr: *mut c_char,
    ) -> c_int {
        if !serr.is_null() {
            let _ = unsafe { write_bytes_with_nul(serr, b"JPL ephemeris not supported\0") };
        }
        set_last_error("JPL ephemeris not supported in wasm32-unknown-unknown build");
        -1
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn swi_close_jpl_file() {}

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn swi_get_jpl_denum() -> i32 {
        0
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn swi_pleph(
        _et: f64,
        _ntarg: c_int,
        _ncent: c_int,
        _rrd: *mut f64,
        _serr: *mut c_char,
    ) -> c_int {
        -1
    }
}
