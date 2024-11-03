
use std::collections::{BTreeMap, VecDeque};
use std::ptr;



pub struct TrieTree<'a> {
    root_ptr: *mut TrieNode<'a>,
    pub keys: &'a Vec<String>,
}


struct TrieNode<'a> {
    elem: char,
    word: &'a str,
    next_ptr: *mut TrieNode<'a>,
    child_ptr: *mut TrieNode<'a>,
    failed_ptr: *mut TrieNode<'a>,
}


impl<'a> TrieTree<'a> {
    pub fn new(keys: &'a Vec<String>) -> Self {
        let root_ptr = TrieTree::build_ac_automaton(keys);

        TrieTree { root_ptr, keys }
    }

    fn build_ac_automaton(target_strings: &Vec<String>) -> *mut TrieNode {
        let root_ptr = TrieNode::new('_');

        for target in target_strings.iter() {
            let mut node = unsafe { &mut *root_ptr };
            assert_ne!(target.len(), 0);

            for c in target.chars() {
                let matched_child_ptr = node.try_find_child(c);

                if matched_child_ptr.is_null() {
                    node.child_ptr = TrieNode::new(c);
                    node = unsafe { &mut *node.child_ptr };
                    continue;
                }

                let matched_child = unsafe { &mut *matched_child_ptr };
                if matched_child.elem == c {
                    node = matched_child;
                } else {
                    matched_child.next_ptr = TrieNode::new(c);
                    node = unsafe { &mut *matched_child.next_ptr };
                }
            }

            node.word = target;
        }

        let mut queue: VecDeque<&TrieNode> = VecDeque::new();
        let root = unsafe { &*root_ptr };
        queue.push_back(root);

        while !queue.is_empty() {
            if let Some(node) = queue.pop_front() {
                let child_ptr = node.child_ptr;

                if !child_ptr.is_null() {
                    let mut children_sibling_ptr = child_ptr;

                    while !children_sibling_ptr.is_null() {
                        let children_sibling = unsafe { &mut *children_sibling_ptr };
                        let mut failed_ptr = node.failed_ptr;

                        loop {
                            if failed_ptr.is_null() {
                                children_sibling.failed_ptr = root_ptr;
                                break;
                            }

                            let failed_node = unsafe { &mut *failed_ptr };

                            if let Some(matched_ptr) = failed_node.find_child(children_sibling.elem)
                            {
                                children_sibling.failed_ptr = matched_ptr;
                                break;
                            }

                            failed_ptr = failed_node.failed_ptr;
                        }

                        queue.push_back(children_sibling);
                        children_sibling_ptr = children_sibling.next_ptr;
                    }
                }
            }
        }

        //bfs_trie_tree(&tree);

        root_ptr
    }

    pub fn index_of(&self, text: &str) -> BTreeMap<String, Vec<usize>> {
        let mut result: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for s in self.keys.iter() {
            result.insert(s.to_string(), vec![]);
        }

        let root_ptr = self.root_ptr;

        let mut state_node = unsafe { &mut *root_ptr };

        for (i, c) in text.char_indices() {
            loop {
                if let Some(matched_child_ptr) = state_node.find_child(c) {
                    let matched_child = unsafe { &mut *matched_child_ptr };

                    // solve subword scenario like `she` and `he`
                    let mut match_node = &mut *matched_child;
                    while !match_node.failed_ptr.is_null() {
                        if match_node.word.len() > 0 {
                            if let Some(list) = result.get_mut(match_node.word) {
                                list.push(i + c.to_string().len() - match_node.word.len());
                            }
                        }

                        match_node = unsafe { &mut *match_node.failed_ptr };
                    }

                    state_node = matched_child;
                    break;
                } else if state_node.failed_ptr.is_null() {
                    state_node = unsafe { &mut *root_ptr };
                    break;
                }

                state_node = unsafe { &mut *state_node.failed_ptr };
            }
        }
        result
    }
}


