use rand::seq::SliceRandom;
use rand::Rng;

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

/// Generate a random worktree name in format: adjective-noun-suffix
/// Example: swift-falcon-a3b2
pub fn generate() -> String {
    let mut rng = rand::thread_rng();

    let adjective = ADJECTIVES.choose(&mut rng).unwrap();
    let noun = NOUNS.choose(&mut rng).unwrap();

    // Generate a 4-character hex suffix
    let suffix: String = (0..4)
        .map(|_| {
            let idx = rng.gen_range(0..16);
            "0123456789abcdef".chars().nth(idx).unwrap()
        })
        .collect();

    format!("{}-{}-{}", adjective, noun, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_format() {
        let name = generate();
        let parts: Vec<&str> = name.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert!(ADJECTIVES.contains(&parts[0]));
        assert!(NOUNS.contains(&parts[1]));
        assert_eq!(parts[2].len(), 4);
    }

    #[test]
    fn test_generate_unique() {
        let name1 = generate();
        let name2 = generate();
        // With 56 adjectives * 56 nouns * 65536 suffixes, collision is very unlikely
        assert_ne!(name1, name2);
    }
}
