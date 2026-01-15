use rand::seq::SliceRandom;

const ADJECTIVES: &[&str] = &[
    "swift", "happy", "cool", "brave", "bright", "calm", "clever", "eager", "fair", "fierce",
    "gentle", "grand", "keen", "kind", "lively", "mighty", "noble", "proud", "quick", "quiet",
    "rapid", "sharp", "sleek", "smart", "solid", "steady", "strong", "true", "vivid", "warm",
    "wild", "wise", "agile", "bold", "crisp", "deft", "epic", "fast", "golden", "humble", "jolly",
    "lunar", "merry", "neat", "olive", "prime", "royal", "silky", "tidy", "ultra", "vital",
    "witty", "young", "zesty", "azure", "coral",
];

const NOUNS: &[&str] = &[
    "falcon", "tiger", "wolf", "eagle", "hawk", "bear", "lion", "fox", "deer", "owl", "swan",
    "crane", "raven", "dove", "heron", "finch", "panda", "koala", "otter", "whale", "shark",
    "dolphin", "seal", "penguin", "cedar", "maple", "oak", "pine", "birch", "willow", "elm", "ash",
    "river", "ocean", "mountain", "valley", "forest", "meadow", "canyon", "island", "comet",
    "nova", "star", "moon", "nebula", "aurora", "galaxy", "quasar", "flame", "storm", "thunder",
    "breeze", "frost", "mist", "cloud", "rain",
];

/// Generate a random worktree name in format: adjective-noun
/// Example: swift-falcon, wise-wolf, lunar-willow
pub fn generate() -> String {
    let mut rng = rand::thread_rng();

    let adjective = ADJECTIVES.choose(&mut rng).unwrap();
    let noun = NOUNS.choose(&mut rng).unwrap();

    format!("{}-{}", adjective, noun)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_format() {
        let name = generate();
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 2);
        assert!(ADJECTIVES.contains(&parts[0]));
        assert!(NOUNS.contains(&parts[1]));
    }

    #[test]
    fn test_generate_produces_valid_names() {
        // Generate several names and verify they're all valid
        for _ in 0..10 {
            let name = generate();
            let parts: Vec<&str> = name.split('-').collect();
            assert_eq!(parts.len(), 2);
            assert!(ADJECTIVES.contains(&parts[0]));
            assert!(NOUNS.contains(&parts[1]));
        }
    }
}
