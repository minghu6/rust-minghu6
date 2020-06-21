use std::fmt;

use minghu6::collections::key_hash_set::{ KeyHashSet, GetKeyType, IteratorWrapper };

#[test]
fn create_cutomhashset_basictype() {
    let mut myset = KeyHashSet::new(None);

    myset.insert("a");
    myset.insert("b");
    myset.insert("c");

    assert!(myset.contains(&"a"));
    assert!(myset.contains(&"b"));
    assert!(myset.contains(&"c"));
    debug_assert_eq!(myset.contains(&"d"), false);
    assert!(myset.contains(&"a"));

    // test remove
    myset.remove(&"a");
    myset.remove(&"c");
    assert!(!myset.contains(&"a"));
    assert!(myset.contains(&"b"));
    assert!(!myset.contains(&"c"));
}

#[test]
fn create_cutomhashset_struct() {
    #[derive(Hash, Clone)]
    struct Person {
        id: u32,
        name: String,
        phone: u64,
    }

    let person1 = Person {
        id: 5,
        name: "Janet".to_string(),
        phone: 555_666_7777,
    };

    let person2 = Person {
        id: 5,
        name: "Byn".to_string(),
        phone: 555_666_7777,
    };

    let person3 = Person {
        id: 6,
        name: "Janet".to_string(),
        phone: 888_999_000,
    };

    let person4 = Person {
        id: 5,
        name: "BiboBibo".to_string(),
        phone: 555_666_7777,
    };

    // fn GET_KEY_FUNC(person: &Person) -> String {
    //     String::from(&person.name)
    // }

    let get_key_func_byname= |person: &Person| String::from(&person.name);

    let get_key = Some(get_key_func_byname as GetKeyType<Person>);

    let mut myset:KeyHashSet<Person> = KeyHashSet::new(get_key);

    myset.insert(person1);
    myset.insert(person2);

    assert!(myset.contains(&person3));
    assert!(!myset.contains(&person4));
}


#[test]
fn wrap_a_iterator_1() {
    use std::collections::HashSet;

    let mut myset = HashSet::new();

    myset.insert("a");
    myset.insert("b");
    myset.insert("c");

    IteratorWrapper::new(0..3).for_each(|x| print!("{} ",x));
    IteratorWrapper::new(myset.drain()).for_each(|x| print!("{} ",x));

}

#[test]
fn for_into_iterator() {
    use std::collections::HashSet;

    let mut myset = HashSet::new();

    myset.insert("a");
    myset.insert("b");
    myset.insert("c");

    for v in myset {
        print!("{} ",v);
    } println!("");

}

#[test]
fn tellme_set_relationship_basictype() {
    let mut set1 = KeyHashSet::new(None);
    set1.insert("a");
    set1.insert("b");
    set1.insert("c");

    let mut set2 = KeyHashSet::new(None);
    set2.insert("a");
    set2.insert("b");

    assert!(set1.is_superset(&set2));
    assert!(set2.is_subset(&set1));

    let mut set3 = KeyHashSet::new(None);
    set3.insert("a");
    set3.insert("b");

    assert!(set3.is_superset(&set2));
    assert!(set2.is_subset(&set3));

    let set4:KeyHashSet<&str> = KeyHashSet::new(None);
    assert!(set4.is_empty());

    assert!(set1.is_disjoint(&set4));
    assert!(set4.is_disjoint(&set1));
}

#[test]
fn set_op_basictype() {
    let mut set1 = KeyHashSet::new(None);
    set1.insert("a");
    set1.insert("b");
    set1.insert("c");

    let mut set2 = KeyHashSet::new(None);
    set2.insert("d");
    set2.insert("b");
    set2.insert("e");

    // test union
    let unioned_set = set1.union(&set2);

    let mut set3 = KeyHashSet::new(None);
    set3.insert("b");

    assert_eq!(unioned_set, set3);

    // test intersection
    let intersectioned_set =  set1.intersection(&set2);
    let mut set4 = KeyHashSet::new(None);
    set4.insert("a");
    set4.insert("b");
    set4.insert("c");
    set4.insert("d");
    set4.insert("e");

    assert_eq!(intersectioned_set, set4);
    assert_eq!(set1.intersection(&set3), set1);

    // test difference
    let differenced_set = set1.difference(&set2);
    let mut set5 = KeyHashSet::new(None);
    set5.insert("a");
    set5.insert("c");

    assert_eq!(differenced_set, set5);

    // test symmertic_difference
    let mut set6 = KeyHashSet::new(None);
    set6.insert("a");
    set6.insert("c");
    set6.insert("d");
    set6.insert("e");
    assert_eq!(set1.symmetric_difference(&set2), set6);
}

#[test]
fn set_io_basictype() {
    let mut set1 = KeyHashSet::new(None);
    set1.insert("a");
    set1.insert("b");
    set1.insert("c");

    // test remove
    assert!(set1.remove(&"a"));
    assert!(!set1.contains(&"a"));
    assert!(!set1.remove(&"e"));

    // test take
    match set1.take(&"b") {
        Some(v) => assert_eq!(v, "b"),
        None => assert!(false)
    }

    assert!(!set1.contains(&"b"));

    // test get
    match set1.get(&"c") {
        Some(v) => assert_eq!(v, &"c"),
        None => assert!(false)
    }

    assert!(set1.contains(&"c"))
}

