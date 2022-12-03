use std::{fs, path::Path};

// ─── Private ─────────────────────────────────────────────────────────────────

fn create_cart() {
    fs::File::create("temp\\cart.txt").unwrap();
}

fn does_cart_exist() -> bool {
    Path::new("temp\\cart.txt").exists()
}

// ─── Public ──────────────────────────────────────────────────────────────────

pub fn delete_cart() {
    if does_cart_exist() {
        fs::remove_file("temp\\cart.txt").unwrap();
    }
}

pub fn add_to_cart(item: &str) {
    if !does_cart_exist() {
        create_cart();
    }

    let mut lines = get_cart();

    lines.push(item.to_owned());

    fs::write("temp\\cart.txt", lines.join("\n")).unwrap();
}

pub fn remove_from_cart(item: &String) {
    let mut lines = get_cart();

    let index = lines.iter().position(|x| x.eq(item)).unwrap();
    lines.remove(index);

    fs::write("temp\\cart.txt", lines.join("\n")).unwrap();
}

pub fn get_cart() -> Vec<String> {
    if !does_cart_exist() {
        create_cart();
    }

    let vector: Vec<String> = fs::read_to_string("temp\\cart.txt")
        .unwrap()
        .lines()
        .map(|s| s.to_string())
        .collect();

    vector
}
