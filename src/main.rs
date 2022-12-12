use crate::config::Config;

mod config;
mod sessions;

fn main() {
    println!("Hello, world!");
    Config::new();
}
