pub fn house_system_to_swisseph(system: rubrum::HouseSystem) -> swisseph::HouseSystemKind {
    match system {
        rubrum::HouseSystem::None => swisseph::HouseSystemKind::Placidus,
        rubrum::HouseSystem::Placidus => swisseph::HouseSystemKind::Placidus,
        rubrum::HouseSystem::Koch => swisseph::HouseSystemKind::Koch,
        rubrum::HouseSystem::Porphyry => swisseph::HouseSystemKind::Porphyrius,
        rubrum::HouseSystem::Regiomontanus => swisseph::HouseSystemKind::Regiomontanus,
        rubrum::HouseSystem::Campanus => swisseph::HouseSystemKind::Campanus,
        rubrum::HouseSystem::Equal => swisseph::HouseSystemKind::Equal,
        rubrum::HouseSystem::WholeSign => swisseph::HouseSystemKind::WholeSign,
        rubrum::HouseSystem::Meridian => {
            swisseph::HouseSystemKind::AxialRotationSystemMeridianSystemZariel
        }
        rubrum::HouseSystem::Alcabitius => swisseph::HouseSystemKind::Alcabitus,
        rubrum::HouseSystem::Morinus => swisseph::HouseSystemKind::Morinus,
        rubrum::HouseSystem::Topocentric => swisseph::HouseSystemKind::PolichPageTopocentricSystem,
    }
}
