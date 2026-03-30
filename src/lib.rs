#[expect(
    clippy::print_stdout,
    reason = "This is a simple example function that prints to the console."
)]
pub fn run() {
    println!("Hello, world!");
}
