use std::rc::Rc;

use minghu6::collections::{ CustomHashSet, GetKeyType };

#[test]
fn create_cutomhashset_basictype() {
    let mut myset = CustomHashSet::new(None);

    myset.insert("a");
    myset.insert("b");
    myset.insert("c");

    assert!(myset.contains(&"a"));
    assert!(myset.contains(&"b"));
    assert!(myset.contains(&"c"));
    debug_assert_eq!(myset.contains(&"d"), false);
    assert!(myset.contains(&"a"));
}

#[test]
fn create_cutomhashset_struct() {
    #[derive(Hash)]
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

    // fn get_key_func(person: &Person) -> String {
    //     String::from(&person.name)
    // }

    let get_key_func= |person: &Person| String::from(&person.name);

    let get_key = Some(get_key_func as GetKeyType<Person>);

    let mut myset:CustomHashSet<Person> = CustomHashSet::new(get_key);

    myset.insert(person1);
    myset.insert(person2);
    //myset.insert(person3);

    assert!(myset.contains(&person3));
    assert!(!myset.contains(&person4));
}

#[test]
fn create_drop() {

}

