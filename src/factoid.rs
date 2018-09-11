use rand::prelude::*;
use rand::prng::XorShiftRng;
use rand::rngs::SmallRng;
use rand::FromEntropy;

use std::collections::HashMap;
use std::io::Error;

pub struct Brain {
    name: String,
    hippocampus: HashMap<String, Vec<String>>,
    verbs: Vec<String>,
    rng: XorShiftRng,
}

impl Brain {
    pub fn new(name: String, verbs: Vec<String>) -> Brain {
        Brain {
            hippocampus: HashMap::new(),
            name: name,
            verbs: verbs,
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
            .position(|pivot| self.verbs.contains(&pivot.to_string()))
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

// TODO needs to split on whitespass + punctuassion
pub fn creates_factoid(name: &String, verbs: &Vec<String>, s: &String) -> bool {
    if !s.starts_with((name.to_owned() + ":").as_str()) {
        return false;
    }

    verbs.iter().any(|x| s.contains(x.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_create_factoid() {
        let verbs = vec!["is".to_owned()];
        let mut brain = Brain::new("sidra".to_owned(), verbs);
        brain.create_factoid("sidra: foo is bar".to_string());
        assert_eq!(
            brain.hippocampus.get("foo").unwrap(),
            &vec!["bar".to_string()]
        );
    }

    #[test]
    fn can_retrieve_factoid() {
        let verbs = vec!["is".to_owned()];
        let mut brain = Brain::new("sidra".to_owned(), verbs);
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
        let verbs = vec!["is".to_owned()];
        let mut brain = Brain::new("sidra".to_owned(), verbs);

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
        let verbs = vec!["is".to_owned()];
        let mut brain = Brain::new("sidra".to_owned(), verbs);
        brain
            .hippocampus
            .insert("foo".to_string(), vec!["bar".to_string()]);
        assert!(brain.get_factoid(&"bar".to_string()).is_none());
    }

    #[test]
    fn can_literal_factoid() {
        let verbs = vec!["is".to_owned()];
        let mut brain = Brain::new("sidra".to_owned(), verbs);
        brain
            .hippocampus
            .insert("foo".to_string(), vec!["bar".to_string()]);
        assert_eq!("bar".to_string(), brain.literal_factoid(&"foo".to_string()));

        assert_eq!(
            "I don't know anything about that".to_string(),
            brain.literal_factoid(&"zip".to_string())
        );
    }

    #[test]
    fn ids_factoid_creation() {
        let verbs = vec!["is".to_owned()];
        assert!(creates_factoid(
            &"bot_name".to_string(),
            &verbs,
            &"bot_name: a is b".to_string()
        ));
    }

    #[test]
    fn ids_non_factoid_creation() {
        let verbs = vec!["is".to_owned()];
        assert!(
            !creates_factoid(&"bot_name".to_string(), &verbs, &"a is b".to_string()),
            "I wasn't addressed, this shouldn't create a factoid"
        );

        assert!(
            !creates_factoid(
                &"bot_name".to_string(),
                &verbs,
                &"bot_name: a foo b".to_string()
            ),
            "None of my verbs were present, this shouldn't create a factoid"
        );
    }
}
