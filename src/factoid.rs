use rand::prelude::*;
use rand::prng::XorShiftRng;
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
    fn create_factoid(&mut self, _: String) -> Result<(), Error>;
    fn get_factoid<'a>(&'a mut self, _: &String) -> Option<&'a String>;
    fn literal_factoid(&self, _: &String) -> String;
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
            .position(|pivot| pivot == "is" || pivot == "are")
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
fn is_extension(base: &String, candidate: &String) -> bool {
    if base.len() == 0 && candidate.len() == 0 {
        return true;
    } else if base.len() == 0 && candidate.len() > 0 {
        return false;
    }

    let my_base = base.to_lowercase().to_string();
    let my_candidate = candidate.to_lowercase().to_string();

    let mut bs = my_base.chars().peekable();
    let mut cs = my_candidate.chars().peekable();

    let mut b = bs.next().unwrap(); // If you base in an empty base it's
                                    // your problem >:(
    let mut c = match cs.next() {
        Some(chr) => chr,
        None => return false,
    };

    'outer: loop {
        // first, fast forward bs to the end of its current "run",
        // while making sure cs moves with us
        while let (Some(bx), Some(cx)) = (bs.peek(), cs.peek()) {
            if *bx == b && *cx == c && c == b {
                b = bs.next().unwrap(); // We know it's safe, we peeked
                c = cs.next().unwrap(); // We know it's safe, we peeked
            } else {
                // run is over, break
                break;
            }
        }

        // We've moved b and c to the end of their *shared* run.  Now,
        // keep moving c forward til it finishes that *entire* run, if
        // its run was longer. If c runs out entirely, move b forward
        // one as well. If it had more left, they didn't match. If
        // it's also done, they did.
        while b == c {
            c = match cs.next() {
                Some(chr) => chr,
                None => {
                    b = match bs.next() {
                        Some(_) => return false,
                        None => return true,
                    };

                    break 'outer;
                }};
        }

        // if we're still here, cs had at least 1 element left. bs
        // must have at least 1 element left as well, or the two don't
        // match.
        b = match bs.next() {
            Some(chr) => chr,
            None => return false,
        };

        // if their next chars don't match, this can't work.
        if b != c {
            return false;
        }
    }

    return bs.next() == None;
}

#[test]
pub fn test_is_extension() {
    assert!(is_extension(&"awoo".to_string(), &"awoo".to_string()));
    assert!(is_extension(&"awoo".to_string(), &"awooo".to_string()));
    assert!(is_extension(&"awoo".to_string(), &"aawoo".to_string()));
    assert!(is_extension(&"awoo".to_string(), &"awwoo".to_string()));
    assert!(!is_extension(&"awoo".to_string(), &"awo".to_string()));
    assert!(!is_extension(&"awwo".to_string(), &"awo".to_string()));
    assert!(!is_extension(&"awoo".to_string(), &"ao".to_string()));
    assert!(!is_extension(&"awoo".to_string(), &"aowo".to_string()));
    assert!(!is_extension(&"awoo".to_string(), &"aw0o".to_string()));
}

pub fn is_awoo(s: &String) -> bool {
    is_extension(&"awoo".to_string(), s)
}

pub fn is_meow(s: &String) -> bool {
    is_extension(&"meow".to_string(), s)
        || is_extension(&"miao".to_string(), s)
        || is_extension(&"miaow".to_string(), s)
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
            !creates_factoid(&"bot_name".to_string(), &"bot_name: a foo b".to_string()),
            "None of my verbs were present, this shouldn't create a factoid"
        );
    }
}
