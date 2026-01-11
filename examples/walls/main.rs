mod gltf;
mod svg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running SVG examples...");
    svg::run()?;
    println!("Running GLTF examples...");
    gltf::run()?;
    Ok(())
}
