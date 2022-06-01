use tokio::{process::Command, test};

#[test]
async fn smoke() {
    let mut cmd = Command::new("cargo")
        .arg("run")
        .arg("../fixtures/big_decimals.csv")
        .spawn()
        .expect("failed to spawn cargo");
    let exit = cmd.wait().await.expect("failed to run cargo");
    assert!(
        exit.success(),
        "cargo run did not succeed exit code {:?}",
        exit.code()
    )
}
