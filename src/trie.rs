use std::{collections::HashMap, fs};

#[derive(Debug)]
pub struct Trie {
    pub child: HashMap<char, Trie>,
    is_leaf: bool,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            child: HashMap::new(),
            is_leaf: false,
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut trie = self;
        for letter in word.chars() {
            trie = trie.child.entry(letter).or_insert(Trie::new());
        }
        trie.is_leaf = true;
    }

    pub fn get_words_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut trie = self;
        let mut results = Vec::new();
        for letter in prefix.chars() {
            if let Some(child) = trie.child.get(&letter) {
                trie = child;
            } else {
                return results;
            }
        }
        dfs(trie, prefix.to_owned(), &mut results);
        results
    }
}

fn dfs(trie: &Trie, word: String, results: &mut Vec<String>) {
    if trie.is_leaf {
        results.push(word.clone());
    }

    for (letter, child) in &trie.child {
        let mut new_word = word.clone();
        new_word.push(*letter);
        dfs(child, new_word, results);
    }
}

pub fn initialize_trie(trie: &mut Trie) {
    // could be added to a sqlite db
    for path in std::env::var("PATH").unwrap_or_default().split(':') {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(Result::ok) {
                if let Some(command_file) = entry.file_name().to_str() {
                    trie.insert(command_file);
                }
            }
        }
    }
    trie.insert("exit");
}
