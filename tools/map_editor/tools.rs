#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EditorTool {
    Draw,
    Erase,
    SpawnPoint,
    Item,
    JumpPad,
    Teleporter,
    TeleporterDestination,
    Light,
    Select,
    Background,
}

#[derive(Clone, PartialEq)]
pub enum SelectedObject {
    SpawnPoint(usize),
    Item(usize),
    JumpPad(usize),
    Teleporter(usize),
    Light(usize),
    BackgroundElement(usize),
}

#[derive(Clone, Copy)]
pub enum ItemPlaceType {
    Health25,
    Health50,
    Health100,
    Armor50,
    Armor100,
    Shotgun,
    GrenadeLauncher,
    RocketLauncher,
    LightningGun,
    Railgun,
    Plasmagun,
    BFG,
    Quad,
    Regen,
    Battle,
    Flight,
    Haste,
    Invis,
}

impl ItemPlaceType {
    pub fn to_string(&self) -> String {
        match self {
            ItemPlaceType::Health25 => "Health25",
            ItemPlaceType::Health50 => "Health50",
            ItemPlaceType::Health100 => "Health100",
            ItemPlaceType::Armor50 => "Armor50",
            ItemPlaceType::Armor100 => "Armor100",
            ItemPlaceType::Shotgun => "Shotgun",
            ItemPlaceType::GrenadeLauncher => "GrenadeLauncher",
            ItemPlaceType::RocketLauncher => "RocketLauncher",
            ItemPlaceType::LightningGun => "LightningGun",
            ItemPlaceType::Railgun => "Railgun",
            ItemPlaceType::Plasmagun => "Plasmagun",
            ItemPlaceType::BFG => "BFG",
            ItemPlaceType::Quad => "Quad",
            ItemPlaceType::Regen => "Regen",
            ItemPlaceType::Battle => "Battle",
            ItemPlaceType::Flight => "Flight",
            ItemPlaceType::Haste => "Haste",
            ItemPlaceType::Invis => "Invis",
        }
        .to_string()
    }

    pub fn next(&self) -> Self {
        match self {
            ItemPlaceType::Health25 => ItemPlaceType::Health50,
            ItemPlaceType::Health50 => ItemPlaceType::Health100,
            ItemPlaceType::Health100 => ItemPlaceType::Armor50,
            ItemPlaceType::Armor50 => ItemPlaceType::Armor100,
            ItemPlaceType::Armor100 => ItemPlaceType::Shotgun,
            ItemPlaceType::Shotgun => ItemPlaceType::GrenadeLauncher,
            ItemPlaceType::GrenadeLauncher => ItemPlaceType::RocketLauncher,
            ItemPlaceType::RocketLauncher => ItemPlaceType::LightningGun,
            ItemPlaceType::LightningGun => ItemPlaceType::Railgun,
            ItemPlaceType::Railgun => ItemPlaceType::Plasmagun,
            ItemPlaceType::Plasmagun => ItemPlaceType::BFG,
            ItemPlaceType::BFG => ItemPlaceType::Quad,
            ItemPlaceType::Quad => ItemPlaceType::Regen,
            ItemPlaceType::Regen => ItemPlaceType::Battle,
            ItemPlaceType::Battle => ItemPlaceType::Flight,
            ItemPlaceType::Flight => ItemPlaceType::Haste,
            ItemPlaceType::Haste => ItemPlaceType::Invis,
            ItemPlaceType::Invis => ItemPlaceType::Health25,
        }
    }
}
