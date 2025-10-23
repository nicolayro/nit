
fn main() -> Result<()> {
    let hello_world = String::from("Hello, world!");

    for c in hello_world.chars() {
        println!("{}", c)
    }

    Ok(())
}