#[derive(Hash, Clone, fmt::Debug)]
struct Person {
    id: u32,
    name: String,
    phone: u64,
}

impl PartialEq for Person {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

fn gen_person_sample(identifier: &str) -> Person {

    match identifier {
        "a" => Person{
            id: 5,
            name: "Janet".to_string(),
            phone: 555_666_7777,
        },
        "b" => Person {
            id: 6,
            name: "Byn".to_string(),
            phone: 222_333_4444,
        },
        "c" => Person {
            id: 7,
            name: "Janet".to_string(),
            phone: 888_999_0000,
        },
        "d" => Person {
            id: 8,
            name: "Jun".to_string(),
            phone: 888_999_0000,
        },
        "e" => Person {
            id: 9,
            name: "Kat".to_string(),
            phone: 678_123_4567,
        },
        _ => Person {
            id: 0,
            name: "anonymous".to_string(),
            phone: 000_000_0000,
        }
    }
}

static GET_KEY_FUNC:GetKeyType<Person> = |person: &Person| person.id.to_string();

#[test]
fn tellme_set_relationship_struct() {
    let mut set1 = KeyHashSet::new(Some(GET_KEY_FUNC));
    set1.insert(gen_person_sample("a"));
    set1.insert(gen_person_sample("b"));
    set1.insert(gen_person_sample("c"));

    let mut set2 = KeyHashSet::new(Some(GET_KEY_FUNC));
    set2.insert(gen_person_sample("a"));
    set2.insert(gen_person_sample("b"));

    assert!(set1.is_superset(&set2));
    assert!(set2.is_subset(&set1));

    let mut set3 = KeyHashSet::new(Some(GET_KEY_FUNC));
    set3.insert(gen_person_sample("a"));
    set3.insert(gen_person_sample("b"));

    assert!(set3.is_superset(&set2));
    assert!(set2.is_subset(&set3));

    let set4:KeyHashSet<Person> = KeyHashSet::new(Some(GET_KEY_FUNC));
    assert!(set4.is_empty());

    assert!(set1.is_disjoint(&set4));
    assert!(set4.is_disjoint(&set1));
}

#[test]
fn set_op_struct() {
    let mut set1 = KeyHashSet::new(None);
    set1.insert(gen_person_sample("a"));
    set1.insert(gen_person_sample("b"));
    set1.insert(gen_person_sample("c"));

    let mut set2 = KeyHashSet::new(None);
    set2.insert(gen_person_sample("b"));
    set2.insert(gen_person_sample("d"));
    set2.insert(gen_person_sample("e"));

    // test union
    let unioned_set = set1.union(&set2);

    let mut set3 = KeyHashSet::new(None);
    set3.insert(gen_person_sample("b"));

    assert_eq!(unioned_set, set3);

    // test intersection
    let intersectioned_set =  set1.intersection(&set2);
    let mut set4 = KeyHashSet::new(None);
    set4.insert(gen_person_sample("a"));
    set4.insert(gen_person_sample("b"));
    set4.insert(gen_person_sample("c"));
    set4.insert(gen_person_sample("d"));
    set4.insert(gen_person_sample("e"));

    assert_eq!(intersectioned_set, set4);
    assert_eq!(set1.intersection(&set3), set1);

    // test difference
    let differenced_set = set1.difference(&set2);
    let mut set5 = KeyHashSet::new(None);
    set5.insert(gen_person_sample("a"));
    set5.insert(gen_person_sample("c"));

    assert_eq!(differenced_set, set5);

    // test symmertic_difference
    let mut set6 = KeyHashSet::new(None);
    set6.insert(gen_person_sample("a"));
    set6.insert(gen_person_sample("c"));
    set6.insert(gen_person_sample("d"));
    set6.insert(gen_person_sample("e"));
    assert_eq!(set1.symmetric_difference(&set2), set6);
}

#[test]
fn set_io_struct() {
    let mut set1 = KeyHashSet::new(None);
    set1.insert(gen_person_sample("a"));
    set1.insert(gen_person_sample("b"));
    set1.insert(gen_person_sample("c"));

    // test remove
    assert!(set1.remove(&gen_person_sample("a")));
    assert!(!set1.contains(&gen_person_sample("a")));
    assert!(!set1.remove(&gen_person_sample("e")));

    // test take
    match set1.take(&gen_person_sample("b")) {
        Some(v) => assert_eq!(v, gen_person_sample("b")),
        None => assert!(false)
    }

    assert!(!set1.contains(&gen_person_sample("b")));

    // test get
    match set1.get(&gen_person_sample("c")) {
        Some(v) => assert_eq!(v, &gen_person_sample("c")),
        None => assert!(false)
    }

    assert!(set1.contains(&gen_person_sample("c")))
}

