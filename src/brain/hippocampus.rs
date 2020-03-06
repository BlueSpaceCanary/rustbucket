use crate::models::{Factoid, NewFactoid};
use std::option::Option;

pub trait Hippocampus {
    fn new(path: &'static str) -> Self;
    fn recall(&self, key: &str) -> Option<Factoid>;
    fn learn(&self, factoid: &NewFactoid) -> Result<(), &'static str>;
}

pub struct IdMemory {}

impl Hippocampus for IdMemory {
    fn new(path: &'static str) -> Self {
        IdMemory {}
    }

    fn recall(&self, key: &str) -> Option<Factoid> {
        None
    }

    fn learn(&self, factoid: &NewFactoid) -> Result<(), &'static str> {
        Ok(())
    }
}
