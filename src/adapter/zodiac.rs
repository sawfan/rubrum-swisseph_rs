pub fn ayanamsa_to_swisseph_sid_mode(ayanamsa: rubrum::Ayanamsa) -> i32 {
    match ayanamsa {
        rubrum::Ayanamsa::Lahiri => swisseph_sys::raw::SE_SIDM_LAHIRI as i32,
        rubrum::Ayanamsa::FaganBradley => swisseph_sys::raw::SE_SIDM_FAGAN_BRADLEY as i32,
        rubrum::Ayanamsa::Raman => swisseph_sys::raw::SE_SIDM_RAMAN as i32,
        rubrum::Ayanamsa::Krishnamurti => swisseph_sys::raw::SE_SIDM_KRISHNAMURTI as i32,
        rubrum::Ayanamsa::Yukteswar => swisseph_sys::raw::SE_SIDM_YUKTESHWAR as i32,
        rubrum::Ayanamsa::TrueChitra => swisseph_sys::raw::SE_SIDM_TRUE_CITRA as i32,
        rubrum::Ayanamsa::TrueRevati => swisseph_sys::raw::SE_SIDM_TRUE_REVATI as i32,
    }
}

pub fn set_sidereal_mode(ayanamsa: rubrum::Ayanamsa) {
    unsafe {
        swisseph_sys::raw::swe_set_sid_mode(ayanamsa_to_swisseph_sid_mode(ayanamsa), 0.0, 0.0);
    }
}
