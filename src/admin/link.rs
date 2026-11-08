use crate::admin::{Frame, Incoming, Order, Report};
use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex, mpsc},
    thread,
};

/// Port local par défaut du lien entre la simulation et la fenêtre d'administration.
pub(crate) const DEFAULT_PORT: u16 = 47821;

/// Côté simulation du lien avec la fenêtre d'administration : un serveur local qui reçoit les
/// demandes sur un fil d'exécution à part et les remet à la boucle de simulation.
pub(crate) struct AdminLink {
    incoming: mpsc::Receiver<Incoming>,
    client: Arc<Mutex<Option<TcpStream>>>,
}

impl AdminLink {
    /// Écoute sur `127.0.0.1:port` (jamais sur le réseau).
    pub(crate) fn listen(port: u16) -> Result<Self, String> {
        let listener = TcpListener::bind(("127.0.0.1", port))
            .map_err(|e| format!("Fenêtre d'administration : port {port} indisponible ({e})."))?;
        let (sender, incoming) = mpsc::channel();
        let client: Arc<Mutex<Option<TcpStream>>> = Arc::default();
        let shared = client.clone();
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let Ok(reader) = stream.try_clone() else { continue };
                if let Ok(mut slot) = shared.lock() {
                    *slot = Some(stream);
                }
                if sender.send(Incoming::Connected).is_err() {
                    return;
                }
                for line in BufReader::new(reader).lines().map_while(Result::ok) {
                    match Frame::decode::<Order>(&line) {
                        Ok(order) => {
                            if sender.send(Incoming::Order(order)).is_err() {
                                return;
                            }
                        }
                        Err(error) => eprintln!("Cell Lab : demande d'administration illisible ({error})."),
                    }
                }
                if let Ok(mut slot) = shared.lock() {
                    *slot = None;
                }
            }
        });
        Ok(Self { incoming, client })
    }

    /// Tout ce qui est arrivé depuis le dernier appel.
    pub(crate) fn poll(&self) -> Vec<Incoming> {
        self.incoming.try_iter().collect()
    }

    pub(crate) fn connected(&self) -> bool {
        self.client.lock().is_ok_and(|slot| slot.is_some())
    }

    /// Envoie un rapport à la fenêtre connectée ; sans fenêtre, ne fait rien.
    pub(crate) fn send(&self, report: &Report) {
        let Ok(mut slot) = self.client.lock() else { return };
        let Some(stream) = slot.as_mut() else { return };
        let Ok(line) = Frame::encode(report) else { return };
        if writeln!(stream, "{line}").and_then(|()| stream.flush()).is_err() {
            *slot = None;
        }
    }
}
