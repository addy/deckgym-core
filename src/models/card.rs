use core::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    actions::{abilities::AbilityMechanic, has_ability_mechanic},
    card_ids::CardId,
    database::get_card_by_enum,
};

/// Represents the type of energy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum EnergyType {
    Grass,
    Fire,
    Water,
    Lightning,
    Psychic,
    Fighting,
    Darkness,
    Metal,
    Dragon,
    Colorless,
}
impl EnergyType {
    /// The 8 Energy types the Energy Zone can generate, i.e. the ones a deck can be built
    /// around. See `is_selectable`.
    pub const SELECTABLE: [EnergyType; 8] = [
        EnergyType::Grass,
        EnergyType::Fire,
        EnergyType::Water,
        EnergyType::Lightning,
        EnergyType::Psychic,
        EnergyType::Fighting,
        EnergyType::Darkness,
        EnergyType::Metal,
    ];

    pub(crate) fn from_str(energy_type: &str) -> Option<Self> {
        match energy_type {
            "Grass" => Some(EnergyType::Grass),
            "Fire" => Some(EnergyType::Fire),
            "Water" => Some(EnergyType::Water),
            "Lightning" => Some(EnergyType::Lightning),
            "Psychic" => Some(EnergyType::Psychic),
            "Fighting" => Some(EnergyType::Fighting),
            "Darkness" => Some(EnergyType::Darkness),
            "Metal" => Some(EnergyType::Metal),
            "Dragon" => Some(EnergyType::Dragon),
            "Colorless" => Some(EnergyType::Colorless),
            _ => None,
        }
    }

    /// Whether a deck can be built around this energy, i.e. whether the Energy Zone
    /// can generate it.
    ///
    /// Dragon and Colorless cannot. Colorless only ever appears as an attack cost
    /// (payable by any energy), and there is no Dragon energy in the game — Dragon
    /// Pokemon are powered by other types.
    pub fn is_selectable(&self) -> bool {
        !matches!(self, EnergyType::Dragon | EnergyType::Colorless)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EnergyType::Grass => "Grass",
            EnergyType::Fire => "Fire",
            EnergyType::Water => "Water",
            EnergyType::Lightning => "Lightning",
            EnergyType::Psychic => "Psychic",
            EnergyType::Fighting => "Fighting",
            EnergyType::Darkness => "Darkness",
            EnergyType::Metal => "Metal",
            EnergyType::Dragon => "Dragon",
            EnergyType::Colorless => "Colorless",
        }
    }
}

/// Represents an attack of a card.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Attack {
    pub energy_required: Vec<EnergyType>,
    pub title: String,
    pub fixed_damage: u32,
    pub effect: Option<String>,
}

/// Represents an attack of a card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ability {
    pub title: String,
    pub effect: String,
}

pub const BASIC_STAGE: u8 = 0;

/// Represents the data of a single pokemon card.
#[derive(Clone, Serialize, Deserialize)]
pub struct PokemonCardData {
    pub id: String,
    pub name: String,
    pub stage: u8, // 0 for Basic, 1 for Stage 1, 2 for Stage 2
    pub evolves_from: Option<String>,
    pub hp: u32,
    pub energy_type: EnergyType,
    /// Second printed Pokémon type; independent of attached Energy and attack costs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_energy_type: Option<EnergyType>,
    pub ability: Option<Ability>,
    pub attacks: Vec<Attack>,
    pub weakness: Option<EnergyType>,
    pub retreat_cost: Vec<EnergyType>,
    pub rarity: String,
    pub booster_pack: String,
}
impl PokemonCard {
    pub fn get_types(&self) -> Vec<EnergyType> {
        let mut types = vec![self.energy_type];
        if let Some(secondary) = self.secondary_energy_type {
            if !types.contains(&secondary) {
                types.push(secondary);
            }
        }
        types
    }

