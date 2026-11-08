use crate::admin::{Frame, Order, Report};
use crate::panel::Inbox;
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

/// Lien de la fenêtre d'administration avec la simulation : un fil d'exécution se (re)connecte,
/// lit les rapports et envoie les demandes.
pub(crate) struct Connection {
    orders: mpsc::Sender<Order>,
    inbox: Arc<Mutex<Inbox>>,
}

impl Connection {
    pub(crate) fn start(port: u16, repaint: impl Fn() + Send + 'static) -> Self {
        let (orders, receiver) = mpsc::channel::<Order>();
        let inbox: Arc<Mutex<Inbox>> = Arc::default();
        let shared = inbox.clone();
        thread::spawn(move || {
            loop {
                let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) else {
                    thread::sleep(Duration::from_millis(700));
                    continue;
                };
                let _ = stream.set_read_timeout(Some(Duration::from_millis(40)));
                if let Ok(mut inbox) = shared.lock() {
                    inbox.connected = true;
                }
                repaint();
                let Ok(clone) = stream.try_clone() else { continue };
                let mut reader = BufReader::new(clone);
                let mut line = String::new();
                'session: loop {
                    for order in receiver.try_iter() {
                        let Ok(text) = Frame::encode(&order) else { continue };
                        if writeln!(stream, "{text}").and_then(|()| stream.flush()).is_err() {
                            break 'session;
                        }
                    }
                    match reader.read_line(&mut line) {
                        Ok(0) => break 'session,
                        Ok(_) if line.ends_with('\n') => {
                            if let (Ok(report), Ok(mut inbox)) = (Frame::decode::<Report>(line.trim()), shared.lock()) {
                                Self::deposit(&mut inbox, report);
                            }
                            line.clear();
                            repaint();
                        }
                        Ok(_) => {}
                        Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
                        Err(_) => break 'session,
                    }
                }
                if let Ok(mut inbox) = shared.lock() {
                    inbox.connected = false;
                }
                repaint();
            }
        });
        Self { orders, inbox }
    }

    fn deposit(inbox: &mut Inbox, report: Report) {
        match report {
            Report::Definitions { ron, save_path } => {
                inbox.definitions = Some(ron);
                inbox.save_path = save_path;
            }
            Report::Status(status) => inbox.status = Some(status),
            Report::Done { ok, message } => {
                inbox.log.push((ok, message));
                if inbox.log.len() > 200 {
                    inbox.log.remove(0);
                }
            }
        }
    }

    pub(crate) fn send(&self, order: Order) {
        let _ = self.orders.send(order);
    }

    pub(crate) fn inbox(&self) -> &Arc<Mutex<Inbox>> {
        &self.inbox
    }
}
