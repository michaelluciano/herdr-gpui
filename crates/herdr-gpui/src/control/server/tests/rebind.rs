//! A second app instance takes over the control socket once the first quits.
#![allow(clippy::unwrap_used)]
use super::*;

fn held_by_another(path: &Path) -> bool {
    matches!(socket::Server::bind(path), Err(Error::SocketInUse { .. }))
}

#[gpui::test]
fn a_second_instance_takes_the_socket_when_the_first_quits(cx: &mut gpui::TestAppContext) {
    let address = socket::test_address();
    let _first = socket::Server::bind(&address.path).unwrap();
    cx.update(|cx| install_at(address.path.clone(), cx));
    cx.run_until_parked();
    cx.executor().advance_clock(REBIND_EVERY);
    cx.run_until_parked();
    // Still the first instance's socket: nothing was taken from a live server.
    assert!(held_by_another(&address.path));
    // The first instance quits and removes its socket file, as `release` does.
    socket::release(&address.path);
    cx.executor().advance_clock(REBIND_EVERY);
    cx.run_until_parked();
    assert!(
        held_by_another(&address.path),
        "the second instance should be listening now"
    );
}
