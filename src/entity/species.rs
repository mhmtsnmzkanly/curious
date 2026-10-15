#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Species {
    Herbivore,
    Carnivore,
    Omnivore,
}
