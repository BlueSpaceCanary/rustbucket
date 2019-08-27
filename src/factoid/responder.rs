use super::util;
use super::Factoid;
use std::boxed::Box;
use std::cmp::Ordering;
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

    pub fn respond<'a, 'b: 'a>(&'a self, input: &'b str) -> impl Iterator<Item = String> + 'a {
        let mut resps = self
            .responders
            .iter()
            .filter_map(move |resp| resp.respond(input))
            .collect::<Vec<Response>>();
        resps.sort_unstable_by(|x, y| x.priority.cmp(&y.priority));

        // if no resps, pick a nice alternative since it's empty anyway
        let min = resps
            .get(0)
            .and_then(|x| Some(x.priority))
            .or_else(|| Some(69))
            .unwrap();
        resps
            .into_iter()
            .filter(|x| x.priority == min)
            .map(|x| x.resp)
            .collect::<Vec<String>>()
            .into_iter()
    }

    // Add some silly nonsense
    pub fn default() -> Responders {
        let mut resps = Responders::new();
        resps.register_responder(SimpleResponder { base: "awoo" });
        resps.register_responder(SimpleResponder { base: "meow" });
        resps.register_responder(SimpleResponder { base: "miao" });
        resps.register_responder(SimpleResponder { base: "mrao" });
        resps.register_responder(SimpleResponder { base: "mraow" });
        resps.register_responder(SimpleResponder { base: "miau" });
        resps.register_responder(GoblinResponder {});
        resps
    }
}

#[test]
pub fn test_default_responders() {
    let resp = Responders::default();
    assert_eq!(resp.respond(&"awoo").collect::<Vec<String>>(), vec!("awoo"));
    assert_eq!(resp.respond(&"Miao").collect::<Vec<String>>(), vec!("Miao"));
    assert_eq!(
        resp.respond(&"look, a goblin!").collect::<Vec<String>>(),
        vec!("MEOW!")
    );
}

#[test]
pub fn test_responder_priority() {
    let mut resp = Responders::default();
    resp.register_responder(FactoidResponder::new(Factoid {
        key: "awoo".to_string(),
        value: "dropped".to_string(),
        pred: "is".to_string(),
    }));

    // Should have dropped everything but the factoid due to priority difference
    assert_eq!(
        resp.respond(&"awoo").collect::<Vec<String>>(),
        vec!("dropped")
    );
}

pub trait Responder {
    fn respond(&self, _: &str) -> Option<Response>;
}

#[derive(Debug, PartialEq)]
pub struct Response {
    priority: u32,
    resp: String,
}

impl Response {
    pub fn new(resp: String, priority: u32) -> Response {
        Response { resp, priority }
    }
}

pub struct FactoidResponder {
    factoid: Factoid,
    priority: u32,
}
impl FactoidResponder {
    pub fn new(factoid: Factoid) -> FactoidResponder {
        // Factoids are sort of important I guess? Not that important though
        FactoidResponder {
            factoid,
            priority: 1000,
        }
    }
}

impl Responder for FactoidResponder {
    fn respond(&self, input: &str) -> Option<Response> {
        if input == self.factoid.key {
            Some(Response::new(self.factoid.value.to_string(), self.priority))
        } else {
            None
        }
    }
}

#[test]
fn test_factoids_respond() {
    let factoid = Factoid {
        key: "spinch the robot".to_string(),
        pred: "is".to_string(),
        value: "beautiful".to_string(),
    };
    let resper = FactoidResponder::new(factoid);
    assert_eq!(
        Some(Response::new("beautiful".to_string(), 1000)),
        resper.respond(&"spinch the robot")
    );
}

pub struct SimpleResponder {
    base: &'static str,
}
impl Responder for SimpleResponder {
    fn respond(&self, input: &str) -> Option<Response> {
        if is_extension(self.base, input) {
            Some(Response::new(input.to_owned(), std::u32::MAX))
        } else {
            None
        }
    }
}

pub struct GoblinResponder {}
impl Responder for GoblinResponder {
    fn respond(&self, input: &str) -> Option<Response> {
        if input.contains("goblin") {
            Some(Response::new("MEOW!".to_string(), std::u32::MAX))
        } else {
            None
        }
    }
}
