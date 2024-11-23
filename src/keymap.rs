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

    pub fn category_map(&self) -> &HashMap<String, Vec<Action>> {
        &self.actions
    }

    pub fn actions(&self) -> impl Iterator<Item=&Action> {
        self.actions.values().flat_map(|v| v.iter())
    }

    pub fn categories(&self) -> Vec<(String, usize)> {
        self.actions.iter()
            .map(|(category, actions)| (category.clone(), actions.len()))
            .collect()
    }

    // pub fn categories(&self) -> Vec<(String, usize)> {
    //     let mut categories: HashMap<String, usize> = HashMap::new();
    //     self.actions.values()
    //         .flatten()
    //         .filter(|a| a.is_bound())
    //         .map(|a| a.category.to_string())
    //         .for_each(|category| {
    //             let count = categories.entry(category).or_insert(0);
    //             *count += 1;
    //         });
    //
    //     let mut cats: Vec<_> = categories.into_iter().collect();
    //     cats.sort_by(|(c1, _), (c2, _)| c1.cmp(c2));
    //     cats
    // }

    // pub fn valid_actions(&self) -> impl Iterator<Item=(&'static str, &Action)> {
    //     // req re-impl since update to KeyMap::actions
    //     self.actions.values()
    //         .flatten()
    //         .filter(|a| a.is_bound())
    //         .map(|a| (crate::parser::jetbrains::categorize_action(a), a))
    // }
    //
    //
    //
    // pub fn categories(&self) -> Vec<(String, usize)> {
    //     let mut categories: HashMap<String, usize> = HashMap::new();
    //     self.actions.values()
    //         .flatten()
    //         .filter(|a| a.is_bound())
    //         // .map(categorize_action)
    //         .for_each(|category| {
    //             let count = categories.entry(category.to_string()).or_insert(0);
    //             *count += 1;
    //         });
    //
    //     let mut cats: Vec<_> = categories.into_iter().collect();
    //     cats.sort_by(|(c1, _), (c2, _)| c1.cmp(c2));
    //     cats
    //
    // }
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