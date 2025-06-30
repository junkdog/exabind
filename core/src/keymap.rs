use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;
use std::path::PathBuf;
use crate::shortcut::Action;

#[derive(Debug, Clone)]
pub struct KeyMap {
    name: String,
    actions: HashMap<String, Vec<Action>>,
}



static EMPTY_ACTIONS: Vec<Action> = Vec::new();

impl KeyMap {

    pub fn new<S: ToString>(name: S, actions: HashMap<String, Vec<Action>>) -> Self {
        Self { name: name.to_string(), actions }
    }

    pub fn actions_by_category(&self, category: &str) -> &[Action] {
        self.actions.get(category)
            .unwrap_or(&EMPTY_ACTIONS)
    }

    pub fn actions(&self) -> impl Iterator<Item=&Action> {
        self.actions.values().flat_map(|v| v.iter())
    }

    pub fn categories(&self) -> Vec<(String, usize)> {
        self.actions.iter()
            .map(|(category, actions)| (category.clone(), actions.len()))
            .collect()
    }
}

impl Display for KeyMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format = |action: &Action| format!("\t{}", action);
        let actions = self.actions().map(format).collect::<Vec<_>>().join("\n");
        write!(f, "keymap name={}:\n{}", self.name, actions)
    }
}


pub trait IntoKeyMap {
    fn into_keymap<F: FnOnce(&str) -> KeyMap>(self, f: F) -> KeyMap;
}

impl IntoKeyMap for &str {
    fn into_keymap<F: FnOnce(&str) -> KeyMap>(self, f: F) -> KeyMap {
        f(self)
    }
}

impl IntoKeyMap for PathBuf {
    fn into_keymap<F: FnOnce(&str) -> KeyMap>(self, f: F) -> KeyMap {
        let mut input = String::new();
        let mut file = std::fs::File::open(self)
            .expect("file to be present");
        file.read_to_string(&mut input)
            .expect("parsable xml");

        f(&input)
    }
}