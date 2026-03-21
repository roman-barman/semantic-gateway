use clap::Parser;

mod server_arguments;

fn main() {
    let _ = server_arguments::ServerArguments::parse();
    println!("Hello, world!");
}
