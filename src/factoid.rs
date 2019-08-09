extern crate rand;
extern crate rand_core;
extern crate rand_xorshift;

mod responder;
mod util;

use rand::prelude::IteratorRandom;
use rand_core::SeedableRng;
use rand_xorshift::XorShiftRng;
use responder::Responder;
use responder::Responders;

pub struct Brain {
    name: String,
    rng: XorShiftRng,
    responders: responder::Responders,
}

impl Brain {
    pub fn new(name: String) -> Brain {
        Brain {
            name: name,
            rng: XorShiftRng::seed_from_u64(69),
            responders: Responders::default(),
        }
    }

    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng = XorShiftRng::seed_from_u64(seed)
    }

    pub fn respond(&mut self, input: &str) -> Option<String> {
        if self.creates_factoid(input) {
            return self.create_factoid(input);
        }

        // Returns None if respond() gave back an empty vec
        self.responders
            .respond(input)
            .into_iter()
            .choose(&mut self.rng)
    }

    pub fn register_responder<T: 'static + Responder>(&mut self, responder: T) {
        self.responders.register_responder(responder)
    }

    fn addressed(&self, input: &str) -> bool {
        input.starts_with((self.name.to_owned() + ":").as_str())
    }
}

pub trait KnowsFactoids {
    fn creates_factoid(&self, _: &str) -> bool;
    fn create_factoid(&mut self, _: &str) -> Option<String>;
}

// TODO strip whitespass + punctuassion
impl KnowsFactoids for Brain {
    fn create_factoid(&mut self, s: &str) -> Option<String> {
        // Drop name:
        let s = s.to_string();
        let name_index = s.find(":").unwrap();
        let cleaned_string = s.clone().split_off(name_index + 1);

        let iter = cleaned_string.split_whitespace();
        let index = match iter
            .clone()
            .position(|pivot| pivot == "is" || pivot == "are")
        {
            Some(i) => i,
            None => return None, // No verb here
        };

        let tmp: Vec<&str> = iter.collect();
        let (k, v) = tmp.split_at(index);

        let full_key = k.join(" ");
        let full_val = v[1..].join(" ");

        let factoid_resp =
            responder::FactoidResponder::new(full_key.clone().as_str(), full_val.clone().as_str());
        self.register_responder(factoid_resp);

        return Some(format!(
            "Ok, now I know that {} {} {}",
            full_key, v[0], full_val
        ));
    }

    fn creates_factoid(&self, s: &str) -> bool {
        self.addressed(s) && (s.contains(" is ") || s.contains(" are "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_factoid() {
        let mut brain = Brain::new("sidra".to_owned());
        brain.create_factoid(&"sidra: foo is bar");
        assert_eq!(brain.respond("foo"), Some("bar".to_string()));
    }

    #[test]
    fn can_set_and_retrieve_multi_factoid() {
        let mut brain = Brain::new("sidra".to_owned());

        // Set arbitrarily to make the test work
        brain.set_rng_seed(0);

        brain.create_factoid(&"sidra: foo is bar".to_string());
        brain.create_factoid(&"sidra: foo is zip".to_string());

        assert_eq!(brain.respond("foo"), Some("bar".to_string()));

        // Set arbitrarily to make the test work
        brain.set_rng_seed(69);

        assert_eq!(brain.respond("foo"), Some("zip".to_string()));
    }

    #[test]
    fn no_nonfactoid_retrieval() {
        let mut brain = Brain::new("sidra".to_owned());
        brain.create_factoid(&"sidra: foo isn't bar".to_string());
        assert!(brain.respond("bar").is_none());
    }

    #[test]
    fn ids_non_factoid_creation() {
        let brain = Brain::new("sidra".to_owned());
        assert!(
            !brain.creates_factoid(&"a is b".to_string()),
            "I wasn't addressed, this shouldn't create a factoid"
        );

        assert!(
            !brain.creates_factoid(&"bot_name: a foo b".to_string()),
            "None of my verbs were present, this shouldn't create a factoid"
        );

        assert!(
            !brain.creates_factoid(&"other_name: a is b".to_string()),
            "Someone else was addressed, this shouldn't create a factoid"
        );
    }
}
