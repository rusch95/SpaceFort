extern crate spacefort;

// Std lib imports
use std::path::Path;
use std::sync::mpsc::{channel, sync_channel};
use std::thread;

// Local imports
use spacefort::*;
use entities::entity;
use game::server;
use map::tiles;
use net::server::NetComm;


fn setup() -> server::Server {
    // Root points to the directory containing
    // static where assets are loaded from
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let map = tiles::init_map(root);
    let (entities, creature_types) = entity::init_entities(root);
    let comm = stub_comm();

    server::init_server(map, entities, creature_types, comm)
}

fn stub_comm() -> NetComm {
    let (send_outgoing, recv_outgoing) = channel();
    let (send_incoming, recv_incoming) = sync_channel(1024);

    let (send_stream_to_game, recv_stream_to_game) = channel();
    let (send_stream_from_game, recv_stream_from_game) = channel();
    let comm = NetComm::new(send_outgoing, recv_incoming,
                            send_stream_from_game, recv_stream_to_game);
    thread::spawn(move|| {
        loop {
            let _ = recv_outgoing.recv();
        }
    });

    comm
}

#[test]
/// Run one update on the server
fn test_map_startup() {
    let mut server = setup();
    server.update();
}

#[test]
fn test_move_one() {
    let mut server = setup();
    {
        let ent = server.g_state.entities.iter()
                                         .find(|ent| ent.id == -1)
                                         .unwrap();
        assert!(ent.pos != (43, 13, 1));
    }
    server.ent_move(-1, (43, 13, 1));
    for _ in 0..6000 {
        server.update()
    } 
    {
        let ent = server.g_state.entities.iter()
                                         .find(|ent| ent.id == -1)
                                         .unwrap();
        assert_eq!(ent.pos, (43, 13, 1));
    }
}

#[test]
fn test_move_multiple() {
    let mut server = setup();
    for i in -3..-1 {
        let pos = server.g_state.entities.iter()
                                         .find(|ent| ent.id == i)
                                         .unwrap()
                                         .pos;
        assert!(pos != (43, 13, 1));
        server.ent_move(i, (43, 13, 1));
    }
    for _ in 0..6000 {
        server.update()
    }
    for i in -3..-1 {
        let ent = server.g_state.entities.iter()
                                         .find(|ent| ent.id == i)
                                         .unwrap();
        assert_eq!(ent.pos, (43, 13, 1));
    }
}

#[test]
fn test_dig_single_block() {
    let mut server = setup();
    let pos = (28, 19, 1);
    assert!(!server.g_state.map.passable(pos));
    server.players.insert(1, server::ServerPlayer::new(1));
    server.dig(1, (pos, pos));
    for _ in 0..6000 {
        server.update()
    }

    assert!(server.g_state.map.passable(pos));
}

/*
#[test]
fn test_dig_mutiple_blocks() {
    let mut server = setup();
    let sel = ((35, 11, 0), (38, 14, 0));
    for x in 35..39 {
        for y in 11..16 {
            assert!(!server.g_state.map.passable((x, y, 0)));
        }
    }
    server.players.insert(1, server::ServerPlayer::new(1));
    server.dig(1, sel);
    for _ in 0..6000 {
        server.update()
    }

    for x in 35..39 {
        for y in 11..15 {
            assert!(server.g_state.map.passable((x, y, 0)));
        }
    }
}
*/

#[test]
fn test_attack_stationary_unit() {
    let mut server = setup();
    let team_id = 1;
    let attacker_id = -1;
    let defender_id = -4;
    {
        let ent = server.g_state.entities.iter()
                                         .find(|ent| ent.id == defender_id)
                                         .unwrap();
        assert!(ent.alive);
    }
    server.attack(team_id, attacker_id, defender_id) ;
    for _ in 0..6000 {
        server.update()
    }

    {
        let ent = server.g_state.entities.iter()
                                         .find(|ent| ent.id == defender_id)
                                         .unwrap();
        assert!(ent.alive);
    }
}

#[test]
fn test_attack_moving_unit() {
    let mut server = setup();
    let team_id = 1;
    let attacker_id = -1;
    let defender_id = -4;
    {
        let ent = server.g_state.entities.iter()
                                         .find(|ent| ent.id == defender_id)
                                         .unwrap();
        assert!(ent.alive);
    }

    server.attack(team_id, attacker_id, defender_id) ;
    server.ent_move(-4, (43, 13, 1));

    for _ in 0..6000 {
        server.update()
    }

    let ent = server.g_state.entities.iter()
                                     .find(|ent| ent.id == defender_id)
                                     .unwrap();
    assert!(ent.alive);
}

#[test]
fn test_attack_unit_then_move() {
    let mut server = setup();
    let team_id = 1;
    let attacker_id = -1;
    let defender_id = -4;
    {
        let ent = server.g_state.entities.iter()
                                         .find(|ent| ent.id == defender_id)
                                         .unwrap();
        assert!(ent.alive);
    }

    server.attack(team_id, attacker_id, defender_id) ;

    for _ in 0..6000 {
        server.update()
    }

    server.ent_move(-1, (43, 13, 1));

    for _ in 0..6000 {
        server.update()
    }

    {
        let defender = server.g_state.entities.iter()
                                         .find(|ent| ent.id == defender_id)
                                         .unwrap();
        assert!(defender.alive);
        let attacker = server.g_state.entities.iter()
                                         .find(|ent| ent.id == attacker_id)
                                         .unwrap();
        assert_eq!(attacker.pos, (43, 13, 1));
    }
}