impl<'a> TrieNode<'a> {
    fn new(elem: char) -> *mut TrieNode<'a> {
        Box::into_raw(Box::new(TrieNode {
            elem,
            word: "",
            next_ptr: ptr::null_mut(),
            child_ptr: ptr::null_mut(),
            failed_ptr: ptr::null_mut(),
        }))
    }

    fn try_find_child(&self, elem: char) -> *mut TrieNode<'a> {
        if self.child_ptr.is_null() {
            return self.child_ptr;
        }

        let mut sibling_ptr = self.child_ptr;
        let mut sibling = unsafe { &*sibling_ptr };

        while !sibling.next_ptr.is_null() {
            if sibling.elem == elem {
                return sibling_ptr;
            }

            sibling_ptr = sibling.next_ptr;
            sibling = unsafe { &*sibling_ptr };
        }
        sibling_ptr
    }

    fn find_child(&self, elem: char) -> Option<*mut TrieNode<'a>> {
        let result_ptr = self.try_find_child(elem);

        if result_ptr.is_null() {
            return None;
        }

        let result = unsafe { &*result_ptr };
        if result.elem == elem {
            Some(result_ptr)
        } else {
            None
        }
    }

    #[allow(unused)]
    #[cfg(test)]
    fn mark_position(&self) -> String {
        let mut tail = String::new();
        let mut whole_word = &self.word;
        let mut mark_str = String::new();

        let mut node_ptr = self.child_ptr;

        while !node_ptr.is_null() {
            let node = unsafe { &mut *node_ptr };
            tail.push(node.elem);

            node_ptr = node.child_ptr;
            whole_word = &node.word;
        }

        if let Some(target_index) = whole_word.find(tail.as_str()) {
            if tail.len() > 0 && target_index == 0 {
                mark_str.push_str("[]");
            }

            for (i, c) in whole_word.char_indices() {
                let c_len = c.to_string().len();
                if target_index > 0 && i == target_index - c_len
                    || tail.len() == 0 && i + c_len == whole_word.len()
                {
                    mark_str.push_str(format!("[{}]", c).as_str());
                } else {
                    mark_str.push(c);
                }
            }

            mark_str
        } else {
            String::from("???")
        }
    }
}


#[allow(unused)]
#[cfg(test)]
fn bfs_trie_tree(tree: &TrieTree) {
    let mut queue: VecDeque<&TrieNode> = VecDeque::new();
    let root = unsafe { &mut *tree.root_ptr };
    queue.push_back(root);

    println!("**********");
    while !queue.is_empty() {
        if let Some(node) = queue.pop_front() {
            let child_ptr = node.child_ptr;

            if !child_ptr.is_null() {
                let mut sibling_ptr = child_ptr;

                while !sibling_ptr.is_null() {
                    let sibling = unsafe { &*sibling_ptr };

                    if !sibling.failed_ptr.is_null() {
                        let failed_to = unsafe { &*(*sibling).failed_ptr };
                        print!(
                            "{} faileTo: {}: ",
                            sibling.mark_position(),
                            failed_to.mark_position()
                        );
                    }
                    if sibling.word.len() > 0 {
                        print!(" {}", sibling.word)
                    }
                    println!("");
                    queue.push_back(sibling);
                    sibling_ptr = sibling.next_ptr;
                }
                println!("---");
            }
        }
    }
    print!("")
}



#[cfg(test)]
mod tests {
    extern crate test;

    use common::btreemap;

    use super::{ *, super::* };

    #[test]
    fn test_ac_automaton_fixeddata() {
        let mut result = TrieTree::new(&vec![
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

        ])
        .index_of("bcabcdebcedfabcdefababkabhabk");

        assert_eq!(
            result,
            btreemap! {
                "bcd".to_string()=> vec![3, 13],
                "cdfkcdf".to_string()=> vec![],
                "cde".to_string()=> vec![4, 14],
                "abcdef".to_string()=> vec![12],
                "abhab".to_string()=> vec![23],

                "ebc".to_string()=> vec![6, ],
                "fab".to_string()=> vec![11, 17],
                "zzzz".to_string()=> vec![],
                "debca".to_string()=> vec![],
                "debce".to_string()=> vec![5],
                "debcd".to_string()=> vec![],
                "defab".to_string()=> vec![15],
                "abcdefg".to_string()=> vec![],
                "habk".to_string()=> vec![25],
                "bkka".to_string()=> vec![],
            }
        );

        result = TrieTree::new(&vec![
            "she".to_string(),
            "shr".to_string(),
            "say".to_string(),
            "he".to_string(),
            "her".to_string(),
        ])
        .index_of("one day she say her has eaten many shrimps");

        assert_eq!(
            result,
            btreemap! {
                "she".to_string()=> vec![8],
                "he".to_string()=> vec![9, 16],
                "say".to_string()=> vec![12],
                "her".to_string()=> vec![16],
                "shr".to_string()=> vec![35]
            }
        );

        result = TrieTree::new(&vec![
            "你好".to_string(),
            "你好棒".to_string(),
            "今天".to_string(),
        ])
        .index_of("你好，今天你看起来好棒");
        assert_eq!(
            result,
            btreemap! {
                "你好".to_string()=> vec![0],
                "今天".to_string()=> vec![9],
                "你好棒".to_string()=> vec![],
            }
        );
    }

    #[test]
    fn test_ac_automaton_randomdata() {
        for (pats, text, res) in gen_test_case_multiple() {
            let tree = TrieTree::new(&pats);

            assert_eq!(tree.index_of(text.as_str()), res);
        }
    }


}
