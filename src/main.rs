pub mod app;

pub fn main() {
    let app = app::App::new();
    let appresult = app.run();
    if let Err(err) = appresult {
        println!("The app end with error: {:?}", err);
    }

    println!("The Application is End !");
}
