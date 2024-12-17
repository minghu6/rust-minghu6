//! Aho–Corasick algorithm

use std::collections::BTreeMap;

use m6ptr::{OwnedPtr, Ptr};

////////////////////////////////////////////////////////////////////////////////
//// Structures

/// KMP + Trie
pub struct ACTrie {
    root: Ptr<Node>,
    keys: Vec<OwnedPtr<str>>,
    _nodes: Vec<OwnedPtr<Node>>,
}

struct Node {
    elem: char,
    word: Option<Ptr<str>>,
    children: Vec<Ptr<Self>>,
    failed: Option<Ptr<Self>>,
}


////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl Node {
    fn new_with_elem(elem: char) -> Self {
        Self {
            elem,
            ..Default::default()
        }
    }

    fn find_child(&self, elem: char) -> Option<Ptr<Self>> {
        self.children
            .iter()
            .find(|child| child.elem == elem)
            .copied()
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            elem: '_',
            word: None,
            children: vec![],
            failed: None,
        }
    }
}

impl ACTrie {
    pub fn with_keys(keys: Vec<String>) -> Self {
        let mut nodes = vec![];
        let keys = keys
            .into_iter()
            .map(|s| OwnedPtr::from_box(s.into_boxed_str()))
            .collect::<Vec<_>>();

        let root = Self::create_new_node(&mut nodes, '_');

        for word in keys.iter() {
            debug_assert!(!word.is_empty());

            let mut ptr = root;

            for c in word.chars() {
                if let Some(child) = ptr.find_child(c) {
                    ptr = child;
                }
                else {
                    let new_child = Self::create_new_node(&mut nodes, c);
                    ptr.children.push(new_child);
                    ptr = new_child;
                }
            }

            ptr.word = Some(word.ptr());
        }

        /* setup failed ptr */

        for ptr in Self::pre_order_walk(root) {
            for mut child in ptr.children.iter().cloned() {
                let mut maybe_failed = ptr.failed;

                while let Some(failed) = maybe_failed {
                    if let Some(matched) = failed.find_child(child.elem) {
                        child.failed = Some(matched);
                        break;
                    }

                    maybe_failed = failed.failed;
                }

                if child.failed.is_none() {
                    child.failed = Some(root);
                }
            }
        }

        Self {
            root,
            keys,
            _nodes: nodes,
        }
    }

    pub fn search(&self, string: &str) -> BTreeMap<&str, Vec<usize>> {
        let mut res = self
            .keys
            .iter()
            .map(|k| (k.as_str(), vec![]))
            .collect::<BTreeMap<_, _>>();

        let mut ptr = self.root;

        for (i, c) in string.char_indices() {
            loop {
                if let Some(child) = ptr.find_child(c) {
                    let mut subptr = child;

                    while !Ptr::ptr_eq(&subptr, &self.root) {
                        if let Some(word) = subptr.word {
                            let list = res.get_mut(word.as_str()).unwrap();
                            list.push(i + c.len_utf8() - word.len())
                        }

                        subptr = subptr.failed.unwrap();
                    }

                    ptr = child;
                    break;
                }

                if let Some(failed) = ptr.failed {
                    ptr = failed;
                }
                else {
                    debug_assert!(Ptr::ptr_eq(&ptr, &self.root));
                    break;
                }
            }
        }

        res
    }

    fn create_new_node(
        nodes: &mut Vec<OwnedPtr<Node>>,
        elem: char,
    ) -> Ptr<Node> {
        let node_owned = OwnedPtr::new(Node::new_with_elem(elem));
        let node = node_owned.ptr();

        nodes.push(node_owned);
        node
    }

    fn pre_order_walk(root: Ptr<Node>) -> impl Iterator<Item = Ptr<Node>> {
        std::iter::from_coroutine(
            #[coroutine]
            move || {
                let mut curlv = vec![vec![root]];

                while !curlv.is_empty() {
                    let mut nxtlv = vec![];

                    for child_group in curlv.into_iter() {
                        for child in child_group {
                            yield child;

                            if !child.children.is_empty() {
                                nxtlv.push(child.children.clone());
                            }
                        }
                    }

                    curlv = nxtlv;
                }
            },
        )
    }
}


#[cfg(test)]
mod tests {
    extern crate test;

    use common::btreemap;

    use super::{super::*, *};

    #[test]
    fn test_ac_automaton2_fixeddata() {
        let mut trie = ACTrie::with_keys(vec![
            "bcd".to_string(),
            "cdfkcdf".to_string(),
            "cde".to_string(),
            "abcdef".to_string(),
            "abhab".to_string(),
            "ebc".to_string(),
            "fab".to_string(),
            "zzzz".to_string(),
            "debca".to_string(),
            "debce".to_string(),
            "debcd".to_string(),
            "defab".to_string(),
            "abcdefg".to_string(),
            "habk".to_string(),
            "bkka".to_string(),
        ]);

        let mut result = trie.search("bcabcdebcedfabcdefababkabhabk");

        assert_eq!(
            result,
            btreemap! {
                "bcd"=> vec![3, 13],
                "cdfkcdf"=> vec![],
                "cde"=> vec![4, 14],
                "abcdef"=> vec![12],
                "abhab"=> vec![23],
                "ebc"=> vec![6, ],
                "fab"=> vec![11, 17],
                "zzzz"=> vec![],
                "debca"=> vec![],
                "debce"=> vec![5],
                "debcd"=> vec![],
                "defab"=> vec![15],
                "abcdefg"=> vec![],
                "habk"=> vec![25],
                "bkka"=> vec![],
            }
        );

        trie = ACTrie::with_keys(vec![
            "she".to_string(),
            "shr".to_string(),
            "say".to_string(),
            "he".to_string(),
            "her".to_string(),
        ]);

        result = trie.search("one day she say her has eaten many shrimps");

        assert_eq!(
            result,
            btreemap! {
                "she"=> vec![8],
                "he"=> vec![9, 16],
                "say"=> vec![12],
                "her"=> vec![16],
                "shr"=> vec![35]
            }
        );

        trie = ACTrie::with_keys(vec![
            "你好".to_string(),
            "你好棒".to_string(),
            "今天".to_string(),
        ]);

        result = trie.search("你好，今天你看起来好棒");
        assert_eq!(
            result,
            btreemap! {
                "你好" => vec![0],
                "今天"=> vec![9],
                "你好棒" => vec![],
            }
        );
    }

    #[test]
    fn test_ac_automaton2_randomdata() {
        for (pats, string, expect) in gen_test_case_multiple() {
            let tree = ACTrie::with_keys(pats);
            let res = tree.search(string.as_str());
            let res = res
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect::<BTreeMap<_, _>>();

            assert_eq!(res, expect);
        }
    }
}
