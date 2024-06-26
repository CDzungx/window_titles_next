use window_titles::{Connection, ConnectionTrait};

fn main() {
    let connection = Connection::new().unwrap();
    connection
        .window_titles()
        .unwrap()
        .into_iter()
        .for_each(|win| println!("{}: {}", win.pid, win.title))
}
