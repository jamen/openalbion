mod app;
mod renderer;

fn main() {
    env_logger::init();

    let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();

    rt.block_on(app::run());
}