    pub fn is_type(&self, energy_type: EnergyType) -> bool {
        self.energy_type == energy_type || self.secondary_energy_type == Some(energy_type)
    }
}


/// A Pokémon card: a shared reference to immutable card data, so cloning a card (and the
/// states and actions that hold cards) copies a pointer instead of every string and list on
/// it. Fields are read through `Deref`. Equality, hashing, `Debug` and serde behave as they
/// did when the card owned its data.
#[derive(Clone)]
pub struct PokemonCard(&'static PokemonCardData);
impl Deref for PokemonCard {
    type Target = PokemonCardData;
    fn deref(&self) -> &PokemonCardData {
        self.0
    }
}
/// Card data that lives for the rest of the program (tests, tools): leaked once. Database
/// cards are borrowed from the database instead (`get_card_by_enum`).
impl From<PokemonCardData> for PokemonCard {
    fn from(data: PokemonCardData) -> Self {
        PokemonCard(Box::leak(Box::new(data)))
    }
}
impl PartialEq for PokemonCard {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for PokemonCard {}
impl Hash for PokemonCard {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl Serialize for PokemonCard {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for PokemonCard {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = PokemonCardData::deserialize(deserializer)?;
        match database_card(&data.id) {
            Some(Card::Pokemon(card)) => Ok(card),
            _ => Ok(data.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainerType {
    Supporter,
    Item,
    Tool,
    Fossil,
    Stadium,
}

/// Represents the data of a single trainer card.
#[derive(Clone, Serialize, Deserialize)]
pub struct TrainerCardData {
    pub id: String,
    pub trainer_card_type: TrainerType,
    pub name: String,
    pub effect: String,
    pub rarity: String,
    pub booster_pack: String,
}

/// A Trainer card: a shared reference to immutable card data (see `PokemonCard`).
#[derive(Clone)]
pub struct TrainerCard(&'static TrainerCardData);
impl Deref for TrainerCard {
    type Target = TrainerCardData;
    fn deref(&self) -> &TrainerCardData {
        self.0
    }
}
/// Card data that lives for the rest of the program (tests, tools): leaked once.
impl From<TrainerCardData> for TrainerCard {
    fn from(data: TrainerCardData) -> Self {
        TrainerCard(Box::leak(Box::new(data)))
    }
}
impl PartialEq for TrainerCard {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for TrainerCard {}
impl Hash for TrainerCard {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl Serialize for TrainerCard {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for TrainerCard {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = TrainerCardData::deserialize(deserializer)?;
        match database_card(&data.id) {
            Some(Card::Trainer(card)) => Ok(card),
            _ => Ok(data.into()),
        }
    }
}

/// The database's card for a printed id, so deserialized cards share the database's data
/// (cards are equal by id) instead of leaking a copy each.
fn database_card(id: &str) -> Option<Card> {
    CardId::from_card_id(id).map(get_card_by_enum)
}

/// A card definition as the generated database spells it; `Card` borrows it.
#[derive(Clone, Serialize, Deserialize)]
pub enum CardData {
    Pokemon(PokemonCardData),
    Trainer(TrainerCardData),
}
/// Card data that lives for the rest of the program: leaked once (see `PokemonCard`).
impl From<CardData> for Card {
    fn from(data: CardData) -> Self {
        match data {
            CardData::Pokemon(data) => Card::Pokemon(data.into()),
            CardData::Trainer(data) => Card::Trainer(data.into()),
        }
    }
}
impl From<&'static CardData> for Card {
    fn from(data: &'static CardData) -> Self {
        match data {
            CardData::Pokemon(data) => Card::Pokemon(PokemonCard(data)),
            CardData::Trainer(data) => Card::Trainer(TrainerCard(data)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Card {
    Pokemon(PokemonCard),
    Trainer(TrainerCard),
}
impl Hash for Card {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Card::Pokemon(pokemon_card) => pokemon_card.id.hash(state),
            Card::Trainer(trainer_card) => trainer_card.id.hash(state),
        }
    }
}
impl Card {
    pub fn get_id(&self) -> String {
        match self {
            Card::Pokemon(pokemon_card) => pokemon_card.id.clone(),
            Card::Trainer(trainer_card) => trainer_card.id.clone(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Card::Pokemon(pokemon_card) => pokemon_card.name.clone(),
            Card::Trainer(trainer_card) => trainer_card.name.clone(),
        }
    }

    pub(crate) fn get_attacks(&self) -> Vec<Attack> {
        match self {
            Card::Pokemon(pokemon_card) => pokemon_card.attacks.clone(),
            _ => vec![],
        }
    }

    pub fn get_retreat_cost(&self) -> Option<Vec<EnergyType>> {
        match self {
            Card::Pokemon(pokemon_card) => Some(pokemon_card.retreat_cost.clone()),
            _ => None, // Fossils
        }
    }

    pub(crate) fn is_ex(&self) -> bool {
        // A pokemon is EX if after splitting by spaces in the name, the last word is "EX"
        match self {
            Card::Pokemon(pokemon_card) => {
                pokemon_card.name.to_lowercase().split(' ').next_back() == Some("ex")
            }
            _ => false,
        }
    }

    pub(crate) fn is_mega(&self) -> bool {
        // A pokemon is Mega if the name starts with "Mega "
        match self {
            Card::Pokemon(pokemon_card) => pokemon_card.name.starts_with("Mega "),
            _ => false,
        }
    }

    pub(crate) fn get_knockout_points(&self) -> u8 {
        // Mega pokemon are worth 3 points, ex pokemon are worth 2, regular pokemon are worth 1
        if self.is_mega() {
            3
        } else if self.is_ex() {
            2
        } else {
            1
        }
    }

    pub(crate) fn get_ability(&self) -> Option<Ability> {
        match self {
            Card::Pokemon(pokemon_card) => pokemon_card.ability.clone(),
            _ => None,
        }
    }

    pub(crate) fn is_support(&self) -> bool {
        match self {
            Card::Trainer(trainer_card) => trainer_card.trainer_card_type == TrainerType::Supporter,
            _ => false,
        }
    }

    /// The Energy types this card is **printed** as.
    ///
    /// Includes both printed types for dual-type Pokémon.
    ///
    /// Note this is the *printed* type set — it deliberately knows nothing about Abilities, which
    /// only apply while the card is in play. Use it for cards in a deck, hand or discard pile;
    /// use `PlayedCard::get_energy_types` for cards on the board.
    pub fn get_types(&self) -> Vec<EnergyType> {
        match self {
            Card::Pokemon(pokemon_card) => pokemon_card.get_types(),
            _ => vec![],
        }
    }

    /// Whether this card's **printed** type set contains `energy_type`. Allocation-free
    /// counterpart of `get_types().contains(..)`.
    pub fn is_type(&self, energy_type: EnergyType) -> bool {
        match self {
            Card::Pokemon(pokemon_card) => pokemon_card.is_type(energy_type),
            _ => false,
        }
    }

    pub fn get_card_id(&self) -> CardId {
        CardId::from_card_id(self.get_id().as_str()).expect("Card ID should be valid")
    }

    pub fn is_basic(&self) -> bool {
        match self {
            Card::Pokemon(pokemon_card) => pokemon_card.stage == BASIC_STAGE,
            _ => false,
        }
    }

    pub fn is_fossil(&self) -> bool {
        match self {
            Card::Trainer(trainer_card) => trainer_card.trainer_card_type == TrainerType::Fossil,
            _ => false,
        }
    }

    pub fn as_trainer(&self) -> TrainerCard {
        match self {
            Card::Trainer(trainer_card) => trainer_card.clone(),
            _ => panic!("Card is not a Trainer"),
        }
    }

    /// Check if this card can evolve into the given evolution card
    /// This handles special evolution rules like Eevee ex's Veevee 'volve ability
    ///
    /// This is a card-level evolution-legality rule with no access to the board, so the Veevee
    /// 'volve lookup below stays card-level too: it is deliberately *not* gated on Alolan Muk's
    /// Power of Alchemy, even though Eevee ex is a Basic. Making it board-aware would mean
    /// threading `State` (or a `PlayedCard`) through every evolution-legality caller.
    pub fn can_evolve_into(&self, evolution_card: &Card) -> bool {
        if let Card::Pokemon(evolution_pokemon) = evolution_card {
            if let Some(evolves_from) = &evolution_pokemon.evolves_from {
                // Normal evolution: the evolution card evolves from this card's name
                if self.get_name() == *evolves_from {
                    return true;
                }

                // Special case: Eevee ex's Veevee 'volve ability
                // Allows Eevee ex to evolve into any Pokemon that evolves from "Eevee"
                if has_ability_mechanic(self, &AbilityMechanic::CanEvolveIntoEeveeEvolution)
                    && evolves_from == "Eevee"
                {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusCondition {
    Poisoned,
    Paralyzed,
    Asleep,
    Burned,
    Confused,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Card::Pokemon(pokemon_card) => write!(f, "{}", pokemon_card.name),
            Card::Trainer(trainer_card) => write!(f, "{}", trainer_card.name),
        }
    }
}

impl fmt::Display for EnergyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnergyType::Grass => write!(f, "Grass"),
            EnergyType::Fire => write!(f, "Fire"),
            EnergyType::Water => write!(f, "Water"),
            EnergyType::Lightning => write!(f, "Lightning"),
            EnergyType::Psychic => write!(f, "Psychic"),
            EnergyType::Fighting => write!(f, "Fighting"),
            EnergyType::Darkness => write!(f, "Darkness"),
            EnergyType::Metal => write!(f, "Metal"),
            EnergyType::Dragon => write!(f, "Dragon"),
            EnergyType::Colorless => write!(f, "Colorless"),
        }
    }
}

impl fmt::Debug for PokemonCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{} {}", self.id, self.name)
        }
    }
}

impl fmt::Debug for TrainerCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{} {}", self.id, self.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data_of(card: &Card) -> *const () {
        match card {
            Card::Pokemon(p) => p.0 as *const PokemonCardData as *const (),
            Card::Trainer(t) => t.0 as *const TrainerCardData as *const (),
        }
    }

    #[test]
    fn cards_share_the_database_data() {
        for id in [CardId::A1001Bulbasaur, CardId::PA001Potion] {
            let card = get_card_by_enum(id);
            assert_eq!(data_of(&card), data_of(&card.clone()));
            assert_eq!(data_of(&card), data_of(&get_card_by_enum(id)));
        }
    }

    #[test]
    fn serde_round_trip_keeps_the_json_and_the_shared_data() {
        for id in [CardId::A1001Bulbasaur, CardId::PA001Potion] {
            let card = get_card_by_enum(id);
            let json = serde_json::to_string(&card).unwrap();
            let back: Card = serde_json::from_str(&json).unwrap();
            assert_eq!(back, card);
            assert_eq!(data_of(&back), data_of(&card));
            assert_eq!(serde_json::to_string(&back).unwrap(), json);
        }
        let bulbasaur = serde_json::to_string(&get_card_by_enum(CardId::A1001Bulbasaur)).unwrap();
        assert!(bulbasaur.starts_with(r#"{"Pokemon":{"id":"A1 001","name":"Bulbasaur","#));
    }

    #[test]
    fn cards_outside_the_database_round_trip_too() {
        let json = r#"{"Trainer":{"id":"X 1","trainer_card_type":"Item","name":"Test","effect":"","rarity":"","booster_pack":""}}"#;
        let card: Card = serde_json::from_str(json).unwrap();
        assert_eq!(card.get_name(), "Test");
        assert_eq!(serde_json::to_string(&card).unwrap(), json);
    }
}
