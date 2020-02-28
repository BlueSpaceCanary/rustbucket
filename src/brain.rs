extern crate rand;
extern crate rand_core;
extern crate rand_xorshift;

mod hippocampus;
mod responder;
mod util;

use super::models::Factoid;
use super::models::NewFactoid;
use hippocampus::Hippocampus;
use rand::prelude::IteratorRandom;
use rand_core::SeedableRng;
use rand_xorshift::XorShiftRng;
use responder::Responder;
use responder::Responders;

// Wraps the Brain, handles the stateful external world
pub struct Superego<T: Hippocampus = hippocampus::IdMemory> {
    ego: Brain,
    mem: T,
}

impl<T: Hippocampus> Superego<T> {
    pub fn new(name: String) -> Self {
        Superego {
            ego: Brain::new(name),
            mem: T::new(&"asdf"),
        }
    }

    pub(crate) fn respond(&mut self, input: &str) -> Option<String> {
        match self.create_factoid(input) {
            Ok(Some(x)) => Some(x),
            Ok(None) => self.ego.respond(input),
            Err(x) => Some(x.to_string()),
        }
    }
}

struct Brain {
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
        // Brain's KnowsFactoid won't error.
        match self.create_factoid(input) {
            Ok(Some(x)) => Some(x),
            Ok(None) => self.responders.respond(input).choose(&mut self.rng),
            Err(x) => Some(x.to_string()),
        }
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

pub trait KnowsFactoids {
    fn creates_factoid(&self, _: &str) -> Option<NewFactoid>;
    fn create_factoid(&mut self, message: &str) -> Result<Option<String>, &'static str> {
        match self.creates_factoid(message) {
            Some(x) => self.learn_factoid(x),
            None => Ok(None),
        }
    }
    fn learn_factoid(&mut self, _: NewFactoid) -> Result<Option<String>, &'static str>;
}

impl<T: hippocampus::Hippocampus> KnowsFactoids for Superego<T> {
    fn creates_factoid(&self, s: &str) -> Option<NewFactoid> {
        self.ego.creates_factoid(s)
    }

    fn learn_factoid(&mut self, factoid: NewFactoid) -> Result<Option<String>, &'static str> {
        //TODO write to db
        self.mem.learn(&factoid)?;
        self.ego.learn_factoid(factoid)
    }
}

// TODO strip whitespass + punctuassion
impl KnowsFactoids for Brain {
    fn learn_factoid(&mut self, factoid: NewFactoid) -> Result<Option<String>, &'static str> {
        let out = format!(
            "Ok, now I know that {} {} {}",
            factoid.key, factoid.pred, factoid.value
        );

        let factoid = Factoid {
            id: 0,
            key: factoid.key,
            pred: factoid.pred,
            value: factoid.value,
        };

        let factoid_resp = responder::FactoidResponder::new(factoid);
        self.register_responder(factoid_resp);
        Ok(Some(out))
    }

    fn creates_factoid(&self, s: &str) -> Option<NewFactoid> {
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
                    return Some(NewFactoid {
                        // back up by 1 because we don't count the
                        // space between the last word of the key and
                        // the predicate
                        key: s[..key_len - 1].to_owned(),
                        pred: s[key_len..][..predicate_len].to_owned(),
                        value: s[key_len + 1 + predicate_len..].to_owned(),
                    });
                } else if part == "are" {
                    predicate_len = 3;
                    return Some(NewFactoid {
                        // back up by 1 because we don't count the space between the last word of
                        // the key and the predicate
                        key: s[..key_len - 1].to_owned(),
                        pred: s[key_len..][..predicate_len].to_owned(),
                        value: s[key_len + 1 + predicate_len..].to_owned(),
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
        assert!(!brain.create_factoid(&"sidra: foo is bar").is_err());
        assert_eq!(brain.respond("foo"), Some("bar".to_string()));
    }

    #[test]
    fn can_set_and_retrieve_multi_factoid() {
        let mut brain = Brain::new("sidra".to_owned());

        // Set arbitrarily to make the test work
        brain.set_rng_seed(0);

        assert!(!brain
            .create_factoid(&"sidra: foo is bar".to_string())
            .is_err());
        assert!(!brain
            .create_factoid(&"sidra: foo is zip".to_string())
            .is_err());

        assert_eq!(brain.respond("foo"), Some("bar".to_string()));

        // Set arbitrarily to make the test work
        brain.set_rng_seed(6969);

        assert_eq!(brain.respond("foo"), Some("zip".to_string()));
    }

    #[test]
    fn no_nonfactoid_retrieval() {
        let mut brain = Brain::new("sidra".to_owned());
        assert!(!brain
            .create_factoid(&"sidra: foo isn't bar".to_string())
            .is_err());
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
