use super::util;
use std::boxed::Box;
use util::is_extension;

pub struct Responders {
    responders: Vec<Box<dyn Responder>>,
}

impl Responders {
    pub fn new() -> Responders {
        Responders {
            responders: Vec::<Box<dyn Responder>>::new(),
        }
    }

    pub fn register_responder<T: 'static + Responder>(&mut self, responder: T) {
        self.responders.push(Box::new(responder));
    }

    pub fn respond(&self, input: &str) -> Vec<String> {
        let mut outvec = Vec::new();
        for resp in &self.responders {
            if let Some(out) = resp.respond(input) {
                outvec.push(out);
            }
        }

        outvec
    }

    // Add some silly nonsense
    pub fn default() -> Responders {
        let mut resps = Responders::new();
        resps.register_responder(SimpleResponder { base: "awoo" });
        resps.register_responder(SimpleResponder { base: "meow" });
        resps.register_responder(SimpleResponder { base: "miao" });
        resps.register_responder(SimpleResponder { base: "mrao" });
        resps.register_responder(SimpleResponder { base: "mraow" });
        resps.register_responder(GoblinResponder {});
        resps
    }
}

#[test]
pub fn test_default_responders() {
    let resp = Responders::default();
    assert_eq!(resp.respond(&"awoo"), vec!("awoo"));
    assert_eq!(
        resp.respond(&"look, a goblin!"),
        vec!("MEOW!")
    );
}

pub trait Responder {
    fn respond(&self, _: &str) -> Option<String>;
}

pub struct FactoidResponder {
    key: String,
    val: String,
}
impl FactoidResponder {
    pub fn new(key: &str, val: &str) -> FactoidResponder {
        FactoidResponder { key: key.to_owned(), val: val.to_owned() }
    }
}

impl Responder for FactoidResponder {
    fn respond(&self, input: &str) -> Option<String> {
        if input == self.key.as_str() {
            Some(self.val.clone())
        } else {
            None
        }
    }
}

pub struct SimpleResponder {
    base: &'static str,
}
impl Responder for SimpleResponder {
    fn respond(&self, input: &str) -> Option<String> {
        if is_extension(self.base, input) {
            Some(self.base.to_owned())
        } else {
            None
        }
    }
}

pub struct GoblinResponder {}
impl Responder for GoblinResponder {
    fn respond(&self, input: &str) -> Option<String> {
        if input.contains("goblin") {
            Some("MEOW!".to_string())
        } else {
            None
        }
    }
}
