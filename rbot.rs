// A Rust IRC bot!

extern mod irc_client;
extern mod oauth;
extern mod std;
extern mod yelp;

use mod std::net_ip;
use irc_client::{Connection, FromSender, JoinMsg, NoSender, NumberedMsg, PrivInMsg, PrivOutMsg};
use irc_client::{UserInfo};
use oauth::{Consumer, Token};

// FIXME: Linker bustage workaround. This is really really terrible.
#[link_args="-lnss3"]
#[nolink]
extern {
}

// FIXME: Botch.
fn slice(s: &a/str) -> &a/str { s }

fn main(args: ~[~str]) {
    let (server, username) = (slice(args[1]), slice(args[2]));
    let channel = slice(args[3]);

    // FIXME: Try ARCs for these.
    let (consumer_key, consumer_secret) = (copy args[4], copy args[5]);
    let (token_key, token_secret) = (copy args[6], copy args[7]);

    let iotask = std::uv_global_loop::get();
    let addr = copy result::unwrap(net_ip::get_addr(server, iotask))[0];
    let userinfo = UserInfo {
        username: username,
        hostname: "asdf.com",
        servername: "localhost",
        realname: "Robots are friendly"
    };

    let note_to_self = username.to_str() + ":";

    let conn = result::unwrap(Connection::make(addr, 6667, username, &userinfo, "x", iotask));

    loop {
        match conn.recv() {
            NumberedMsg(_, 1, _) => break,
            msg => {
                debug!("got msg: %?", msg);
            }
        }
    }

    conn.send(JoinMsg(channel, None));

    loop {
        match conn.recv() {
            PrivInMsg(sender, target, msg)
                    if msg.contains(note_to_self) && msg.contains("lunch") => {
                let result = do task::try |copy consumer_key, copy consumer_secret,
                                           copy token_key, copy token_secret, copy msg,
                                           copy sender| {
                    let rng = rand::Rng();
                    let lunch_start_idx = str::find_str(msg, "lunch").get();

                    let place_start_idx = lunch_start_idx + "lunch ".len();
                    if place_start_idx >= msg.len() { fail; }

                    let place = str::view(msg, place_start_idx, msg.len());
                    if place.len() == 0 { fail; }

                    // Query Yelp.
                    let consumer = Consumer {
                        key: slice(consumer_key),
                        secret: slice(consumer_secret)
                    };
                    let token = Token {
                        key: slice(token_key),
                        secret: slice(token_secret)
                    };
                    let options = yelp::search::Options {
                        term: Some("restaurants"),
                        location: yelp::search::NeighborhoodAddressCity(place)
                    };

                    let yelp_resp = yelp::search::search(rng, &consumer, &token, &options).get();
                    let business_count = yelp_resp.businesses.len() as int;
                    let restaurant = &yelp_resp.businesses[rng.gen_int_range(0, business_count)];

                    let quip;
                    match rng.gen_int_range(0, 3) {
                        0 => quip = ~"How about " + restaurant.name + ~"?",
                        1 => quip = ~"Perhaps you might like to try " + restaurant.name + ~"?",
                        2 => quip = ~"May I suggest " + restaurant.name + ~"?",
                        _ => fail
                    }

                    match sender {
                        NoSender => quip,
                        FromSender(who) => {
                            let colon = str::find_char(who, ':').get();
                            let bang = str::find_char(who, '!').get();
                            str::view(who, colon + 1, bang).to_str() + ~": " + quip
                        }
                    }
                };

                match result {
                    Ok(msg) => {
                        let msg = msg.to_str();
                        conn.send(PrivOutMsg(target, msg));
                    }
                    Err(_) => conn.send(PrivOutMsg(target, "Sorry, I encountered an error."))
                }
            }
            msg => {
                debug!("got msg: %?", msg);
            }
        }
    }
}

