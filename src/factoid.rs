use std::collections::HashMap;

// TODO duplicates
type Brain = HashMap<String, String>;

fn create_factoid(mut brain: Brain, k: String, v: String) -> Brain {
    brain.insert(k, v);
    brain
}

fn get_factoid<'a>(brain: &'a Brain, k: &String) -> Option<&'a String> {
    brain.get(k)
}

// TODO one can not know things in many ways!
fn literal_factoid(brain: &Brain, k: &String) -> String {
    match brain.get(k) {
        Some(v) => v.clone(),
        None => "I don't know anything about that".to_string(),
    }
}

// TODO needs to split on whitespass + punctuassion
fn creates_factoid(name: &String, verbs: &Vec<String>, s: &String) -> bool {
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
        brain = create_factoid(brain, "foo".to_string(), "bar".to_string());
        assert_eq!(brain.get("foo").unwrap(), "bar");
    }

    #[test]
    fn can_retrieve_factoid() {
        let mut brain = Brain::new();
        brain.insert("foo".to_string(), "bar".to_string());
        assert_eq!(
            "bar".to_string(),
            *get_factoid(&brain, &"foo".to_string()).unwrap()
        );
    }

    #[test]
    fn no_nonfactoid_retrieval() {
        let mut brain = Brain::new();
        brain.insert("foo".to_string(), "bar".to_string());
        assert!(get_factoid(&brain, &"bar".to_string()).is_none());
    }

    #[test]
    fn can_literal_factoid() {
        let mut brain = Brain::new();
        brain.insert("foo".to_string(), "bar".to_string());
        assert_eq!(
            "bar".to_string(),
            literal_factoid(&brain, &"foo".to_string())
        );

        assert_eq!(
            "I don't know anything about that".to_string(),
            literal_factoid(&brain, &"zip".to_string())
        );
    }

    #[test]
    fn ids_factoid_creation() {
        let mut verbs = Vec::new();
        verbs.push("foo".to_string());
        assert!(creates_factoid(
            &"bot_name".to_string(),
            &verbs,
            &"bot_name: a foo b".to_string()
        ));
    }

    #[test]
    fn ids_non_factoid_creation() {
        let mut verbs = Vec::new();
        verbs.push("foo".to_string());

        assert!(
            !creates_factoid(&"bot_name".to_string(), &verbs, &"a foo b".to_string()),
            "I wasn't addressed, this shouldn't create a factoid"
        );

        assert!(
            !creates_factoid(
                &"bot_name".to_string(),
                &verbs,
                &"bot_name: a forb b".to_string()
            ),
            "None of my verbs were present, this shouldn't create a factoid"
        );
    }
}
