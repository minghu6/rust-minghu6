#![allow(dead_code)]

use std::ptr;
use std::collections::{ VecDeque, BTreeMap };

struct TrieNode<'a> {
    elem: char,
    word: &'a str,
    next_ptr: *mut TrieNode<'a>,
    child_ptr: *mut TrieNode<'a>,
    failed_ptr: *mut TrieNode<'a>,
}

impl <'a>TrieNode<'a> {
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
                return sibling_ptr
            }

            sibling_ptr = sibling.next_ptr;
            sibling = unsafe { &*sibling_ptr };
        }
        sibling_ptr
    }

    fn find_child(&self, elem: char) -> Option<*mut TrieNode<'a>> {
        let result_ptr = self.try_find_child(elem);

        if result_ptr.is_null() {
            return None
        } 
        
        let result = unsafe { &*result_ptr };
        if result.elem == elem {
            Some(result_ptr)
        } else {
            None
        }
    }

    fn mark_position(&self) -> String {
        let mut tail = String::new();
        let mut whole_word = &self.word; 
        let mut mark_str = String::new();

        let mut node_ptr = self.child_ptr;

        while !node_ptr.is_null() {
            let node = unsafe {&mut *node_ptr };
            tail.push(node.elem);

            node_ptr = node.child_ptr;
            whole_word = &node.word;
        }

        if let Some(target_index) = whole_word.find(tail.as_str()) {
            if tail.len()>0 && target_index == 0 {
                mark_str.push_str("[]");
            }

            for (i, c ) in whole_word.char_indices() {
                let c_len = c.to_string().len();
                if target_index > 0 && i == target_index-c_len || tail.len() == 0 && i+c_len == whole_word.len() {
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

pub struct TrieTree<'a> {
    root_ptr: *mut TrieNode<'a>,
    pub keys: Vec<&'a str>
}

impl <'a>TrieTree<'a> {
    pub fn new(keys: &Vec<&'a str>) -> Self {
        let keys = keys.clone();
        let root_ptr = TrieTree::build_ac_automaton(&keys);

        TrieTree {
            root_ptr,
            keys
        }
    }

    fn build_ac_automaton(target_strings:&Vec<&'a str>) -> *mut TrieNode<'a> {
        let root_ptr = TrieNode::new('_');

        for target in target_strings.iter() {
            let mut node = unsafe { &mut *root_ptr };

            for c in target.chars() {
                let matched_child_ptr = node.try_find_child(c);
    
                if matched_child_ptr.is_null() {
                    node.child_ptr = TrieNode::new(c);
                    node = unsafe { &mut *node.child_ptr };
                    continue
                }
    
                let matched_child = unsafe { &mut *matched_child_ptr };
                if matched_child.elem == c {
                    node = matched_child;
                } else  {
                    matched_child.next_ptr = TrieNode::new(c);
                    node = unsafe { &mut *matched_child.next_ptr};
                }
            }
    
    
            node.word = target;
        }
    
        let mut queue: VecDeque<&TrieNode> = VecDeque::new();
        let root = unsafe { & *root_ptr };
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
    
                            if let Some(matched_ptr) = failed_node.find_child(children_sibling.elem) {
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

    pub fn index_of(&self, text: &str) -> BTreeMap<&'a str, Vec<usize>> {
        let mut result:BTreeMap<&'a str, Vec<usize>> = BTreeMap::new();
        for s in self.keys.iter() {
            result.insert(*s, vec![]);
        }

        let root_ptr = self.root_ptr;

        let mut state_node = unsafe { &mut *root_ptr };
    
        for (i, c) in text.char_indices() {
            loop  {
                if let Some(matched_child_ptr)= state_node.find_child(c) {
                    let matched_child = unsafe { &mut *matched_child_ptr };
    
                    // solve subword scenario like `she` and `he`
                    let mut match_node = &mut *matched_child;
                    while !match_node.failed_ptr.is_null() {
                        if match_node.word.len() > 0 {
                            if let Some(list) = result.get_mut(&(match_node.word)) {
                                list.push(i+c.to_string().len()-match_node.word.len());
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
                        print!("{} faileTo: {}: ", sibling.mark_position(), failed_to.mark_position());
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

    use maplit::btreemap;

    use test::Bencher;
    use std::env;
    use std::fs;

    use super::super::super::text:: { bytes2utf8, read_lines };
    use super::*;

    #[test]
    fn ac_automaton_works() {
        let mut result = TrieTree::new(&vec!["bcd", "cdfkcdf", "cde", "abcdef", "abhab"])
            .index_of("bcabcdebcedfabcdefababkabhabk");

        assert_eq!(result, btreemap! {
            "bcd"=> vec![3, 13],
            "cdfkcdf"=> vec![],
            "cde"=> vec![4, 14],
            "abcdef"=> vec![12],
            "abhab"=> vec![23]
        });
        

        result = TrieTree::new(
            &vec!["she", "shr", "say", "he", "her"]).index_of(
            "one day she say her has eaten many shrimps");
    
        assert_eq!(result, btreemap! {
            "she"=> vec![8],
            "he"=> vec![9, 16],
            "say"=> vec![12],
            "her"=> vec![16],
            "shr"=> vec![35]
        });

        result = TrieTree::new(
            &vec!["你好", "你好棒", "今天"]).index_of(
            "你好，今天你看起来好棒"
        );
        assert_eq!(result, btreemap! {
            "你好"=> vec![0],
            "今天"=> vec![9],
            "你好棒"=> vec![],
        });
    }

    #[ignore]
    #[bench]

    /// ```
    /// cargo bench -- algs::ac::tests::bench_ac -- --words xxx.txt file.txt
    /// ```
    fn bench_ac(b: &mut Bencher) {
        let args: Vec<String> = env::args().collect();
        
        let mut words_file: &str = "";
        let mut test_file: &str = "";

        for (index, item) in args.iter().enumerate() {
            if item == "--words" {
                words_file = args[index+1].as_str();
                test_file = args[index+2].as_str();
            }
        }

        assert!(words_file.len() > 0);
        assert!(test_file.len() > 0);

        println!("{:?}", args);

        let mut keys = vec![];
        
        if let Ok(lines) = read_lines(words_file) {
            for line in lines {
                if let Ok(item) = line {

                    let ptr = Box::into_raw(Box::new(item));
                    let value = unsafe { (*ptr).as_str() };
                    keys.push(value);
                }
            }
        }

        let text_raw_contents = fs::read(test_file).expect(format!("read {} failed", words_file).as_str());
        let text_contents = *bytes2utf8(text_raw_contents);

        b.iter(||TrieTree::new(&keys.clone()).index_of(text_contents.as_str()));
    }

}
