use rand::prelude::*;
use rand::prng::XorShiftRng;
use rand::rngs::SmallRng;
use rand::FromEntropy;

use std::collections::HashMap;
use std::io::Error;

pub struct Brain {
    name: String,
    hippocampus: HashMap<String, Vec<String>>,
    rng: XorShiftRng,
}

impl Brain {
    pub fn new(name: String) -> Brain {
        Brain {
            hippocampus: HashMap::new(),
            name: name,
            rng: XorShiftRng::from_entropy(),
        }
    }

    pub fn set_rng_seed(&mut self, seed: [u8; 16]) {
        self.rng = XorShiftRng::from_seed(seed);
    }
}

pub trait FactoidKnowledge {
    fn create_factoid(&mut self, String) -> Result<(), Error>;
    fn get_factoid<'a>(&'a mut self, &String) -> Option<&'a String>;
    fn literal_factoid(&self, &String) -> String;
}

// TODO strip whitespass + punctuassion
impl FactoidKnowledge for Brain {
    fn create_factoid(&mut self, s: String) -> Result<(), Error> {
        // Drop name:
        let name_index = s.find(":").unwrap();
        let cleaned_string = s.clone().split_off(name_index + 1);

        let iter = cleaned_string.split_whitespace();
        let index = iter
            .clone()
            .position(|pivot| pivot == "is" || pivot == "are" )
            .unwrap();

        let tmp: Vec<&str> = iter.collect();
        let (k, v) = tmp.split_at(index);

        let full_key = k.join(" ").to_owned();
        let full_val = v[1..].join(" ").to_owned();

        self.hippocampus
            .entry(full_key)
            .or_insert(vec![])
            .push(full_val);

    Ok(())
    }

    fn get_factoid<'a>(&'a mut self, k: &String) -> Option<&'a String> {
        if let Some(vals) = self.hippocampus.get(k) {
            self.rng.choose(&vals)
        } else {
            None
        }
    }

    // TODO one can not know things in many ways!
    fn literal_factoid(&self, k: &String) -> String {
        match self.hippocampus.get(k) {
            Some(v) => v.join(", "),
            None => "I don't know anything about that".to_string(),
        }
    }
}

// e.g. awoo -> awooooo or meow -> meoooow
fn is_extension(base: &mut Iterator<Item=char>, mut candidate: &mut Iterator<Item=char>)  -> bool {
    let h = match base.next() {
        Some(chr) => chr,
        None => {
            // Base is out of characters, is this good or bad?
            return candidate.next() == None;
        }
    };

    let mut candidate_remainder = candidate.skip_while(|x| x == &h);
    is_extension(base, &mut candidate_remainder)
}

pub fn is_awoo(s: &str) -> bool {
    let lower_s = s.to_ascii_lowercase();
    is_extension(&mut "awoo".chars(), &mut lower_s.chars())
}

#[test]
pub fn test_awoos() {
    assert!(is_awoo("awoo"));
    assert!(is_awoo("aaaawoo"));
    assert!(is_awoo("aaawwwwoooo"));
    // TODO: this didn't work before, still doesn't,
    // assert!(!is_awoo("awo"));
    assert!(!is_awoo("awo0"));
}


pub fn is_meow(s: &str) -> bool {
    let lower_s = s.to_ascii_lowercase();
    is_extension(&mut "meow".chars(), &mut lower_s.chars())
        || is_extension(&mut "miao".chars(), &mut lower_s.chars())
        || is_extension(&mut "miaow".chars(), &mut lower_s.chars())
}

#[test]
pub fn test_meows() {
    assert!(is_meow("meeeow"));
    assert!(is_meow("miao"));
    assert!(is_meow("mmmeeeooowww"));
    // TODO: this didn't work before, still doesn't,
    // assert!(!is_awoo("awo"));
    assert!(!is_meow("me0w"));
    assert!(!is_meow("meowffff"));
}

// TODO needs to split on whitespass + punctuassion
pub fn creates_factoid(name: &String, s: &String) -> bool {
    if !s.starts_with((name.to_owned() + ":").as_str()) {
        return false;
    }

    s.contains(" is ") || s.contains(" are ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_create_factoid() {
        let mut brain = Brain::new("sidra".to_owned());
        brain.create_factoid("sidra: foo is bar".to_string());
        assert_eq!(
            brain.hippocampus.get("foo").unwrap(),
            &vec!["bar".to_string()]
        );
    }

    #[test]
    fn can_retrieve_factoid() {
        let mut brain = Brain::new("sidra".to_owned());
        brain
            .hippocampus
            .insert("foo".to_string(), vec!["bar".to_string()]);
        assert_eq!(
            "bar".to_string(),
            *brain.get_factoid(&"foo".to_string()).unwrap()
        );
    }

    #[test]
    fn can_set_and_retrieve_multi_factoid() {
        let mut brain = Brain::new("sidra".to_owned());

        // Set arbitrarily to make the test work
        brain.set_rng_seed([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        brain.create_factoid("sidra: foo is bar".to_string());
        brain.create_factoid("sidra: foo is zip".to_string());

        assert_eq!(
            "bar".to_string(),
            *brain.get_factoid(&"foo".to_string()).unwrap()
        );

        // Set arbitrarily to make the test work
        brain.set_rng_seed([92, 0, 123, 0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 21, 42]);

        assert_eq!(
            "zip".to_string(),
            *brain.get_factoid(&"foo".to_string()).unwrap()
        )
    }

    #[test]
    fn no_nonfactoid_retrieval() {
        let mut brain = Brain::new("sidra".to_owned());
        brain
            .hippocampus
            .insert("foo".to_string(), vec!["bar".to_string()]);
        assert!(brain.get_factoid(&"bar".to_string()).is_none());
    }

    #[test]
    fn can_literal_factoids() {
        let mut brain = Brain::new("sidra".to_owned());
        brain
            .hippocampus
            .insert("foo".to_string(), vec!["bar".to_string()]);
        assert_eq!("bar".to_string(), brain.literal_factoid(&"foo".to_string()));

        assert_eq!(
            "I don't know anything about that".to_string(),
            brain.literal_factoid(&"zip".to_string())
        );

        brain.create_factoid("sidra: foo is zip".to_string());
        assert_eq!(
            "bar, zip".to_string(),
            brain.literal_factoid(&"foo".to_string())
        );
    }

    #[test]
    fn ids_factoid_creation() {
        assert!(creates_factoid(
            &"bot_name".to_string(),
            &"bot_name: a is b".to_string()
        ));
    }

    #[test]
    fn ids_non_factoid_creation() {
        assert!(
            !creates_factoid(&"bot_name".to_string(), &"a is b".to_string()),
            "I wasn't addressed, this shouldn't create a factoid"
        );

        assert!(
            !creates_factoid(
                &"bot_name".to_string(),
                &"bot_name: a foo b".to_string()
            ),
            "None of my verbs were present, this shouldn't create a factoid"
        );
    }
}
