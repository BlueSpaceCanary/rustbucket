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
        if let Some(factoid) = self.creates_factoid(input) {
            return self.learn_factoid(factoid);
        }

        // Returns None if respond() gave back an empty vec
        self.responders.respond(input).choose(&mut self.rng)
    }

    pub fn register_responder<T: 'static + Responder>(&mut self, responder: T) {
        self.responders.register_responder(responder)
    }

    fn addressed<'a>(&self, input: &'a str) -> Option<&'a str> {
        if input.starts_with(&self.name) {
            let tail = &input[self.name.len()..];
            if tail.starts_with(": ") {
                return Some(&tail[2..]);
            }
        }

        None
    }
}

pub struct Factoid<'a> {
    pub key: &'a str,
    pub pred: &'a str,
    pub value: &'a str,
}

pub trait KnowsFactoids {
    fn creates_factoid<'a, 'b: 'a>(&'a self, _: &'b str) -> Option<Factoid<'b>>;
    fn create_factoid(&mut self, message: &str) -> Option<String> {
        self.creates_factoid(message)
            .and_then(|factoid| self.learn_factoid(factoid))
    }
    fn learn_factoid(&mut self, _: Factoid) -> Option<String>;
}

// TODO strip whitespass + punctuassion
impl KnowsFactoids for Brain {
    fn learn_factoid(&mut self, factoid: Factoid) -> Option<String> {
        let factoid_resp = responder::FactoidResponder::new(&factoid.key, &factoid.value);
        self.register_responder(factoid_resp);

        Some(format!(
            "Ok, now I know that {} {} {}",
            &factoid.key, &factoid.pred, &factoid.value
        ))
    }

    fn creates_factoid<'a, 'b: 'a>(&'a self, s: &'b str) -> Option<Factoid<'b>> {
        if let Some(s) = self.addressed(s) {
            let predicate_len: usize;
            let mut key_len = 0usize;

            let mut count_space = false;

            for part in s.split_whitespace() {
                if count_space {
                    key_len += 1;
                }

                if part == "is" {
                    predicate_len = 2;
                    return Some(Factoid {
                        // back up by 1 because we don't count the space between the last word of
                        // the key and the predicate
                        key: &s[..key_len - 1],
                        pred: &s[key_len..][..predicate_len],
                        value: &s[key_len + 1 + predicate_len..],
                    });
                } else if part == "are" {
                    predicate_len = 3;
                    return Some(Factoid {
                        // back up by 1 because we don't count the space between the last word of
                        // the key and the predicate
                        key: &s[..key_len - 1],
                        pred: &s[key_len..][..predicate_len],
                        value: &s[key_len + 1 + predicate_len..],
                    });
                } else {
                    key_len += part.len();
                    count_space = true;
                }
            }
        }

        None
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
        brain.set_rng_seed(696969);

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
            brain.creates_factoid("a is b").is_none(),
            "I wasn't addressed, this shouldn't create a factoid"
        );

        assert!(
            brain.creates_factoid("bot_name: a foo b").is_none(),
            "None of my verbs were present, this shouldn't create a factoid"
        );

        assert!(
            brain.creates_factoid("other_name: a is b").is_none(),
            "Someone else was addressed, this shouldn't create a factoid"
        );
    }
}
