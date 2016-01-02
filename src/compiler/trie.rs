use std::sync::Arc;
use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone, Debug)]
enum TrieNode<'a, R> where R: Clone + PartialEq<R> {
    Null,
    End(R),
    SubTrie(Box<Trie<'a, R>>),
    SubEndTrie(R, Box<Trie<'a, R>>)
}

#[derive(Clone)]
pub struct Trie<'a, R> where R: Clone + PartialEq<R> {
    list: Vec<TrieNode<'a, R>>,
    index: Arc<&'a Fn(char) -> i32>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TrieError {
    End, // Caused by a TrieNode::End(_) at the location
    SubTrie, // Caused by a TrieNode::SubTrie(_) at the location
    Null, // Caused by a TrieNode::Null at the location
    NoHash, // Caused by -1 from index function
    NoChar // Cause by no char in the search string
}

impl<'a, R> Debug for Trie<'a, R>
    where R: Clone + PartialEq<R> + Debug {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct("Trie")
            .field("list", &self.list)
            .finish()
    }
}

impl<'a, R> Trie<'a, R>
    where R: Clone + PartialEq<R> {
    pub fn new(hspace: usize, f: &Fn(char) -> i32) -> Trie<R> {
        let mut tmp = Trie {
            list: Vec::with_capacity(hspace),
            index: Arc::new(f)
        };

        for _ in 0..hspace {
            tmp.list.push(TrieNode::Null);
        };
        tmp
    }

    fn new_with_arc(hspace: usize, f: Arc<&Fn(char) -> i32>) -> Trie<R> {
        let mut tmp = Trie {
            list: Vec::with_capacity(hspace),
            index: f
        };

        for _ in 0..hspace {
            tmp.list.push(TrieNode::Null);
        };
        tmp
    }

    pub fn add_string(&mut self, f: &str, end: R) {
        match f.chars().nth(0) {
            Some(c) => {
                let index = &self.index;
                let lindex = index(c);
                if lindex != -1  && ((lindex as usize) < self.list.len() || lindex >= 0) {
                    let y = self.list.iter().cloned().nth(lindex as usize);
                    match y {
                        Some(TrieNode::Null) => {
                            if f.len() > 1 {
                                // subtrie
                                let mut nsubtrie: Trie<R> = Trie::new_with_arc(self.list.len(), index.clone());
                                nsubtrie.add_string(&*f.chars().skip(1).collect::<String>(), end);
                                self.list.remove(lindex as usize);
                                self.list.insert(lindex as usize, TrieNode::SubTrie(Box::new(nsubtrie)));
                            } else {
                                // end
                                self.list.remove(lindex as usize);
                                self.list.insert(lindex as usize, TrieNode::End(end));
                            }
                        },
                        Some(TrieNode::End(ref obj)) => {
                            if f.len() > 1 {
                                // make node subendtrie
                                let mut nsubtrie: Trie<R> = Trie::new_with_arc(self.list.len(), index.clone());
                                nsubtrie.add_string(&*f.chars().skip(1).collect::<String>(), end);
                                self.list.remove(lindex as usize);
                                self.list.insert(lindex as usize, TrieNode::SubEndTrie(obj.clone(), Box::new(nsubtrie)));
                            } else {
                                // They want to change the ending (if they are different)
                                if obj != &end {
                                    self.list.remove(lindex as usize);
                                    self.list.insert(lindex as usize, TrieNode::End(end));
                                }
                            }
                        },
                        Some(TrieNode::SubTrie(ref trie)) => {
                            if f.len() > 1 {
                                // Sumbit to your superior officer
                                let mut ltrie = trie.clone();
                                ltrie.add_string(&*f.chars().skip(1).collect::<String>(), end);
                                self.list.remove(lindex as usize);
                                self.list.insert(lindex as usize, TrieNode::SubTrie(ltrie));
                            } else {
                                // make node subendtrie
                                self.list.remove(lindex as usize);
                                self.list.insert(lindex as usize, TrieNode::SubEndTrie(end, trie.clone()));
                            }
                        },
                        Some(TrieNode::SubEndTrie(ref obj, ref trie)) => {
                            if f.len() > 1 {
                                // Sumbit to your superior officer
                                let mut ltrie = trie.clone();
                                ltrie.add_string(&*f.chars().skip(1).collect::<String>(), end);
                                self.list.remove(lindex as usize);
                                self.list.insert(lindex as usize, TrieNode::SubEndTrie(obj.clone(), ltrie));
                            } else {
                                // They want to change the ending (if they are different)
                                if obj != &end {
                                    self.list.remove(lindex as usize);
                                    self.list.insert(lindex as usize, TrieNode::SubEndTrie(end, trie.clone()));
                                }
                            }
                        }
                        None => {}
                    }
                } else {
                    panic!("Attempt to index nonindexable char: {}", c);
                }
            },
            None => {}
        }
    }

    pub fn search(&self, f: &str) -> Result<R, TrieError> {
        match f.chars().nth(0){
            Some(c) => {
                let index = &self.index;
                let lindex = index(c);
                match lindex {
                    -1 => Err(TrieError::NoHash),
                    _ => {
                        let y = self.list.iter().cloned().nth(lindex as usize);
                        match y {
                            Some(TrieNode::Null) => Err(TrieError::Null),
                            Some(TrieNode::End(ref e)) => {
                                if f.len() > 1 {
                                    Err(TrieError::End) // You want to continue, but we have no children
                                } else {
                                    Ok(e.clone())
                                }
                            },
                            Some(TrieNode::SubEndTrie(ref e, ref trie)) => {
                                if f.len() > 1 {
                                    trie.search(&*f.chars().skip(1).collect::<String>())
                                } else {
                                    Ok(e.clone())
                                }
                            }
                            Some(TrieNode::SubTrie(ref trie)) => {
                                if f.len() > 1 {
                                    trie.search(&*f.chars().skip(1).collect::<String>())
                                } else {
                                    Err(TrieError::SubTrie)
                                }
                            },
                            None => unreachable!("Space size doesn't match hash size")
                        }
                    }
                }
            },
            None => Err(TrieError::NoChar)
        }
    }
}
