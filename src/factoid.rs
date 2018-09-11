use std::collections::HashMap;
use std::io::Error;

// TODO duplicates

pub type Brain = HashMap<String, String>;

pub trait FactoidKnowledge {
    fn create_factoid(&mut self, &Vec<String>, String) -> Result<(), Error>;
    fn get_factoid<'a>(&'a self, &String) -> Option<&'a String>;
    fn literal_factoid(&self, &String) -> String;
}


// TODO strip whitespass + punctuassion
impl FactoidKnowledge for Brain {
    fn create_factoid(&mut self, verbs: &Vec<String>, s: String) -> Result<(), Error> {
        // Drop name:
        let name_index = s.find(":").unwrap();
        let cleaned_string = s.clone().split_off(name_index+1);
        
        let iter = cleaned_string.split_whitespace();
        let index = iter
            .clone()
            .position(|pivot| verbs.contains(&pivot.to_string()))
            .unwrap();

        let tmp: Vec<&str> = iter.collect();
        let (k, v) = tmp.split_at(index);

        self.insert(k.join(" ").to_owned(), v[1..].join(" ").to_owned());
        Ok(())
    }

    fn get_factoid<'a>(&'a self, k: &String) -> Option<&'a String> {
        self.get(k)
    }

    // TODO one can not know things in many ways!
    fn literal_factoid(&self, k: &String) -> String {
        match self.get(k) {
            Some(v) => v.clone(),
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
        let mut brain = Brain::new();
        let verbs = vec!["is".to_owned()];
        brain.create_factoid(&verbs, "sidra: foo is bar".to_string());
        assert_eq!(brain.get("foo").unwrap(), "bar");
    }

    #[test]
    fn can_retrieve_factoid() {
        let mut brain = Brain::new();
        brain.insert("foo".to_string(), "bar".to_string());
        assert_eq!(
            "bar".to_string(),
            *brain.get_factoid(&"foo".to_string()).unwrap()
        );
    }

    #[test]
    fn no_nonfactoid_retrieval() {
        let mut brain = Brain::new();
        brain.insert("foo".to_string(), "bar".to_string());
        assert!(brain.get_factoid(&"bar".to_string()).is_none());
    }

    #[test]
    fn can_literal_factoid() {
        let mut brain = Brain::new();
        brain.insert("foo".to_string(), "bar".to_string());
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
