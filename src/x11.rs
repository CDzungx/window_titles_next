use xcb::{Connection as XConnection, x::{self, Atom}};

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
        let client_list = connection.send_request(&x::InternAtom {
            only_if_exists: false,
            name: "_NET_CLIENT_LIST".as_bytes(),
        });
        let client_list = connection.wait_for_reply(client_list)?.atom();
    
        let string = connection.send_request(&x::InternAtom {
            only_if_exists: false,
            name: "UTF8_STRING".as_bytes(),
        });
        let string = connection.wait_for_reply(string)?.atom();
    
        let window_name = connection.send_request(&x::InternAtom {
            only_if_exists: false,
            name: "_NET_WM_NAME".as_bytes(),
        });
        let window_name = connection.wait_for_reply(window_name)?.atom();

        let pid = connection.send_request(&x::InternAtom {
            only_if_exists: false,
            name: "_NET_WM_PID".as_bytes(),
        });
        let pid = connection.wait_for_reply(pid)?.atom();

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
                    self.connection.send_request(&x::GetProperty {
                        delete: false,
                        window,
                        property: self.client_list,
                        r#type: x::ATOM_NONE,
                        long_offset: 0,
                        long_length: 1024,
                    }),
                    self.connection.send_request(&x::GetProperty {
                        delete: false,
                        window,
                        property: self.pid,
                        r#type: x::ATOM_CARDINAL,
                        long_offset: 0,
                        long_length: 4
                    })
                )
            })
            .filter_map(|cookie| Some((
                self.connection.wait_for_reply(cookie.0).ok(),
                self.connection.wait_for_reply(cookie.1).ok()
            )));

        let windows = pairs.filter_map(|pair| {
            let window = pair.0?.value()[0];
            let pid = pair.1?;
            let pid = if pid.value::<u32>().len() <= 0 { 0 } else { pid.value()[0] };

            let name = self.connection.send_request(&x::GetProperty {
                delete: false,
                window,
                property: self.window_name,
                r#type: self.string,
                long_offset: 0,
                long_length: 1024,
            });

            let name: Vec<u8> = self.connection.wait_for_reply(name).ok()?.value().to_vec();
            let name = String::from_utf8(name).ok()?;
            let name = name.trim_matches('\0').to_string();

            Some(Window { title: name, pid })
        }).collect();

        Ok(windows)
    }
}
