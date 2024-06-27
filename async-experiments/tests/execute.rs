use async_experiments::Executor;

#[test]
fn execute() {
    let executor = Executor::new();

    executor.block_on(async {});
    executor.block_on(async {});
}

#[test]
fn join2() {
    let exec = Executor::new();

    let r = exec.block_on(async {
        let t1 = async_experiments::spawn_blocking(|| 1);
        let t2 = async_experiments::spawn_blocking(|| 2);

        let (r1, r2) = async_experiments::join2(t1, t2).await;
        r1 + r2
    });
    assert_eq!(r, 3)
}
