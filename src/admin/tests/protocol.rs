use crate::admin::{AdminLink, Frame, Incoming, Order, Report};
use crate::definitions::{Definitions, FounderDefinition};
use crate::ecosystem::{Intervention, KillTarget};
use std::{io::Write, net::TcpStream, time::{Duration, Instant}};

#[test]
fn messages_survive_the_wire_on_one_line() {
    let order = Order::Intervene(Intervention::Spawn { group: FounderDefinition::default(), at: Some((2, 3)) });
    let line = Frame::encode(&order).unwrap();
    assert!(!line.contains('\n'));
    assert_eq!(Frame::decode::<Order>(&line).unwrap(), order);
    let kill = Order::Intervene(Intervention::Kill(KillTarget::Fraction(0.5)));
    assert_eq!(Frame::decode::<Order>(&Frame::encode(&kill).unwrap()).unwrap(), kill);
}

#[test]
fn the_whole_catalog_travels_as_text_and_comes_back_identical() {
    let definitions = Definitions::builtin();
    let text = ron::ser::to_string(&*definitions).unwrap();
    let back: Definitions = ron::from_str(&text).unwrap();
    back.validate().unwrap();
    assert_eq!(ron::ser::to_string(&back).unwrap(), text);
    let report = Report::Definitions { ron: text, save_path: "runtime/session.bin".into() };
    assert_eq!(Frame::decode::<Report>(&Frame::encode(&report).unwrap()).unwrap(), report);
}

#[test]
fn a_connected_window_sends_orders_and_receives_reports() {
    let port = 47_999;
    let link = AdminLink::listen(port).expect("port libre");
    let mut window = TcpStream::connect(("127.0.0.1", port)).unwrap();
    writeln!(window, "{}", Frame::encode(&Order::SetPaused(true)).unwrap()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut seen = Vec::new();
    while seen.len() < 2 && Instant::now() < deadline {
        seen.extend(link.poll());
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(matches!(seen[0], Incoming::Connected));
    assert!(matches!(seen[1], Incoming::Order(Order::SetPaused(true))));
    assert!(link.connected());
    link.send(&Report::Done { ok: true, message: "ok".into() });
    let mut line = String::new();
    std::io::BufRead::read_line(&mut std::io::BufReader::new(&window), &mut line).unwrap();
    assert!(matches!(Frame::decode::<Report>(line.trim()).unwrap(), Report::Done { ok: true, .. }));
}
