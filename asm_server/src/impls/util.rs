use std::future::Future;
use std::sync::Arc;
use tokio::runtime;
use tokio::runtime::Runtime;

pub fn new_tokio_thread<F, Fut>(async_logic: F)
where
    F: FnOnce(Arc<Runtime>) -> Fut + Send + 'static,
    Fut: Future,
{
    std::thread::spawn(move || {
        let runtime: Arc<Runtime> = runtime::Builder::new_multi_thread()
            .enable_all().build().unwrap().into();
        let copied_runtime = runtime.clone();
        runtime.block_on(async move {
            let runtime = copied_runtime;
            async_logic(runtime).await;
        });
    });
}
