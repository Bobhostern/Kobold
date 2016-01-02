use super::compiler::trie::Trie;
use super::compiler::trie::TrieError;

#[test]
fn test_trie() {
    let h = |a| match a {
        e @ 'a'...'z' => e as i32 - 'a' as i32,
        _ => -1
    };
    let mut t = Trie::new(26, &h);
    t.add_string("apple", 1);
    t.add_string("tree", 2i32);
    t.add_string("app", 3);
    t.add_string("turn", 4);
    t.add_string("y", 5);

    assert_eq!(t.search("apple").ok(), Some(1));
    assert_eq!(t.search("app").ok(), Some(3));
    assert_eq!(t.search("appl").err(), Some(TrieError::SubTrie));
    assert_eq!(t.search("turn").ok(), Some(4));
    assert_eq!(t.search("turns").err(), Some(TrieError::End));
    assert_eq!(t.search("x").err(), Some(TrieError::Null));
    assert_eq!(t.search("").err(), Some(TrieError::NoChar));
    assert_eq!(t.search("!").err(), Some(TrieError::NoHash));
    assert_eq!(t.search("y").ok(), Some(5));
    assert_eq!(t.search("tree").ok(), Some(2));
}
