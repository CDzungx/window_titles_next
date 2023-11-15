use xcb::{
    xproto::{get_property, intern_atom, Atom, ATOM_WINDOW},
    Connection as XConnection, ATOM_INTEGER
};

use crate::{ConnectionTrait, Result, Window};

pub struct Connection {
    connection: XConnection,
    client_list: Atom,
    string: Atom,
    window_name: Atom,
    pid: Atom,
}

impl ConnectionTrait for Connection {
    fn new() -> Result<Self> {
        let connection = XConnection::connect(None)?.0;
        let client_list = intern_atom(&connection, false, "_NET_CLIENT_LIST")
            .get_reply()?
            .atom();
        let string = intern_atom(&connection, false, "UTF8_STRING")
            .get_reply()?
            .atom();
        let window_name = intern_atom(&connection, false, "_NET_WM_NAME")
            .get_reply()?
            .atom();
        let pid = intern_atom(&connection, false, "_NET_WM_PID")
            .get_reply()?
            .atom();
        Ok(Self {
            connection,
            client_list,
            string,
            window_name,
            pid,
        })
    }
    fn window_titles(&self) -> Result<Vec<Window>> {
        let pairs = self
            .connection
            .get_setup()
            .roots()
            .map(|screen| screen.root())
            .map(|window| {
                (
                    get_property(
                        &self.connection,
                        false,
                        window,
                        self.client_list,
                        ATOM_WINDOW,
                        0,
                        1024,
                    ),
                    get_property(
                        &self.connection,
                        false,
                        window,
                        self.pid,
                        ATOM_INTEGER,
                        0,
                        1024
                    )
                )
            })
            .filter_map(|cookie| Some((
                cookie.0.get_reply().ok(),
                cookie.1.get_reply().ok()
            )));

        let windows = pairs.filter_map(|pair| {
            let window = pair.0?.value()[0];
            let pid = pair.1?;
            let pid = if pid.value_len() <= 0 { 0 } else { pid.value()[0] };

            let name = get_property(
                &self.connection,
                false,
                window,
                self.window_name,
                self.string,
                0,
                1024,
            ).get_reply().ok()?.value().to_vec();

            let name = String::from_utf8(name).ok()?;
            let name = name.trim_matches('\0').to_string();

            Some(Window { title: name, pid })
        }).collect();

        Ok(windows)
    }
}
