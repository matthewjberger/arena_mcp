#[tokio::main]
async fn main() -> anyhow::Result<()> {
    arena::serve().await
}
