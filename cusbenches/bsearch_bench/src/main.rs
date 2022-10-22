

fn main() {
    println!("Hello, world!");

    let input = [1, 5, 2, 34, 72, 2, 44, 44, 42, 3];

    let res = input.binary_search(&5);

    println!("{res:?}");
}
